use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::NromCartridge, Ppu};

const RAM_SIZE: usize = 0x800;

/// A trait for interacting with the bus.
pub trait Bus {
    fn read(&mut self, pins: &mut Pins);
    fn write(&mut self, pins: &mut Pins);
}

/// IO pins used to communicate with the bus.
#[derive(Default)]
pub struct Pins {
    pub address: u16,
    pub data: u8,
    pub irq: bool,
    pub nmi: bool,
    pub rst: bool,
}

pub struct DuNesBus {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<NromCartridge>>,
    pub ppu: Ppu,
}

impl Bus for DuNesBus {
    fn read(&mut self, pins: &mut Pins) {
        pins.data = self.read_unclocked(pins.address);

        self.ppu.tick();
        self.ppu.tick();
        self.ppu.tick();

        pins.nmi = self.ppu.nmi;
    }

    fn write(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                self.ram[(pins.address & 0x07ff) as usize] = pins.data
            }
            0x2000..=0x2007 => {
                self.ppu.write_register(pins.address, pins.data)
            }
            0x2008..=0x3fff => (),
            0x4000..=0x4013 => (),
            0x4014 => (),
            0x4015..=0x401f => (),
            0x4020..=0xffff => self
                .cartridge
                .borrow_mut()
                .write_prg(pins.address, pins.data),
        }

        self.ppu.tick();
        self.ppu.tick();
        self.ppu.tick();

        pins.nmi = self.ppu.nmi;
    }
}

impl DuNesBus {
    pub fn new(cartridge: NromCartridge) -> DuNesBus {
        let cartridge = Rc::new(RefCell::new(cartridge));

        DuNesBus {
            ram: [0; RAM_SIZE],
            cartridge: cartridge.clone(),
            ppu: Ppu::new(cartridge),
        }
    }

    pub fn read_unclocked(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.ram[(address & 0x07ff) as usize],
            0x2000..=0x2007 => self.ppu.read_register(address),
            0x2008..=0x3fff => 0,
            0x4000..=0x401f => 0,
            0x4020..=0xffff => self.cartridge.borrow_mut().read_prg(address),
        }
    }
}
