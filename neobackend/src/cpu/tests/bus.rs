#![allow(dead_code)]

use crate::emu::Emu;

/// The size of the CPU's internal ram in bytes.
const ADDR_SPACE_SIZE: u32 = 0x10000;

pub struct Bus {
    // The CPU tests assume 64 KB of RAM.
    pub(super) ram: Box<[u8; ADDR_SPACE_SIZE as usize]>,
    pub(super) cycles: Vec<(u16, u8, &'static str)>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: vec![0; ADDR_SPACE_SIZE as usize].try_into().unwrap(),
            cycles: vec![],
        }
    }
}

pub fn read(emu: &mut Emu, addr: u16) -> u8 {
    let data = emu.cpu.bus.ram[addr as usize];
    emu.cpu.bus.cycles.push((addr, data, "read"));
    data
}

pub fn write(emu: &mut Emu, addr: u16, data: u8) {
    emu.cpu.bus.ram[addr as usize] = data;
    emu.cpu.bus.cycles.push((addr, data, "write"));
}
