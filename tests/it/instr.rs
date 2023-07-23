use std::fs;

use dunes::{Cpu, DuNesBus, NromCartridge};

fn run_instr(filepath: &str) {
    const STATUS: u16 = 0x6000;
    const RUNNING: u8 = 0x80;
    const OUTPUT: u16 = 0x6004;

    let rom = fs::read(filepath)
        .unwrap_or_else(|_| panic!("{filepath} should exist"));
    let cartridge = NromCartridge::new(&rom);
    let bus = DuNesBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    cpu.step();
    let mut status = cpu.bus.read_unclocked(STATUS);
    while status != RUNNING {
        cpu.step();
        status = cpu.bus.read_unclocked(STATUS);
    }

    while status == RUNNING {
        cpu.step();
        status = cpu.bus.read_unclocked(STATUS);
    }

    let mut output = Vec::new();
    let mut addr = OUTPUT;
    let mut byte = cpu.bus.read_unclocked(addr);
    while byte != b'\0' {
        output.push(byte);
        addr += 1;
        byte = cpu.bus.read_unclocked(addr);
    }

    assert!(String::from_utf8_lossy(&output).contains("Passed"));
}

#[test]
fn instr_basics() {
    run_instr("tests/roms/01-basics.nes");
}

#[test]
fn implied() {
    run_instr("tests/roms/02-implied.nes");
}

#[test]
fn immediate() {
    run_instr("tests/roms/03-immediate.nes");
}

#[test]
fn zero_page() {
    run_instr("tests/roms/04-zero_page.nes");
}

#[test]
fn zp_xy() {
    run_instr("tests/roms/05-zp_xy.nes");
}

#[test]
fn absolute() {
    run_instr("tests/roms/06-absolute.nes");
}

#[test]
fn abs_xy() {
    run_instr("tests/roms/07-abs_xy.nes");
}

#[test]
fn ind_x() {
    run_instr("tests/roms/08-ind_x.nes");
}

#[test]
fn ind_y() {
    run_instr("tests/roms/09-ind_y.nes");
}

#[test]
fn branches() {
    run_instr("tests/roms/10-branches.nes");
}

#[test]
fn stack() {
    run_instr("tests/roms/11-stack.nes");
}

#[test]
fn jmp_jsr() {
    run_instr("tests/roms/12-jmp_jsr.nes");
}

#[test]
fn rts() {
    run_instr("tests/roms/13-rts.nes");
}

#[test]
fn rti() {
    run_instr("tests/roms/14-rti.nes");
}

#[test]
fn brk() {
    run_instr("tests/roms/15-brk.nes");
}

#[test]
fn special() {
    run_instr("tests/roms/16-special.nes");
}
