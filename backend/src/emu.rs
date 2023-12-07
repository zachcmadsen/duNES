use common::Writer;

use crate::{
    apu::{self, Apu},
    bus::{self, Bus},
    cpu::{self, Cpu},
    mapper::{self, Nrom},
    ppu::{self, Ppu},
};

/// The size of the CPU RAM in bytes.
pub(crate) const CPU_RAM_SIZE: usize = 2048;
/// The size of the CPU address space in bytes;
pub(crate) const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

pub struct Emu {
    pub(crate) bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub apu: Apu,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub(crate) mapper: Nrom,

    pub(crate) ram: Box<[u8; CPU_RAM_SIZE]>,
}

impl Emu {
    pub fn new(
        rom: &[u8],
        buffer: Writer<Box<[u8; ppu::BUFFER_SIZE]>>,
    ) -> Emu {
        let mut bus = Bus::new();
        bus.set(0x0000..=0x1FFF, Some(read_ram), Some(write_ram));
        bus.set(
            0x2000..=0x2007,
            Some(Ppu::read_register),
            Some(Ppu::write_register),
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
            bus,
            cpu: Cpu::new(),
            ppu: Ppu::new(buffer),
            mapper: Nrom::new(rom),

            ram: vec![0; CPU_RAM_SIZE].try_into().unwrap(),
        }
    }

    pub fn step(&mut self) {
        cpu::step(self);
        // TODO: Update emu.cpu.nmi directly in the PPU?
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
        self.apu.tick();
    }

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
