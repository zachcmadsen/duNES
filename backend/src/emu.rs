use std::mem::MaybeUninit;

use crate::{
    apu::{self, Apu},
    cpu::{self, Cpu},
    nrom::Nrom,
    scheduler::{self, EventKind, Scheduler},
};

pub struct Emu {
    pub(crate) cpu: Cpu,
    pub(crate) nrom: Nrom,
    pub(crate) scheduler: Scheduler,
    pub(crate) apu: Apu,
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut emu = Emu {
            cpu: Cpu::new(),
            nrom: Nrom::new(rom),
            scheduler: Scheduler::new(),
            apu: Apu::new(),
        };

        scheduler::queue(&mut emu, EventKind::Reset, 0);

        emu
    }

    pub fn step(&mut self) {
        scheduler::handle_events(self);
        cpu::step(self);
    }

    pub fn peek(&mut self, addr: u16) -> Option<u8> {
        cpu::peek(self, addr)
    }

    pub fn fill(&mut self, dst: &mut [MaybeUninit<i16>]) {
        apu::fill(self, dst);
    }
}
