use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::Mirroring,
    ppu::{address::Address, control::Control, mask::Mask, status::Status},
    NromCartridge,
};

mod address;
mod control;
mod mask;
mod status;

/// The size of a nametable in bytes.
const NAMETABLE_SIZE: usize = 1024;
/// The size of the palette memory in bytes.
const PALETTES_SIZE: usize = 32;

pub struct Ppu {
    cartridge: Rc<RefCell<NromCartridge>>,
    pub nametables: [u8; 2 * NAMETABLE_SIZE],
    palettes: [u8; PALETTES_SIZE],

    control: Control,
    mask: Mask,
    status: Status,
    latch: bool,
    address: Address,
    temp_address: Address,
    fine_x_scroll: u8,

    read_buffer: u8,

    cycle: u16,
    scanline: u16,
    pub nmi: bool,
    pub is_frame_ready: bool,

    nametable_latch: u8,
    attribute_byte: u8,
    tile_low: u8,
    tile_high: u8,

    background_shift_low: u16,
    background_shift_high: u16,
    palette_shifter_low: u8,
    palette_shifter_high: u8,
    attribute_latch_low: u8,
    attribute_latch_high: u8,

    frame: Vec<(u8, u8)>,
    pub done_frame: Vec<(u8, u8)>,
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<NromCartridge>>) -> Ppu {
        Ppu {
            cartridge,
            nametables: [0; 2 * NAMETABLE_SIZE],
            palettes: [0; PALETTES_SIZE],
            control: Control(0),
            mask: Mask(0),
            status: Status(0),
            latch: true,
            address: Address(0),
            temp_address: Address(0),
            fine_x_scroll: 0,
            read_buffer: 0,
            cycle: 0,
            scanline: 0,
            nmi: false,
            is_frame_ready: false,
            nametable_latch: 0,
            attribute_byte: 0,
            tile_low: 0,
            tile_high: 0,
            background_shift_low: 0,
            background_shift_high: 0,
            palette_shifter_low: 0,
            palette_shifter_high: 0,
            attribute_latch_low: 0,
            attribute_latch_high: 0,
            frame: vec![(0, 0); 240 * 256],
            done_frame: vec![(0, 0); 240 * 256],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.cartridge.borrow().read_chr(address),
            0x2000..=0x3eff => {
                let address = match self.cartridge.borrow().mirroring() {
                    Mirroring::Horizontal => {
                        let base = (address / 2) & NAMETABLE_SIZE as u16;
                        let offset = address % NAMETABLE_SIZE as u16;
                        base + offset
                    }
                    Mirroring::Vertical => {
                        address % (2 * NAMETABLE_SIZE as u16)
                    }
                };

                self.nametables[address as usize]
            }
            0x3f00..=0x3fff => {
                // 0x3f20 - 0x3fff is a mirror of 0x3f00 - 0x3f1f.
                let address = address & 0x1f;
                let address = match address {
                    0x10 | 0x14 | 0x18 | 0x1c => address - 0x10,
                    _ => address,
                };

                self.palettes[address as usize]
            }
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => (),
            0x2000..=0x3eff => {
                let address = address & 0x0fff;
                let address = match self.cartridge.borrow().mirroring() {
                    Mirroring::Horizontal => {
                        let base = (address / 2) & NAMETABLE_SIZE as u16;
                        let offset = address % NAMETABLE_SIZE as u16;
                        base + offset
                    }
                    Mirroring::Vertical => {
                        address % (2 * NAMETABLE_SIZE as u16)
                    }
                };

                self.nametables[address as usize] = data;
            }
            0x3f00..=0x3fff => {
                // 0x3f20 - 0x3fff is a mirror of 0x3f00 - 0x3f1f.
                let address = address & 0x1f;
                let address = match address {
                    0x10 | 0x14 | 0x18 | 0x1c => address - 0x10,
                    _ => address,
                };

                self.palettes[address as usize] = data;
            }
            _ => unreachable!(),
        }
    }

    pub fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => {
                let data = self.status.0 | (self.read_buffer & 0x1f);
                self.status.set_vblank(false);
                self.latch = true;

                data
            }
            0x2007 => {
                let mut data = self.read_buffer;
                self.read_buffer = self.read(address);

                if address > 0x3f00 {
                    data = self.read_buffer;
                }

                self.address.0 += if self.control.vram_address_increment() {
                    32
                } else {
                    1
                };
                data
            }
            _ => unreachable!(
                "tried to read from ppu register at {:04X}",
                address
            ),
        }
    }

    pub fn write_register(&mut self, address: u16, data: u8) {
        match address {
            0x2000 => {
                self.control = Control(data);
                self.temp_address.set_nametable(self.control.nametable());
            }
            0x2001 => self.mask = Mask(data),
            0x2005 => {
                if self.latch {
                    self.fine_x_scroll = data & 0x7;
                    self.temp_address.set_coarse_x_scroll(data >> 3);
                } else {
                    self.temp_address.set_fine_y_scroll(data & 0x7);
                    self.temp_address.set_coarse_y_scroll(data >> 3);
                }

                self.latch = !self.latch;
            }
            0x2006 => {
                if self.latch {
                    self.temp_address.set_high(data & 0x3f);
                } else {
                    self.temp_address.set_low(data);
                    self.address = Address(self.temp_address.0);
                }

                self.latch = !self.latch;
            }
            0x2007 => {
                self.write(self.address.0, data);
                self.address.0 += if self.control.vram_address_increment() {
                    32
                } else {
                    1
                };
            }
            _ => (),
        }
    }

    fn fetch_tile_data(&mut self) {
        match (self.cycle) % 8 {
            1 => {
                self.reload_background_shifters();
                self.nametable_latch =
                    self.read(0x2000 | (self.address.0 & 0x0fff));
            }
            3 => {
                // Each attribute byte applies to a 4x4 group of tiles. To get the
                // attribute byte for a 4x4 group we can divide the x and y offsets
                // by four.
                let attribute_address = 0x23c0
                    | ((self.address.nametable() as u16) << 10)
                    | (((self.address.coarse_y_scroll() as u16) / 4) << 3)
                    | ((self.address.coarse_x_scroll() as u16) / 4);
                self.attribute_byte = self.read(attribute_address);

                // Each attribute byte represents a 4x4 group of tiles,
                // but we only need two bits to specify a palette. So,
                // the attribute byte actually maps four palettes to the
                // 4 2x2 tile groups in the 4x4 group. We can use the coarse coordinates
                // to figure out which 2x2 group we're on.

                if (self.address.coarse_y_scroll() & 0x02) != 0 {
                    // We're in the bottom row of the 2x2 group which
                    // is represented by the last 4 bits so shift the rest out.
                    self.attribute_byte >>= 4;
                }
                if (self.address.coarse_x_scroll() & 0x02) != 0 {
                    // We're in the right column of the 2x2 group so which
                    // is represented by the two leftmost bits.
                    self.attribute_byte >>= 2;
                }

                // Mask off the rest of the bits.
                self.attribute_byte &= 0x03;
            }
            5 => {
                self.tile_low = self.read(
                    self.control.background_pattern_table_address()
                        + ((self.nametable_latch as u16) << 4)
                        + self.address.fine_y_scroll() as u16,
                );
            }
            7 => {
                self.tile_high = self.read(
                    self.control.background_pattern_table_address()
                        + ((self.nametable_latch as u16) << 4)
                        + self.address.fine_y_scroll() as u16
                        + 8,
                )
            }
            0 => {
                if self.rendering_enabled() {
                    self.address.increment_coarse_x_scroll();
                }
            }
            _ => (),
        }
    }

    fn on_visible_scanline(&self) -> bool {
        self.scanline < 240
    }

    fn on_prerender_scanline(&self) -> bool {
        self.scanline == 261
    }

    fn on_screen(&self) -> bool {
        self.scanline < 240 && self.cycle < 256
    }

    fn start_of_vblank(&self) -> bool {
        self.scanline == 241 && self.cycle == 1
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.show_background() || self.mask.show_sprites()
    }

    fn fetch_pixel(&self) -> (u8, u8) {
        let pixel_offset = 0x8000 >> self.fine_x_scroll;
        let pixel_low =
            ((self.background_shift_low & pixel_offset) != 0) as u8;
        let pixel_high =
            ((self.background_shift_high & pixel_offset) != 0) as u8;
        let pixel = (pixel_high << 1) | pixel_low;

        let palette_offset = 0x80 >> self.fine_x_scroll;
        let palette_low =
            ((self.palette_shifter_low & palette_offset) != 0) as u8;
        let palette_high =
            ((self.palette_shifter_high & palette_offset) != 0) as u8;
        let palette = (palette_high << 1) | palette_low;

        (pixel, palette)
    }

    pub fn tick(&mut self) {
        if self.on_prerender_scanline() {
            if self.cycle == 1 {
                self.status.set_sprite_overflow(false);
                self.status.set_sprite_0_hit(false);
                self.status.set_vblank(false);
                self.nmi = false;
            }

            // The vertical scroll bits are reloaded during pixels 280 to 304,
            // if rendering is enabled.
            if self.cycle >= 280
                && self.cycle <= 304
                && self.rendering_enabled()
            {
                self.address.load_y_scroll(&self.temp_address);
            }
        }

        if self.on_visible_scanline() || self.on_prerender_scanline() {
            if (self.cycle >= 1 && self.cycle <= 256)
                || (self.cycle >= 321 && self.cycle <= 337)
            {
                self.shift_background_shifters();
                self.fetch_tile_data();
            }

            if self.cycle == 256 && self.rendering_enabled() {
                self.address.increment_y_scroll();
            }

            if self.cycle == 257 && self.rendering_enabled() {
                self.address.load_x_scroll(&self.temp_address);
            }

            if self.cycle == 339 {
                self.nametable_latch =
                    self.read(0x2000 | (self.address.0 & 0x0fff));
            }
        }

        if self.start_of_vblank() {
            self.status.set_vblank(true);
            self.nmi = self.control.nmi();
        }

        if self.rendering_enabled() && self.on_screen() {
            let (pixel, palette) = self.fetch_pixel();
            self.frame[self.scanline as usize * 256 + self.cycle as usize] =
                (pixel, palette);
        }

        self.cycle = (self.cycle + 1) % 341;
        if self.cycle == 0 {
            self.scanline = (self.scanline + 1) % 262;

            if self.on_prerender_scanline() {
                self.is_frame_ready = true;
                self.done_frame = self.frame.clone();
            }
        }
    }

    fn reload_background_shifters(&mut self) {
        self.background_shift_low =
            (self.background_shift_low & 0xff00) | self.tile_low as u16;
        self.background_shift_high =
            (self.background_shift_high & 0xff00) | self.tile_high as u16;
        self.attribute_latch_low = self.attribute_byte & 1;
        self.attribute_latch_high = (self.attribute_byte & 0x2) >> 1;
    }

    fn shift_background_shifters(&mut self) {
        self.background_shift_low <<= 1;
        self.background_shift_high <<= 1;
        self.palette_shifter_low =
            (self.palette_shifter_low << 1) | self.attribute_latch_low;
        self.palette_shifter_high =
            (self.palette_shifter_high << 1) | self.attribute_latch_high;
    }
}
