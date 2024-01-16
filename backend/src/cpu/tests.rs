mod bus;
mod klaus;
mod processor;

use crate::{apu::Apu, cpu::Cpu, nrom::Nrom, scheduler::Scheduler, Emu};

fn make_emu() -> Emu {
    Emu {
        cpu: Cpu::new(),
        nrom: Nrom { prg_ram: Box::new([]), prg_rom: Box::new([]) },
        scheduler: Scheduler::new(),
        apu: Apu::new(),
    }
}
