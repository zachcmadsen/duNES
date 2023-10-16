use std::fs;

use backend::{read_byte, Emu};

const STATUS_ADDR: u16 = 0x6000;
const OUTPUT_ADDR: u16 = 0x6004;
const RUNNING_STATUS: u8 = 0x80;

fn run(filename: &str) {
    let rom = fs::read(format!("../roms/instr_test-v5/{}", filename)).unwrap();
    let mut emu = Emu::new(&rom);

    let mut status = read_byte(&mut emu, STATUS_ADDR);
    while status != RUNNING_STATUS {
        emu.step();
        status = read_byte(&mut emu, STATUS_ADDR);
    }

    while status == RUNNING_STATUS {
        emu.step();
        status = read_byte(&mut emu, STATUS_ADDR);
    }

    let mut output = Vec::new();
    let mut addr = OUTPUT_ADDR;
    let mut byte = read_byte(&mut emu, addr);
    while byte != b'\0' {
        output.push(byte);
        addr += 1;
        byte = read_byte(&mut emu, addr);
    }

    assert!(String::from_utf8_lossy(&output).contains("Passed"));
}

#[test]
fn basics() {
    run("01-basics.nes");
}

#[test]
fn implied() {
    run("02-implied.nes");
}

#[test]
fn immediate() {
    run("03-immediate.nes");
}

#[test]
fn zero_page() {
    run("04-zero_page.nes");
}

#[test]
fn zp_xy() {
    run("05-zp_xy.nes");
}

#[test]
fn absolute() {
    run("06-absolute.nes");
}

#[test]
fn abs_xy() {
    run("07-abs_xy.nes");
}

#[test]
fn ind_x() {
    run("08-ind_x.nes");
}

#[test]
fn ind_y() {
    run("09-ind_y.nes");
}

#[test]
fn branches() {
    run("10-branches.nes");
}

#[test]
fn stack() {
    run("11-stack.nes");
}

#[test]
fn jmp_jsr() {
    run("12-jmp_jsr.nes");
}

#[test]
fn rts() {
    run("13-rts.nes");
}

#[test]
fn rti() {
    run("14-rti.nes");
}

#[test]
fn brk() {
    run("15-brk.nes");
}

#[test]
fn special() {
    run("16-special.nes");
}
