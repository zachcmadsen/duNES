use common::Writer;

use crate::{
    apu::{self, Apu},
    bus::{self, Bus},
    cpu::{self, Cpu},
    mapper::{self, Nrom},
    ppu::{self, Ppu},
};

/// The width of the screen in pixels.
pub const WIDTH: u32 = 256;
/// The height of the screen in pixels.
pub const HEIGHT: u32 = 240;
/// The size of the framebuffer in bytes.
pub const FRAMEBUFFER_SIZE: usize = 4 * WIDTH as usize * HEIGHT as usize;

/// The size of RAM in bytes.
pub(crate) const RAM_SIZE: usize = 2048;

pub struct Emu {
    pub apu: Apu,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub(crate) mapper: Nrom,
    pub(crate) ram: Box<[u8; RAM_SIZE]>,
}

impl Emu {
    pub fn new(rom: &[u8], writer: Writer<[u8; FRAMEBUFFER_SIZE]>) -> Emu {
        let mut bus = Bus::new();
        bus.set(0x0000..=0x1FFF, Some(read_ram), Some(write_ram));
        bus.set(
            0x2000..=0x2007,
            Some(ppu::read_register),
            Some(ppu::write_register),
        );
        bus.set(0x4000..=0x4014, None, Some(apu::write));
        bus.set(0x4015..=0x4015, Some(apu::read), Some(apu::write));
        bus.set(0x4017..=0x4017, None, Some(apu::write));
        bus.set(
            0x6000..=0x7FFF,
            Some(mapper::read_prg_ram),
            Some(mapper::write_prg_ram),
        );
        bus.set(
            0x8000..=0xFFFF,
            Some(mapper::read_prg_rom),
            Some(mapper::write_prg_rom),
        );

        Emu {
            apu: Apu::new(),
            cpu: Cpu::new(bus),
            ppu: Ppu::new(writer),
            mapper: Nrom::new(rom),
            ram: vec![0; RAM_SIZE].try_into().unwrap(),
        }
    }

    pub fn step(&mut self) {
        cpu::tick(self);
        ppu::tick(self);
        ppu::tick(self);
        ppu::tick(self);
        self.apu.tick();
    }

    // TODO: Find a better solution for peeking memory. Some read handlers have
    // side effects.
    pub fn read(&mut self, addr: u16) -> u8 {
        bus::read_byte(self, addr)
    }
}

fn read_ram(emu: &mut Emu, addr: u16) -> u8 {
    emu.ram[(addr & 0x07FF) as usize]
}

fn write_ram(emu: &mut Emu, addr: u16, data: u8) {
    emu.ram[(addr & 0x07FF) as usize] = data;
}
