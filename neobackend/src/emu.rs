use crate::scheduler::{EventKind, Scheduler};
use crate::{cpu, cpu::Cpu, nrom::Nrom, scheduler};

pub struct Emu {
    pub(crate) cpu: Cpu,
    pub(crate) nrom: Nrom,
    pub(crate) scheduler: Scheduler,
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut emu = Emu {
            cpu: Cpu::new(),
            nrom: Nrom::new(rom),
            scheduler: Scheduler::new(),
        };

        scheduler::queue(&mut emu, EventKind::Reset, 0);

        emu
    }

    pub fn step(&mut self) {
        if scheduler::ready(self) {
            scheduler::handle(self);
        }

        cpu::step(self);
    }
}
