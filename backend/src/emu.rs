use crate::{
    bus::{self, Bus},
    cpu::{self, Cpu, CPU_ADDR_SPACE_SIZE},
    mapper::{self, Nrom},
};

pub struct Emu {
    pub(crate) bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub(crate) cpu: Cpu,
    pub(crate) mapper: Nrom,
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut bus = Bus::new();
        cpu::register(&mut bus);
        mapper::register(&mut bus);

        Emu { bus, cpu: Cpu::new(), mapper: Nrom::new(rom) }
    }

    pub fn step(&mut self) {
        cpu::step(self);
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        bus::read_byte(self, addr)
    }
}
