use std::fs;

use dunes::{Bus, Cpu, DuNesBus, NromCartridge};

const ZERO_PAGE_START: usize = 0xa;
const CODE_SEGMENT_START: u16 = 0x400;
const INTERRUPT_FEEDBACK_REGISTER: u16 = 0xbffc;
const IRQ_MASK: u8 = 0x1;
const NMI_MASK: u8 = 0x2;
const FUNCTIONAL_SUCCESS: u16 = 0x336d;
const INTERRUPT_SUCCESS: u16 = 0x6f5;

const STATUS: u16 = 0x6000;
const RUNNING: u8 = 0x80;
const OUTPUT: u16 = 0x6004;

struct KlausBus {
    memory: [u8; 0x10000],
}

impl KlausBus {
    fn new(rom: &[u8]) -> KlausBus {
        let mut memory = [0; 0x10000];
        memory[ZERO_PAGE_START..].copy_from_slice(rom);

        KlausBus { memory }
    }
}

impl Bus for KlausBus {
    fn read(&mut self, pins: &mut dunes::Pins) {
        pins.data = self.memory[pins.address as usize];
    }

    fn write(&mut self, pins: &mut dunes::Pins) {
        if pins.address == INTERRUPT_FEEDBACK_REGISTER {
            let old_data = self.memory[pins.address as usize];
            let prev_nmi = old_data & NMI_MASK != 0;
            let new_nmi = pins.data & NMI_MASK != 0;

            pins.irq = pins.data & IRQ_MASK != 0;
            pins.nmi = !prev_nmi && new_nmi;
        }

        self.memory[pins.address as usize] = pins.data;
    }
}

fn run_klaus(filepath: &str, success_address: u16) {
    let rom = fs::read(filepath)
        .unwrap_or_else(|_| panic!("{filepath} should exist"));
    let mut cpu = Cpu::new(KlausBus::new(&rom));

    cpu.step();
    cpu.pc = CODE_SEGMENT_START;
    let mut prev_pc = cpu.pc;

    loop {
        cpu.step();

        if prev_pc == cpu.pc {
            if cpu.pc == success_address {
                break;
            }

            panic!("trapped at 0x{:04X}", cpu.pc);
        }

        prev_pc = cpu.pc;
    }
}

fn run_instr(filepath: &str) {
    let rom = fs::read(filepath)
        .unwrap_or_else(|_| panic!("{} should exist", filepath));
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
    let mut address = OUTPUT;
    let mut byte = cpu.bus.read_unclocked(address);
    while byte != b'\0' {
        output.push(byte);
        address += 1;
        byte = cpu.bus.read_unclocked(address);
    }

    assert!(String::from_utf8_lossy(&output).contains("Passed"));
}

#[test]
fn klaus_functional() {
    run_klaus("tests/roms/6502_functional_test.bin", FUNCTIONAL_SUCCESS);
}
#[test]
fn klaus_interrupt() {
    run_klaus("tests/roms/6502_interrupt_test.bin", INTERRUPT_SUCCESS);
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
