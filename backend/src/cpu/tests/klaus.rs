use std::fs;

use crate::{cpu, cpu::tests};

const ZERO_PAGE_ADDR: u16 = 0x000A;
const CODE_ADDR: u16 = 0x0400;
const SUCCESS_ADDR: u16 = 0x336D;

#[test]
fn functional() {
    let rom = fs::read("../roms/klaus/6502_functional_test.bin").unwrap();
    let mut emu = tests::make_emu();
    emu.cpu.bus.ram[(ZERO_PAGE_ADDR as usize)..].copy_from_slice(&rom);

    cpu::step(&mut emu);
    emu.cpu.pc = CODE_ADDR;
    let mut prev_pc = emu.cpu.pc;

    loop {
        cpu::step(&mut emu);

        if prev_pc == emu.cpu.pc {
            if emu.cpu.pc == SUCCESS_ADDR {
                break;
            }

            panic!("trapped at 0x{:04X}", emu.cpu.pc);
        }

        prev_pc = emu.cpu.pc;
    }
}
