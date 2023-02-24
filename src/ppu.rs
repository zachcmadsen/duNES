use std::{cell::RefCell, rc::Rc};

use proc_bitfield::bitfield;

use crate::NromCartridge;

/// The size of a nametable in bytes.
const NAMETABLE_SIZE: usize = 1024;
/// The size of the palette memory in bytes.
const PALETTES_SIZE: usize = 32;

bitfield! {
    struct Control(u8) {
        nametable: u8 @ 0..2,
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
        low: u8 @ 0..8,
        high: u8 @ 8..14,
    }
}

pub struct Ppu {
    cartridge: Rc<RefCell<NromCartridge>>,
    nametables: [u8; 2 * NAMETABLE_SIZE],
    palettes: [u8; PALETTES_SIZE],

    control: Control,
    mask: Mask,
    status: Status,
    latch: bool,
    address: Address,

    read_buffer: u8,

    cycle: u16,
    scanline: i16,
    pub nmi: bool,
    pub is_frame_done: bool,
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
            read_buffer: 0,
            cycle: 0,
            scanline: 0,
            nmi: false,
            is_frame_done: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.cartridge.borrow().read_chr(address),
            0x2000..=0x3eff => 0,
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
            0x2000..=0x3eff => (),
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

                self.address.0 += 1;
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
            0x2000 => self.control = Control(data),
            0x2001 => self.mask = Mask(data),
            0x2006 => {
                if self.latch {
                    self.address.set_high(data);
                } else {
                    self.address.set_low(data);
                }

                self.latch = !self.latch;
            }
            0x2007 => {
                self.write(self.address.0, data);
                self.address.0 += 1;
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        if self.scanline == -1 && self.cycle == 1 {
            self.status.set_vblank(false);
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set_vblank(true);
            self.nmi = self.control.nmi();
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                self.is_frame_done = true;
            }
        }
    }
}
