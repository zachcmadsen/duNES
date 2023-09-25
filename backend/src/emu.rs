use crate::{
    bus::Bus,
    cpu::{self, Cpu},
};

/// The size of the CPU address space in bytes;
const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

pub struct Emu {
    pub bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub cpu: Cpu,
}

impl Emu {
    pub fn new() -> Emu {
        Emu { bus: Bus::new(), cpu: Cpu::new() }
    }

    pub fn step(&mut self) {
        cpu::step(self);
    }
}
