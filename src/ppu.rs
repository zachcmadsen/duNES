use std::{cell::RefCell, rc::Rc};

use crate::NromCartridge;

/// The size of a nametable in bytes.
const NAMETABLE_SIZE: usize = 1024;
/// The size of the palette memory in bytes.
const PALETTES_SIZE: usize = 32;

pub struct Ppu {
    cartridge: Rc<RefCell<NromCartridge>>,
    nametables: [u8; 2 * NAMETABLE_SIZE],
    palettes: [u8; PALETTES_SIZE],
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<NromCartridge>>) -> Ppu {
        Ppu {
            cartridge,
            nametables: [0; 2 * NAMETABLE_SIZE],
            palettes: [0; PALETTES_SIZE],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.cartridge.borrow().read_chr(address),
            0x2000..=0x3eff => todo!(),
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
            0x0000..=0x1fff => unimplemented!(),
            0x2000..=0x3eff => todo!(),
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
}
