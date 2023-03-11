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
/// The base address of the nametable memory.
const BASE_NAMETABLE_ADDRESS: u16 = 0x2000;
/// The base address of the attribute tables.
const BASE_ATTRIBUTE_ADDRESS: u16 = 0x23c0;
/// The base address of the palette memory.
const BASE_PALETTE_ADDRESS: u16 = 0x3f00;
/// The size of a tile in bytes.
const TILE_SIZE: usize = 16;
/// The size of one plane of a tile in bytes.
const TILE_PLANE_SIZE: usize = 8;

pub struct Ppu {
    // Data
    cartridge: Rc<RefCell<NromCartridge>>,
    nametables: [u8; 2 * NAMETABLE_SIZE],
    palettes: [u8; PALETTES_SIZE],
    read_buffer: u8,

    // Registers
    control: Control,
    mask: Mask,
    status: Status,

    // Scrolling
    v: Address,
    t: Address,
    fine_x_scroll: u8,
    w: bool,

    // State
    cycle: u16,
    scanline: u16,
    pub nmi: bool,

    // Background latches
    nametable_latch: u8,
    attribute_bits: u8,
    attribute_latch: u8,
    tile_latch_low: u8,
    tile_latch_high: u8,

    // Background shifters
    tile_shifter_low: u16,
    tile_shifter_high: u16,
    attribute_shifter_low: u8,
    attribute_shifter_high: u8,

    scratch_frame: Vec<(u8, u8)>,
    pub frame: Vec<(u8, u8)>,
    pub is_frame_ready: bool,
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<NromCartridge>>) -> Ppu {
        Ppu {
            cartridge,
            nametables: [0; 2 * NAMETABLE_SIZE],
            palettes: [0; PALETTES_SIZE],
            read_buffer: 0,

            control: Control(0),
            mask: Mask(0),
            status: Status(0),

            v: Address(0),
            t: Address(0),
            fine_x_scroll: 0,
            w: false,

            cycle: 0,
            scanline: 0,
            nmi: false,

            nametable_latch: 0,
            attribute_bits: 0,
            attribute_latch: 0,
            tile_latch_low: 0,
            tile_latch_high: 0,

            tile_shifter_low: 0,
            tile_shifter_high: 0,
            attribute_shifter_low: 0,
            attribute_shifter_high: 0,

            scratch_frame: vec![(0, 0); 240 * 256],
            frame: vec![(0, 0); 240 * 256],
            is_frame_ready: false,
        }
    }

    pub fn read_data(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.cartridge.borrow().read_chr(address),
            0x2000..=0x3eff => {
                let mirrored_address = self.mirror_nametable_address(address);
                self.nametables
                    [(mirrored_address - BASE_NAMETABLE_ADDRESS) as usize]
            }
            0x3f00..=0x3fff => {
                let mirrored_address = self.mirror_palette_address(address);
                self.palettes
                    [(mirrored_address - BASE_PALETTE_ADDRESS) as usize]
            }
            _ => unreachable!(),
        }
    }

    fn write_data(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => unimplemented!(),
            0x2000..=0x3eff => {
                let mirrored_address = self.mirror_nametable_address(address);
                self.nametables
                    [(mirrored_address - BASE_NAMETABLE_ADDRESS) as usize] =
                    data;
            }
            0x3f00..=0x3fff => {
                let mirrored_address = self.mirror_palette_address(address);
                self.palettes
                    [(mirrored_address - BASE_PALETTE_ADDRESS) as usize] =
                    data;
            }
            _ => unreachable!(),
        }
    }

    pub fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => {
                // TODO: Using the read buffer here isn't accurate. It should
                // be the data bus, which isn't emulated yet.
                let data = self.status.0 | (self.read_buffer & 0x1f);
                // TODO: Should self.nmi be set to false? Also, reading the
                // status register within two cycles of vblank causes an NMI
                // not to occur?
                self.status.set_vblank(false);
                self.w = false;

                data
            }
            0x2007 => {
                let data = if address < BASE_PALETTE_ADDRESS {
                    self.read_buffer
                } else {
                    self.read_data(address)
                };

                // TODO: The data in the buffer should be different when
                // reading palette data.
                self.read_buffer = self.read_data(address);

                self.v.increment(self.control.increment_mode());

                data
            }
            _ => unimplemented!(),
        }
    }

    pub fn write_register(&mut self, address: u16, data: u8) {
        match address {
            0x2000 => {
                self.control = Control(data);
                self.t.set_nametable(self.control.nametable());
            }
            0x2001 => self.mask = Mask(data),
            0x2005 => {
                if !self.w {
                    self.fine_x_scroll = data & 0x7;
                    self.t.set_coarse_x_scroll(data >> 3);
                } else {
                    self.t.set_fine_y_scroll(data & 0x7);
                    self.t.set_coarse_y_scroll(data >> 3);
                }

                self.w = !self.w;
            }
            0x2006 => {
                if !self.w {
                    self.t.set_high(data & 0x3f);
                } else {
                    self.t.set_low(data);
                    self.v = Address(self.t.0);
                }

                self.w = !self.w;
            }
            0x2007 => {
                // TODO: Should self.v.0 be mirrored down?
                self.write_data(self.v.0, data);
                self.v.increment(self.control.increment_mode());
            }
            _ => (),
        }
    }

    fn mirror_nametable_address(&self, address: u16) -> u16 {
        match self.cartridge.borrow().mirroring() {
            // The second and fourth nametables map to the first and second
            // nametables, respectively.
            Mirroring::Horizontal => match address {
                0x2400..=0x27ff => address - 0x0400,
                // We have to map the third and fourth logical nametables to
                // the second physical nametable.
                0x2800..=0x2bff => address - 0x0400,
                0x2c00..=0x2fff => address - 0x0800,
                _ => address,
            },
            // The third and fourth nametables map to the first and second
            // nametables, respectively.
            Mirroring::Vertical => match address {
                0x2800..=0x2bff | 0x2c00..=0x2fff => address - 0x0800,
                _ => address,
            },
        }
    }

    fn mirror_palette_address(&self, address: u16) -> u16 {
        // 0x3f20 - 0x3fff is a mirror of 0x3f00 - 0x3f1f.
        let address = address & 0x3f1f;
        match address {
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => address - 0x10,
            _ => address,
        }
    }

    fn fetch_nametable_byte(&self) -> u8 {
        // The bottom 12 bits of v are the address of the next tile in
        // nametable memory.
        self.read_data(BASE_NAMETABLE_ADDRESS | (self.v.0 & 0x0fff))
    }

    fn fetch_attribute_bits(&self) -> u8 {
        // Divide the coarse offsets by four since each attribute byte applies
        // to a 4x4 group of tiles.
        let attribute_address = BASE_ATTRIBUTE_ADDRESS
            | ((self.v.nametable() as u16) << 10)
            | (((self.v.coarse_y_scroll() as u16) / 4) << 3)
            | ((self.v.coarse_x_scroll() as u16) / 4);
        let mut attribute_byte = self.read_data(attribute_address);

        // The attribute byte is divided into four 2-bit areas that specify the
        // palettes for four 2x2 tile groups.
        //
        // The bottom row of a 2x2 group is covered by the four leftmost
        // bits.
        attribute_byte >>= ((self.v.coarse_y_scroll() & 0x02) != 0) as u8 * 4;
        // The right column of a 2x2 group is covered by the two leftmost
        // bits.
        attribute_byte >>= ((self.v.coarse_x_scroll() & 0x02) != 0) as u8 * 2;

        // Mask off the rest of the bits.
        attribute_byte & 0x03
    }

    fn background_pattern_table_address(&self) -> u16 {
        if self.control.background_pattern_table() {
            0x1000
        } else {
            0x0000
        }
    }

    fn fetch_tile_data(&mut self) {
        match (self.cycle) % 8 {
            1 => {
                self.reload_background_shifters();
                self.nametable_latch = self.fetch_nametable_byte();
            }
            3 => {
                self.attribute_bits = self.fetch_attribute_bits();
            }
            5 => {
                self.tile_latch_low = self.read_data(
                    self.background_pattern_table_address()
                        + (self.nametable_latch as u16 * TILE_SIZE as u16)
                        + self.v.fine_y_scroll() as u16,
                );
            }
            7 => {
                self.tile_latch_high = self.read_data(
                    self.background_pattern_table_address()
                        + ((self.nametable_latch as u16) << 4)
                        + self.v.fine_y_scroll() as u16
                        + TILE_PLANE_SIZE as u16,
                )
            }
            0 => {
                if self.rendering_enabled() {
                    self.v.increment_coarse_x_scroll();
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
        let pixel_low = ((self.tile_shifter_low & pixel_offset) != 0) as u8;
        let pixel_high = ((self.tile_shifter_high & pixel_offset) != 0) as u8;
        let pixel = (pixel_high << 1) | pixel_low;

        let palette_offset = 0x80 >> self.fine_x_scroll;
        let palette_low =
            ((self.attribute_shifter_low & palette_offset) != 0) as u8;
        let palette_high =
            ((self.attribute_shifter_high & palette_offset) != 0) as u8;
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
                self.v.load_y_scroll(&self.t);
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
                self.v.increment_y_scroll();
            }

            if self.cycle == 257 && self.rendering_enabled() {
                self.v.load_x_scroll(&self.t);
            }

            if self.cycle == 339 {
                self.nametable_latch = self.fetch_nametable_byte();
            }
        }

        if self.start_of_vblank() {
            self.status.set_vblank(true);
            self.nmi = self.control.nmi();
        }

        if self.rendering_enabled() && self.on_screen() {
            let (pixel, palette) = self.fetch_pixel();
            self.scratch_frame
                [self.scanline as usize * 256 + self.cycle as usize] =
                (pixel, palette);
        }

        self.cycle = (self.cycle + 1) % 341;
        if self.cycle == 0 {
            self.scanline = (self.scanline + 1) % 262;

            if self.on_prerender_scanline() {
                self.is_frame_ready = true;
                self.frame.copy_from_slice(&self.scratch_frame);
            }
        }
    }

    fn reload_background_shifters(&mut self) {
        self.tile_shifter_low =
            (self.tile_shifter_low & 0xff00) | self.tile_latch_low as u16;
        self.tile_shifter_high =
            (self.tile_shifter_high & 0xff00) | self.tile_latch_high as u16;
        self.attribute_latch = self.attribute_bits;
    }

    fn shift_background_shifters(&mut self) {
        self.tile_shifter_low <<= 1;
        self.tile_shifter_high <<= 1;
        self.attribute_shifter_low =
            (self.attribute_shifter_low << 1) | (self.attribute_latch & 1);
        self.attribute_shifter_high = (self.attribute_shifter_high << 1)
            | ((self.attribute_latch & 0x2) >> 1);
    }
}
