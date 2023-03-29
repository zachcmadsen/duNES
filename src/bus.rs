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
    pub oam_dma: Option<u8>,
}

pub struct DuNesBus {
    ram: Box<[u8; RAM_SIZE]>,
    cartridge: Rc<RefCell<NromCartridge>>,
    pub ppu: Ppu,

    pub controller: u8,
    pub controller_state: u8,
}

impl Bus for DuNesBus {
    fn read(&mut self, pins: &mut Pins) {
        pins.data = match pins.address {
            0x0000..=0x1fff => self.ram[(pins.address & 0x07ff) as usize],
            0x2000..=0x2007 => self.ppu.read_register(pins.address),
            0x2008..=0x3fff => 0,
            0x4016 => {
                let data = ((self.controller_state & 0x80) != 0) as u8;
                self.controller_state <<= 1;
                data
            }
            0x4000..=0x401f => 0,
            0x4020..=0xffff => {
                self.cartridge.borrow_mut().read_prg(pins.address)
            }
        };

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
            0x4014 => pins.oam_dma = Some(pins.data),
            0x4016 => {
                self.controller_state = self.controller;
            }
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
            ram: vec![0; RAM_SIZE].try_into().unwrap(),
            cartridge: cartridge.clone(),
            ppu: Ppu::new(cartridge),
            controller: 0,
            controller_state: 0,
        }
    }

    pub fn read_unclocked(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.ram[(address & 0x07ff) as usize],
            0x4020..=0xffff => self.cartridge.borrow_mut().read_prg(address),
            _ => unimplemented!(),
        }
    }
}
