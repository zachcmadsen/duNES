use crate::{
    apu::{self, Apu},
    bus::{self, Bus},
    cpu::{self, Cpu},
    mapper::{self, Nrom},
    ppu::{self, Ppu},
};

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
    pub fn new(rom: &[u8]) -> Emu {
        let mut bus = Bus::new();
        bus.set(0x0000..=0x1FFF, Some(read_ram), Some(write_ram));

        // TODO: Handle PPU register mirroring at 0x2008 - 0x3FFF.
        bus.set(0x2000..=0x2000, Some(ppu::read_bus), Some(ppu::write_ctrl));
        bus.set(0x2001..=0x2001, Some(ppu::read_bus), Some(ppu::write_mask));
        bus.set(0x2002..=0x2002, Some(ppu::read_status), Some(ppu::write_bus));
        bus.set(
            0x2003..=0x2003,
            Some(ppu::read_bus),
            Some(ppu::write_oam_addr),
        );
        bus.set(
            0x2004..=0x2004,
            Some(ppu::read_oam_data),
            Some(ppu::write_oam_data),
        );
        bus.set(0x2005..=0x2005, Some(ppu::read_bus), Some(ppu::write_scroll));
        bus.set(0x2006..=0x2006, Some(ppu::read_bus), Some(ppu::write_addr));
        bus.set(
            0x2007..=0x2007,
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
            ppu: Ppu::new(),
            mapper: Nrom::new(rom),
            ram: vec![0; RAM_SIZE].try_into().unwrap(),
        }
    }

    pub fn tick(&mut self) {
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
