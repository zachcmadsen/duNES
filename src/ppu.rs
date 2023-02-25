use std::{cell::RefCell, rc::Rc};

use proc_bitfield::bitfield;

use crate::{cartridge::Mirroring, NromCartridge};

/// The size of a nametable in bytes.
const NAMETABLE_SIZE: usize = 1024;
/// The size of the palette memory in bytes.
const PALETTES_SIZE: usize = 32;

bitfield! {
    struct Control(u8) {
        nametable: u8 @ 0..2,
        nametable_x: bool @ 0,
        nametable_y: bool @ 1,
        address_increment: bool @ 2,
        sprite_pattern_table: bool @ 3,
        background_pattern_table: bool @ 4,
        sprite_size: bool @ 5,
        nmi: bool @ 7,
    }
}

bitfield! {
    struct Mask(u8) {
        greyscale: bool @ 0,
        show_background_left: bool @ 1,
        show_sprites_left: bool @ 2,
        show_background: bool @ 3,
        show_sprites: bool @ 4,
        emphasize_red: bool @ 5,
        emphasize_green: bool @ 6,
        emphasize_blue: bool @ 7,
    }
}

bitfield! {
    struct Status(u8) {
        sprite_overflow: bool @ 5,
        sprite_0_hit: bool @ 6,
        vblank: bool @ 7
    }
}

bitfield! {
    struct Address(u16) {
        coarse_x_scroll: u8 @ 0..5,
        coarse_y_scroll: u8 @ 5..10,
        nametable: u8 @ 10..12,
        nametable_x: bool @ 10,
        nametable_y: bool @ 11,
        fine_y_scroll: u8 @ 12..15,

        low: u8 @ 0..8,
        high: u8 @ 8..14,
    }
}

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
    scanline: i16,
    pub nmi: bool,
    pub is_frame_done: bool,

    nametable_byte: u8,
    attribute_byte: u8,
    tile_low: u8,
    tile_high: u8,

    background_shifter_low: u16,
    background_shifter_high: u16,
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
            is_frame_done: false,
            nametable_byte: 0,
            attribute_byte: 0,
            tile_low: 0,
            tile_high: 0,
            background_shifter_low: 0,
            background_shifter_high: 0,
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

                self.address.0 += if self.control.address_increment() {
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
                self.temp_address
                    .set_nametable_x(self.control.nametable_x());
                self.temp_address
                    .set_nametable_y(self.control.nametable_y());
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
                self.address.0 += if self.control.address_increment() {
                    32
                } else {
                    1
                };
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.set_vblank(false);
            }

            if (self.cycle >= 2 && self.cycle < 258)
                || (self.cycle >= 321 && self.cycle < 338)
            {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.nametable_byte =
                            self.read(0x2000 | (self.address.0 & 0x0fff));
                    }
                    2 => {
                        self.attribute_byte = self.read(
                            0x23c0
                                | ((self.address.nametable_y() as u16) << 11)
                                | ((self.address.nametable_x() as u16) << 10)
                                | (((self.address.coarse_y_scroll() as u16)
                                    >> 2)
                                    << 3)
                                | ((self.address.coarse_x_scroll() as u16)
                                    >> 2),
                        );
                        if (self.address.coarse_y_scroll() & 0x02) != 0 {
                            self.attribute_byte >>= 4;
                        }
                        if (self.address.coarse_x_scroll() & 0x02) != 0 {
                            self.attribute_byte >>= 2;
                        }
                        self.attribute_byte &= 0x03;
                    }
                    4 => {
                        self.tile_low = self.read(
                            ((self.control.background_pattern_table() as u16)
                                << 12)
                                + ((self.nametable_byte as u16) << 4)
                                + self.address.fine_y_scroll() as u16,
                        );
                    }
                    6 => {
                        self.tile_high = self.read(
                            ((self.control.background_pattern_table() as u16)
                                << 12)
                                + ((self.nametable_byte as u16) << 4)
                                + self.address.fine_y_scroll() as u16
                                + 8,
                        )
                    }
                    7 => {
                        self.increment_scroll_x();
                    }
                    _ => (),
                }
            }

            if self.cycle == 256 {
                self.increment_scroll_y();
            }

            if self.cycle == 257 {
                self.load_background_shifters();
                self.transfer_x_address();
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.nametable_byte =
                    self.read(0x2000 | (self.address.0 & 0x0fff));
            }

            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.transfer_y_address();
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set_vblank(true);
            self.nmi = self.control.nmi();
        }

        let mut pixel_value = 0;
        let mut palette = 0;

        if self.mask.show_background() {
            let selected_bit = 0x8000 >> self.fine_x_scroll;

            let pixel_low: u8 =
                if (self.background_shifter_low & selected_bit) > 0 {
                    1
                } else {
                    0
                };
            let pixel_high =
                if (self.background_shifter_high & selected_bit) > 0 {
                    1
                } else {
                    0
                };
            pixel_value = (pixel_high << 1) | pixel_low;

            let pal_selected_bit = 0x80 >> self.fine_x_scroll;
            let pal_low = if (self.palette_shifter_low & pal_selected_bit) > 0
            {
                1
            } else {
                0
            };
            let pal_high =
                if (self.palette_shifter_high & pal_selected_bit) > 0 {
                    1
                } else {
                    0
                };
            palette = (pal_high << 1) | pal_low;
        }

        if self.scanline != -1
            && (self.scanline as usize * 256 + (self.cycle as usize))
                < (256 * 240)
        {
            self.frame[self.scanline as usize * 256 + self.cycle as usize] =
                (pixel_value, palette);
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                self.is_frame_done = true;
                self.done_frame = self.frame.clone();
            }
        }
    }

    fn load_background_shifters(&mut self) {
        self.background_shifter_low =
            (self.background_shifter_low & 0xff00) | self.tile_low as u16;
        self.background_shifter_high =
            (self.background_shifter_high & 0xff00) | self.tile_high as u16;
        self.attribute_latch_low = self.attribute_byte & 1;
        self.attribute_latch_high = (self.attribute_byte & 0x2) >> 1;
    }

    fn update_shifters(&mut self) {
        if self.mask.show_background() {
            self.background_shifter_low <<= 1;
            self.background_shifter_high <<= 1;
            self.palette_shifter_low =
                (self.palette_shifter_low << 1) | self.attribute_latch_low;
            self.palette_shifter_high =
                (self.palette_shifter_high << 1) | self.attribute_latch_high;
        }
    }

    fn increment_scroll_x(&mut self) {
        if self.mask.show_background() {
            if self.address.coarse_x_scroll() == 31 {
                self.address.set_coarse_x_scroll(0);
                self.address.set_nametable_x(!self.address.nametable_x());
            } else {
                self.address
                    .set_coarse_x_scroll(self.address.coarse_x_scroll() + 1);
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.mask.show_background() {
            if self.address.fine_y_scroll() < 7 {
                self.address
                    .set_fine_y_scroll(self.address.fine_y_scroll() + 1);
            } else {
                self.address.set_fine_y_scroll(0);

                if self.address.coarse_y_scroll() == 29 {
                    self.address.set_coarse_y_scroll(0);
                    self.address.set_nametable_y(!self.address.nametable_y());
                } else if self.address.coarse_y_scroll() == 31 {
                    self.address.set_coarse_y_scroll(0);
                } else {
                    self.address.set_coarse_y_scroll(
                        self.address.coarse_y_scroll() + 1,
                    );
                }
            }
        }
    }

    fn transfer_x_address(&mut self) {
        if self.mask.show_background() {
            self.address
                .set_nametable_x(self.temp_address.nametable_x());
            self.address
                .set_coarse_x_scroll(self.temp_address.coarse_x_scroll());
        }
    }

    fn transfer_y_address(&mut self) {
        if self.mask.show_background() {
            self.address
                .set_fine_y_scroll(self.temp_address.fine_y_scroll());
            self.address
                .set_nametable_y(self.temp_address.nametable_y());
            self.address
                .set_coarse_y_scroll(self.temp_address.coarse_y_scroll());
        }
    }
}
