use std::{cell::RefCell, rc::Rc};

use crate::cartridge::NromCartridge;

const RAM_SIZE: usize = 0x800;

#[derive(Default)]
pub struct Pins {
    pub address: u16,
    pub data: u8,
    pub irq: bool,
    pub nmi: bool,
    pub rst: bool,
}

pub struct Bus {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<NromCartridge>>,
}

impl Bus {
    pub fn new(cartridge: NromCartridge) -> Bus {
        Bus {
            ram: [0; RAM_SIZE],
            cartridge: Rc::new(RefCell::new(cartridge)),
        }
    }

    pub fn hidden_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.ram[(address & 0x07ff) as usize],
            0x2000..=0x3fff => 0,
            0x4000..=0x401f => 0,
            0x4020..=0xffff => self.cartridge.borrow_mut().read_prg(address),
        }
    }

    pub fn read(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                pins.data = self.ram[(pins.address & 0x07ff) as usize]
            }
            0x2000..=0x3fff => (),
            0x4000..=0x401f => todo!(),
            0x4020..=0xffff => {
                pins.data = self.cartridge.borrow_mut().read_prg(pins.address)
            }
        }
    }

    pub fn write(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                self.ram[(pins.address & 0x07ff) as usize] = pins.data
            }
            0x2000..=0x3fff => todo!(),
            0x4000..=0x4013 => todo!(),
            0x4014 => todo!(),
            0x4015..=0x401f => todo!(),
            0x4020..=0xffff => self
                .cartridge
                .borrow_mut()
                .write_prg(pins.address, pins.data),
        }
    }
}
