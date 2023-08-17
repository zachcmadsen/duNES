use std::fs;

use dunes::{Cpu, DuNesBus, NromCartridge};

fn run(filepath: &str) {
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
fn basics() {
    run("roms/instr_test-v5/01-basics.nes");
}

#[test]
fn implied() {
    run("roms/instr_test-v5/02-implied.nes");
}

#[test]
fn immediate() {
    run("roms/instr_test-v5/03-immediate.nes");
}

#[test]
fn zero_page() {
    run("roms/instr_test-v5/04-zero_page.nes");
}

#[test]
fn zp_xy() {
    run("roms/instr_test-v5/05-zp_xy.nes");
}

#[test]
fn absolute() {
    run("roms/instr_test-v5/06-absolute.nes");
}

#[test]
fn abs_xy() {
    run("roms/instr_test-v5/07-abs_xy.nes");
}

#[test]
fn ind_x() {
    run("roms/instr_test-v5/08-ind_x.nes");
}

#[test]
fn ind_y() {
    run("roms/instr_test-v5/09-ind_y.nes");
}

#[test]
fn branches() {
    run("roms/instr_test-v5/10-branches.nes");
}

#[test]
fn stack() {
    run("roms/instr_test-v5/11-stack.nes");
}

#[test]
fn jmp_jsr() {
    run("roms/instr_test-v5/12-jmp_jsr.nes");
}

#[test]
fn rts() {
    run("roms/instr_test-v5/13-rts.nes");
}

#[test]
fn rti() {
    run("roms/instr_test-v5/14-rti.nes");
}

#[test]
fn brk() {
    run("roms/instr_test-v5/15-brk.nes");
}

#[test]
fn special() {
    run("roms/instr_test-v5/16-special.nes");
}
