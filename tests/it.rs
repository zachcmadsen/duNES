use std::fs;

use dunes::{Bus, Cpu};

const ZERO_PAGE_START: usize = 0xa;
const CODE_SEGMENT_START: u16 = 0x400;
const INTERRUPT_FEEDBACK_REGISTER: u16 = 0xbffc;
const IRQ_MASK: u8 = 0x1;
const NMI_MASK: u8 = 0x2;
const FUNCTIONAL_SUCCESS: u16 = 0x336d;
const INTERRUPT_SUCCESS: u16 = 0x6f5;

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

#[test]
fn klaus_functional() {
    run_klaus("tests/roms/6502_functional_test.bin", FUNCTIONAL_SUCCESS);
}

#[test]
fn klaus_interrupt() {
    run_klaus("tests/roms/6502_interrupt_test.bin", INTERRUPT_SUCCESS);
}
