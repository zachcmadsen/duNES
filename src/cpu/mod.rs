use proc_bitfield::bitfield;

use crate::bus::{Bus, DuNesBus, Pins};

use instruction::*;
use mode::*;

mod instruction;
mod mode;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;
const STACK_BASE: u16 = 0x0100;

/// The number of instructions to disassemble per call to disassemble.
const DISASSEMBLY_INSTRUCTIONS: usize = 15;
/// A LUT for opcode names and addressing modes.
const OPCODE_NAMES_AND_MODES: [(&str, AddressingMode); 256] = [
    ("BRK", AddressingMode::Implied),
    ("ORA", AddressingMode::IndexedIndirect),
    ("JAM", AddressingMode::Implied),
    ("SLO", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::ZeroPage),
    ("ORA", AddressingMode::ZeroPage),
    ("ASL", AddressingMode::ZeroPage),
    ("SLO", AddressingMode::ZeroPage),
    ("PHP", AddressingMode::Implied),
    ("ORA", AddressingMode::Immediate),
    ("ASL", AddressingMode::Accumulator),
    ("ANC", AddressingMode::Immediate),
    ("NOP", AddressingMode::Absolute),
    ("ORA", AddressingMode::Absolute),
    ("ASL", AddressingMode::Absolute),
    ("SLO", AddressingMode::Absolute),
    ("BPL", AddressingMode::Relative),
    ("ORA", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("SLO", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("ORA", AddressingMode::ZeroPageX),
    ("ASL", AddressingMode::ZeroPageX),
    ("SLO", AddressingMode::ZeroPageX),
    ("CLC", AddressingMode::Implied),
    ("ORA", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("SLO", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("ORA", AddressingMode::AbsoluteX),
    ("ASL", AddressingMode::AbsoluteX),
    ("SLO", AddressingMode::AbsoluteX),
    ("JSR", AddressingMode::Absolute),
    ("AND", AddressingMode::IndexedIndirect),
    ("JAM", AddressingMode::Implied),
    ("RLA", AddressingMode::IndexedIndirect),
    ("BIT", AddressingMode::ZeroPage),
    ("AND", AddressingMode::ZeroPage),
    ("ROL", AddressingMode::ZeroPage),
    ("RLA", AddressingMode::ZeroPage),
    ("PLP", AddressingMode::Implied),
    ("AND", AddressingMode::Immediate),
    ("ROL", AddressingMode::Accumulator),
    ("ANC", AddressingMode::Immediate),
    ("BIT", AddressingMode::Absolute),
    ("AND", AddressingMode::Absolute),
    ("ROL", AddressingMode::Absolute),
    ("RLA", AddressingMode::Absolute),
    ("BMI", AddressingMode::Relative),
    ("AND", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("RLA", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("AND", AddressingMode::ZeroPageX),
    ("ROL", AddressingMode::ZeroPageX),
    ("RLA", AddressingMode::ZeroPageX),
    ("SEC", AddressingMode::Implied),
    ("AND", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("RLA", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("AND", AddressingMode::AbsoluteX),
    ("ROL", AddressingMode::AbsoluteX),
    ("RLA", AddressingMode::AbsoluteX),
    ("RTI", AddressingMode::Implied),
    ("EOR", AddressingMode::IndexedIndirect),
    ("JAM", AddressingMode::Implied),
    ("SRE", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::ZeroPage),
    ("EOR", AddressingMode::ZeroPage),
    ("LSR", AddressingMode::ZeroPage),
    ("SRE", AddressingMode::ZeroPage),
    ("PHA", AddressingMode::Implied),
    ("EOR", AddressingMode::Immediate),
    ("LSR", AddressingMode::Accumulator),
    ("ALR", AddressingMode::Immediate),
    ("JMP", AddressingMode::Absolute),
    ("EOR", AddressingMode::Absolute),
    ("LSR", AddressingMode::Absolute),
    ("SRE", AddressingMode::Absolute),
    ("BVC", AddressingMode::Relative),
    ("EOR", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("SRE", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("EOR", AddressingMode::ZeroPageX),
    ("LSR", AddressingMode::ZeroPageX),
    ("SRE", AddressingMode::ZeroPageX),
    ("CLI", AddressingMode::Implied),
    ("EOR", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("SRE", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("EOR", AddressingMode::AbsoluteX),
    ("LSR", AddressingMode::AbsoluteX),
    ("SRE", AddressingMode::AbsoluteX),
    ("RTS", AddressingMode::Implied),
    ("ADC", AddressingMode::IndexedIndirect),
    ("JAM", AddressingMode::Implied),
    ("RRA", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::ZeroPage),
    ("ADC", AddressingMode::ZeroPage),
    ("ROR", AddressingMode::ZeroPage),
    ("RRA", AddressingMode::ZeroPage),
    ("PLA", AddressingMode::Implied),
    ("ADC", AddressingMode::Immediate),
    ("ROR", AddressingMode::Accumulator),
    ("ARR", AddressingMode::Immediate),
    ("JMP", AddressingMode::Indirect),
    ("ADC", AddressingMode::Absolute),
    ("ROR", AddressingMode::Absolute),
    ("RRA", AddressingMode::Absolute),
    ("BVS", AddressingMode::Relative),
    ("ADC", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("RRA", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("ADC", AddressingMode::ZeroPageX),
    ("ROR", AddressingMode::ZeroPageX),
    ("RRA", AddressingMode::ZeroPageX),
    ("SEI", AddressingMode::Implied),
    ("ADC", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("RRA", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("ADC", AddressingMode::AbsoluteX),
    ("ROR", AddressingMode::AbsoluteX),
    ("RRA", AddressingMode::AbsoluteX),
    ("NOP", AddressingMode::Immediate),
    ("STA", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::Immediate),
    ("SAX", AddressingMode::IndexedIndirect),
    ("STY", AddressingMode::ZeroPage),
    ("STA", AddressingMode::ZeroPage),
    ("STX", AddressingMode::ZeroPage),
    ("SAX", AddressingMode::ZeroPage),
    ("DEY", AddressingMode::Implied),
    ("NOP", AddressingMode::Immediate),
    ("TXA", AddressingMode::Implied),
    ("ANE", AddressingMode::Immediate),
    ("STY", AddressingMode::Absolute),
    ("STA", AddressingMode::Absolute),
    ("STX", AddressingMode::Absolute),
    ("SAX", AddressingMode::Absolute),
    ("BCC", AddressingMode::Relative),
    ("STA", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("SHA", AddressingMode::AbsoluteY),
    ("STY", AddressingMode::ZeroPageX),
    ("STA", AddressingMode::ZeroPageX),
    ("STX", AddressingMode::ZeroPageY),
    ("SAX", AddressingMode::ZeroPageY),
    ("TYA", AddressingMode::Implied),
    ("STA", AddressingMode::AbsoluteY),
    ("TXS", AddressingMode::Implied),
    ("TAS", AddressingMode::AbsoluteY),
    ("SHY", AddressingMode::AbsoluteX),
    ("STA", AddressingMode::AbsoluteX),
    ("SHX", AddressingMode::AbsoluteY),
    ("SHA", AddressingMode::IndirectIndexed),
    ("LDY", AddressingMode::Immediate),
    ("LDA", AddressingMode::IndexedIndirect),
    ("LDX", AddressingMode::Immediate),
    ("LAX", AddressingMode::IndexedIndirect),
    ("LDY", AddressingMode::ZeroPage),
    ("LDA", AddressingMode::ZeroPage),
    ("LDX", AddressingMode::ZeroPage),
    ("LAX", AddressingMode::ZeroPage),
    ("TAY", AddressingMode::Implied),
    ("LDA", AddressingMode::Immediate),
    ("TAX", AddressingMode::Implied),
    ("LXA", AddressingMode::Immediate),
    ("LDY", AddressingMode::Absolute),
    ("LDA", AddressingMode::Absolute),
    ("LDX", AddressingMode::Absolute),
    ("LAX", AddressingMode::Absolute),
    ("BCS", AddressingMode::Relative),
    ("LDA", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("LAX", AddressingMode::IndirectIndexed),
    ("LDY", AddressingMode::ZeroPageX),
    ("LDA", AddressingMode::ZeroPageX),
    ("LDX", AddressingMode::ZeroPageY),
    ("LAX", AddressingMode::ZeroPageY),
    ("CLV", AddressingMode::Implied),
    ("LDA", AddressingMode::AbsoluteY),
    ("TSX", AddressingMode::Implied),
    ("LAS", AddressingMode::AbsoluteY),
    ("LDY", AddressingMode::AbsoluteX),
    ("LDA", AddressingMode::AbsoluteX),
    ("LDX", AddressingMode::AbsoluteY),
    ("LAX", AddressingMode::AbsoluteY),
    ("CPY", AddressingMode::Immediate),
    ("CMP", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::Immediate),
    ("DCP", AddressingMode::IndexedIndirect),
    ("CPY", AddressingMode::ZeroPage),
    ("CMP", AddressingMode::ZeroPage),
    ("DEC", AddressingMode::ZeroPage),
    ("DCP", AddressingMode::ZeroPage),
    ("INY", AddressingMode::Implied),
    ("CMP", AddressingMode::Immediate),
    ("DEX", AddressingMode::Implied),
    ("SBX", AddressingMode::Immediate),
    ("CPY", AddressingMode::Absolute),
    ("CMP", AddressingMode::Absolute),
    ("DEC", AddressingMode::Absolute),
    ("DCP", AddressingMode::Absolute),
    ("BNE", AddressingMode::Relative),
    ("CMP", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("DCP", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("CMP", AddressingMode::ZeroPageX),
    ("DEC", AddressingMode::ZeroPageX),
    ("DCP", AddressingMode::ZeroPageX),
    ("CLD", AddressingMode::Implied),
    ("CMP", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("DCP", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("CMP", AddressingMode::AbsoluteX),
    ("DEC", AddressingMode::AbsoluteX),
    ("DCP", AddressingMode::AbsoluteX),
    ("CPX", AddressingMode::Immediate),
    ("SBC", AddressingMode::IndexedIndirect),
    ("NOP", AddressingMode::Immediate),
    ("ISB", AddressingMode::IndexedIndirect),
    ("CPX", AddressingMode::ZeroPage),
    ("SBC", AddressingMode::ZeroPage),
    ("INC", AddressingMode::ZeroPage),
    ("ISB", AddressingMode::ZeroPage),
    ("INX", AddressingMode::Implied),
    ("SBC", AddressingMode::Immediate),
    ("NOP", AddressingMode::Implied),
    ("SBC", AddressingMode::Immediate),
    ("CPX", AddressingMode::Absolute),
    ("SBC", AddressingMode::Absolute),
    ("INC", AddressingMode::Absolute),
    ("ISB", AddressingMode::Absolute),
    ("BEQ", AddressingMode::Relative),
    ("SBC", AddressingMode::IndirectIndexed),
    ("JAM", AddressingMode::Implied),
    ("ISB", AddressingMode::IndirectIndexed),
    ("NOP", AddressingMode::ZeroPageX),
    ("SBC", AddressingMode::ZeroPageX),
    ("INC", AddressingMode::ZeroPageX),
    ("ISB", AddressingMode::ZeroPageX),
    ("SED", AddressingMode::Implied),
    ("SBC", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::Implied),
    ("ISB", AddressingMode::AbsoluteY),
    ("NOP", AddressingMode::AbsoluteX),
    ("SBC", AddressingMode::AbsoluteX),
    ("INC", AddressingMode::AbsoluteX),
    ("ISB", AddressingMode::AbsoluteX),
];

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Status(u8): FromRaw, IntoRaw {
        pub c: bool @ 0,
        pub z: bool @ 1,
        pub i: bool @ 2,
        pub d: bool @ 3,
        pub b: bool @ 4,
        pub u: bool @ 5,
        pub v: bool @ 6,
        pub n: bool @ 7,
    }
}

#[derive(PartialEq, Eq)]
pub enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
    IndexedIndirect,
    Indirect,
    IndirectIndexed,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

pub struct Cpu<B> {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: Status,
    pub pins: Pins,
    pub cycles: u64,

    prev_irq: bool,
    irq: bool,
    prev_nmi: bool,
    prev_need_nmi: bool,
    need_nmi: bool,
    rst: bool,

    // TODO: Try boxing the bus once there's a benchmark with DuNesBus.
    pub bus: B,
}

impl<B: Bus> Cpu<B> {
    pub fn new(bus: B) -> Cpu<B> {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xfd,
            p: 0x24.into(),
            pins: Pins::default(),
            cycles: 0,
            prev_irq: false,
            irq: false,
            prev_nmi: false,
            prev_need_nmi: false,
            need_nmi: false,
            rst: true,
            bus,
        }
    }

    pub fn step(&mut self) {
        if self.rst || self.prev_need_nmi || self.prev_irq {
            let brk = if self.rst {
                // TODO: Reset CPU struct fields?
                self.rst = false;
                Cpu::rst
            } else if self.prev_need_nmi {
                self.need_nmi = false;
                Cpu::nmi
            } else {
                Cpu::irq
            };

            self.read_byte(self.pc);
            brk(self);
        } else {
            let opcode = self.consume_byte();
            match opcode {
                0x00 => self.brk(),
                0x01 => self.ora(Cpu::indexed_indirect),
                0x02 => self.jam(),
                0x03 => self.slo_indexed_indirect(),
                0x04 => self.nop(Cpu::zero_page),
                0x05 => self.ora(Cpu::zero_page),
                0x06 => self.asl(Cpu::zero_page),
                0x07 => self.slo_zero_page(),
                0x08 => self.php(),
                0x09 => self.ora(Cpu::immediate),
                0x0a => self.asl_accumulator(),
                0x0b => self.anc_immediate(),
                0x0c => self.nop(Cpu::absolute),
                0x0d => self.ora(Cpu::absolute),
                0x0e => self.asl(Cpu::absolute),
                0x0f => self.slo_absolute(),
                0x10 => self.bpl(),
                0x11 => self.ora(Cpu::indirect_indexed_read),
                0x12 => self.jam(),
                0x13 => self.slo_indirect_indexed(),
                0x14 => self.nop(Cpu::zero_page_x),
                0x15 => self.ora(Cpu::zero_page_x),
                0x16 => self.asl(Cpu::zero_page_x),
                0x17 => self.slo_zero_page_x(),
                0x18 => self.clc(),
                0x19 => self.ora(Cpu::absolute_y_read),
                0x1a => self.nop_implied(),
                0x1b => self.slo_absolute_y(),
                0x1c => self.nop(Cpu::absolute_x_read),
                0x1d => self.ora(Cpu::absolute_x_read),
                0x1e => self.asl(Cpu::absolute_x_write),
                0x1f => self.slo_absolute_x(),
                0x20 => self.jsr(),
                0x21 => self.and(Cpu::indexed_indirect),
                0x22 => self.jam(),
                0x23 => self.rla(Cpu::indexed_indirect),
                0x24 => self.bit(Cpu::zero_page),
                0x25 => self.and(Cpu::zero_page),
                0x26 => self.rol(Cpu::zero_page),
                0x27 => self.rla(Cpu::zero_page),
                0x28 => self.plp(),
                0x29 => self.and(Cpu::immediate),
                0x2a => self.rol_accumulator(),
                0x2b => self.anc_immediate(),
                0x2c => self.bit(Cpu::absolute),
                0x2d => self.and(Cpu::absolute),
                0x2e => self.rol(Cpu::absolute),
                0x2f => self.rla(Cpu::absolute),
                0x30 => self.bmi(),
                0x31 => self.and(Cpu::indirect_indexed_read),
                0x32 => self.jam(),
                0x33 => self.rla(Cpu::indirect_indexed_write),
                0x34 => self.nop(Cpu::zero_page_x),
                0x35 => self.and(Cpu::zero_page_x),
                0x36 => self.rol(Cpu::zero_page_x),
                0x37 => self.rla(Cpu::zero_page_x),
                0x38 => self.sec(),
                0x39 => self.and(Cpu::absolute_y_read),
                0x3a => self.nop_implied(),
                0x3b => self.rla(Cpu::absolute_y_write),
                0x3c => self.nop(Cpu::absolute_x_read),
                0x3d => self.and(Cpu::absolute_x_read),
                0x3e => self.rol(Cpu::absolute_x_write),
                0x3f => self.rla(Cpu::absolute_x_write),
                0x40 => self.rti(),
                0x41 => self.eor(Cpu::indexed_indirect),
                0x42 => self.jam(),
                0x43 => self.sre_indexed_indirect(),
                0x44 => self.nop(Cpu::zero_page),
                0x45 => self.eor(Cpu::zero_page),
                0x46 => self.lsr(Cpu::zero_page),
                0x47 => self.sre_zero_page(),
                0x48 => self.pha(),
                0x49 => self.eor(Cpu::immediate),
                0x4a => self.lsr_accumulator(),
                0x4b => self.alr_immediate(),
                0x4c => self.jmp(Cpu::absolute),
                0x4d => self.eor(Cpu::absolute),
                0x4e => self.lsr(Cpu::absolute),
                0x4f => self.sre_absolute(),
                0x50 => self.bvc(),
                0x51 => self.eor(Cpu::indirect_indexed_read),
                0x52 => self.jam(),
                0x53 => self.sre_indirect_indexed(),
                0x54 => self.nop(Cpu::zero_page_x),
                0x55 => self.eor(Cpu::zero_page_x),
                0x56 => self.lsr(Cpu::zero_page_x),
                0x57 => self.sre_zero_page_x(),
                0x58 => self.cli(),
                0x59 => self.eor(Cpu::absolute_y_read),
                0x5a => self.nop_implied(),
                0x5b => self.sre_absolute_y(),
                0x5c => self.nop(Cpu::absolute_x_read),
                0x5d => self.eor(Cpu::absolute_x_read),
                0x5e => self.lsr(Cpu::absolute_x_write),
                0x5f => self.sre_absolute_x(),
                0x60 => self.rts(),
                0x61 => self.adc(Cpu::indexed_indirect),
                0x62 => self.jam(),
                0x63 => self.rra(Cpu::indexed_indirect),
                0x64 => self.nop(Cpu::zero_page),
                0x65 => self.adc(Cpu::zero_page),
                0x66 => self.ror(Cpu::zero_page),
                0x67 => self.rra(Cpu::zero_page),
                0x68 => self.pla(),
                0x69 => self.adc(Cpu::immediate),
                0x6a => self.ror_accumulator(),
                0x6b => self.arr_immediate(),
                0x6c => self.jmp(Cpu::indirect),
                0x6d => self.adc(Cpu::absolute),
                0x6e => self.ror(Cpu::absolute),
                0x6f => self.rra(Cpu::absolute),
                0x70 => self.bvs(),
                0x71 => self.adc(Cpu::indirect_indexed_read),
                0x72 => self.jam(),
                0x73 => self.rra(Cpu::indirect_indexed_write),
                0x74 => self.nop(Cpu::zero_page_x),
                0x75 => self.adc(Cpu::zero_page_x),
                0x76 => self.ror(Cpu::zero_page_x),
                0x77 => self.rra(Cpu::zero_page_x),
                0x78 => self.sei(),
                0x79 => self.adc(Cpu::absolute_y_read),
                0x7a => self.nop_implied(),
                0x7b => self.rra(Cpu::absolute_y_write),
                0x7c => self.nop(Cpu::absolute_x_read),
                0x7d => self.adc(Cpu::absolute_x_read),
                0x7e => self.ror(Cpu::absolute_x_write),
                0x7f => self.rra(Cpu::absolute_x_write),
                0x80 => self.nop(Cpu::immediate),
                0x81 => self.sta_indexed_indirect(),
                0x82 => self.nop(Cpu::immediate),
                0x83 => self.sax(Cpu::indexed_indirect),
                0x84 => self.sty_zero_page(),
                0x85 => self.sta_zero_page(),
                0x86 => self.stx_zero_page(),
                0x87 => self.sax(Cpu::zero_page),
                0x88 => self.dey(),
                0x89 => self.nop(Cpu::immediate),
                0x8a => self.txa(),
                0x8b => self.ane_immediate(),
                0x8c => self.sty_absolute(),
                0x8d => self.sta_absolute(),
                0x8e => self.stx_absolute(),
                0x8f => self.sax(Cpu::absolute),
                0x90 => self.bcc(),
                0x91 => self.sta_indirect_indexed(),
                0x92 => self.jam(),
                0x93 => self.sha_absolute_y(),
                0x94 => self.sty_zero_page_x(),
                0x95 => self.sta_zero_page_x(),
                0x96 => self.stx_zero_page_y(),
                0x97 => self.sax(Cpu::zero_page_y),
                0x98 => self.tya(),
                0x99 => self.sta_absolute_y(),
                0x9a => self.txs(),
                0x9b => self.tas_absolute_y(),
                0x9c => self.shy_absolute_x(),
                0x9d => self.sta_absolute_x(),
                0x9e => self.shx_absolute_y(),
                0x9f => self.sha_indirect_indexed(),
                0xa0 => self.ldy(Cpu::immediate),
                0xa1 => self.lda(Cpu::indexed_indirect),
                0xa2 => self.ldx(Cpu::immediate),
                0xa3 => self.lax(Cpu::indexed_indirect),
                0xa4 => self.ldy(Cpu::zero_page),
                0xa5 => self.lda(Cpu::zero_page),
                0xa6 => self.ldx(Cpu::zero_page),
                0xa7 => self.lax(Cpu::zero_page),
                0xa8 => self.tay(),
                0xa9 => self.lda(Cpu::immediate),
                0xaa => self.tax(),
                0xab => self.lxa(),
                0xac => self.ldy(Cpu::absolute),
                0xad => self.lda(Cpu::absolute),
                0xae => self.ldx(Cpu::absolute),
                0xaf => self.lax(Cpu::absolute),
                0xb0 => self.bcs(),
                0xb1 => self.lda(Cpu::indirect_indexed_read),
                0xb2 => self.jam(),
                0xb3 => self.lax(Cpu::indirect_indexed_read),
                0xb4 => self.ldy(Cpu::zero_page_x),
                0xb5 => self.lda(Cpu::zero_page_x),
                0xb6 => self.ldx(Cpu::zero_page_y),
                0xb7 => self.lax(Cpu::zero_page_y),
                0xb8 => self.clv(),
                0xb9 => self.lda(Cpu::absolute_y_read),
                0xba => self.tsx(),
                0xbb => self.las(),
                0xbc => self.ldy(Cpu::absolute_x_read),
                0xbd => self.lda(Cpu::absolute_x_read),
                0xbe => self.ldx(Cpu::absolute_y_read),
                0xbf => self.lax(Cpu::absolute_y_read),
                0xc0 => self.cpy(Cpu::immediate),
                0xc1 => self.cmp(Cpu::indexed_indirect),
                0xc2 => self.nop(Cpu::immediate),
                0xc3 => self.dcp(Cpu::indexed_indirect),
                0xc4 => self.cpy(Cpu::zero_page),
                0xc5 => self.cmp(Cpu::zero_page),
                0xc6 => self.dec(Cpu::zero_page),
                0xc7 => self.dcp(Cpu::zero_page),
                0xc8 => self.iny(),
                0xc9 => self.cmp(Cpu::immediate),
                0xca => self.dex(),
                0xcb => self.sbx(),
                0xcc => self.cpy(Cpu::absolute),
                0xcd => self.cmp(Cpu::absolute),
                0xce => self.dec(Cpu::absolute),
                0xcf => self.dcp(Cpu::absolute),
                0xd0 => self.bne(),
                0xd1 => self.cmp(Cpu::indirect_indexed_read),
                0xd2 => self.jam(),
                0xd3 => self.dcp(Cpu::indirect_indexed_write),
                0xd4 => self.nop(Cpu::zero_page_x),
                0xd5 => self.cmp(Cpu::zero_page_x),
                0xd6 => self.dec(Cpu::zero_page_x),
                0xd7 => self.dcp(Cpu::zero_page_x),
                0xd8 => self.cld(),
                0xd9 => self.cmp(Cpu::absolute_y_read),
                0xda => self.nop_implied(),
                0xdb => self.dcp(Cpu::absolute_y_write),
                0xdc => self.nop(Cpu::absolute_x_read),
                0xdd => self.cmp(Cpu::absolute_x_read),
                0xde => self.dec(Cpu::absolute_x_write),
                0xdf => self.dcp(Cpu::absolute_x_write),
                0xe0 => self.cpx(Cpu::immediate),
                0xe1 => self.sbc(Cpu::indexed_indirect),
                0xe2 => self.nop(Cpu::immediate),
                0xe3 => self.isb(Cpu::indexed_indirect),
                0xe4 => self.cpx(Cpu::zero_page),
                0xe5 => self.sbc(Cpu::zero_page),
                0xe6 => self.inc(Cpu::zero_page),
                0xe7 => self.isb(Cpu::zero_page),
                0xe8 => self.inx(),
                0xe9 => self.sbc(Cpu::immediate),
                0xea => self.nop_implied(),
                0xeb => self.sbc(Cpu::immediate),
                0xec => self.cpx(Cpu::absolute),
                0xed => self.sbc(Cpu::absolute),
                0xee => self.inc(Cpu::absolute),
                0xef => self.isb(Cpu::absolute),
                0xf0 => self.beq(),
                0xf1 => self.sbc(Cpu::indirect_indexed_read),
                0xf2 => self.jam(),
                0xf3 => self.isb(Cpu::indirect_indexed_write),
                0xf4 => self.nop(Cpu::zero_page_x),
                0xf5 => self.sbc(Cpu::zero_page_x),
                0xf6 => self.inc(Cpu::zero_page_x),
                0xf7 => self.isb(Cpu::zero_page_x),
                0xf8 => self.sed(),
                0xf9 => self.sbc(Cpu::absolute_y_read),
                0xfa => self.nop_implied(),
                0xfb => self.isb(Cpu::absolute_y_write),
                0xfc => self.nop(Cpu::absolute_x_read),
                0xfd => self.sbc(Cpu::absolute_x_read),
                0xfe => self.inc(Cpu::absolute_x_write),
                0xff => self.isb(Cpu::absolute_x_write),
            }
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        if let Some(page) = self.pins.oam_dma {
            self.pins.oam_dma = None;
            self.oam_dma(page);
        }

        self.cycles += 1;

        self.pins.address = address;
        self.bus.read(&mut self.pins);

        self.poll_interrupts();

        self.pins.data
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address.wrapping_add(1));
        (high as u16) << 8 | low as u16
    }

    fn read_word_bugged(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address);
        // Indirect addressing modes are affected by a hardware bug where reads
        // that would cross a page instead wrap around in the same page.
        let high = self.read_byte(
            (address & 0xff00) | (address as u8).wrapping_add(1) as u16,
        );
        (high as u16) << 8 | low as u16
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if let Some(page) = self.pins.oam_dma {
            self.pins.oam_dma = None;
            self.oam_dma(page);
        }

        self.cycles += 1;

        self.pins.address = address;
        self.pins.data = data;
        self.bus.write(&mut self.pins);

        self.poll_interrupts();
    }

    fn consume_byte(&mut self) -> u8 {
        let data = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data
    }

    fn consume_word(&mut self) -> u16 {
        let data = self.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        data
    }

    fn peek(&mut self) -> u8 {
        self.read_byte(STACK_BASE + self.s as u16)
    }

    fn push(&mut self, data: u8) {
        self.write_byte(STACK_BASE + self.s as u16, data);
        self.s = self.s.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read_byte(STACK_BASE + self.s as u16)
    }

    fn poll_interrupts(&mut self) {
        // We need to track the previous status of the interrupt pins because
        // their statuses at the end of the second-to-last cycle determine if
        // the next instruction will be an interrupt.
        self.prev_irq = self.irq;
        self.irq = self.pins.irq && !self.p.i();

        self.prev_need_nmi = self.need_nmi;

        // An NMI is raised if the NMI pin goes from inactive during one cycle
        // to active during the next. The NMI stays "raised" until it's
        // handled.
        if !self.prev_nmi && self.pins.nmi {
            self.need_nmi = true;
        }
        self.prev_nmi = self.pins.nmi;

        if !self.rst && self.pins.rst {
            self.rst = self.pins.rst;
        }
    }

    fn oam_dma(&mut self, page: u8) {
        // OAM DMA should take an extra cycle if we're on an odd CPU
        // cycle.
        if self.cycles % 2 == 1 {
            // TODO: Find out what the actual contents of the bus is.
            self.read_byte(page as u16 * 0x100);
        }

        for offset in 0..=0xff {
            let data = self.read_byte(page as u16 * 0x100 + offset);
            self.write_byte(0x2004, data);
        }
    }

    fn add(&mut self, value: u8) {
        let a = self.a;
        let result = (self.a as u16)
            .wrapping_add(value as u16)
            .wrapping_add(self.p.c() as u16);
        self.a = result as u8;

        self.p.set_c(result > 0xff);
        self.p.set_z(self.a == 0);
        self.p.set_v(((a ^ self.a) & (value ^ self.a) & 0x80) != 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn branch(&mut self, condition: bool) {
        let offset = self.consume_byte() as i8 as u16;
        if condition {
            self.read_byte(self.pc);

            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset);

            if old_pc & 0xff00 != self.pc & 0xff00 {
                self.read_byte(
                    (old_pc & 0xff00)
                        | (old_pc as u8).wrapping_add(offset as u8) as u16,
                );
            }
        }
    }

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);

        self.p.set_c(register >= value);
        self.p.set_z(result == 0);
        self.p.set_n(result & 0x80 != 0);
    }

    fn irq(&mut self) {
        self.read_byte(self.pc);
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.push(self.p.into());
        self.p.set_i(true);
        self.pc = self.read_word(IRQ_VECTOR);
    }

    fn nmi(&mut self) {
        self.read_byte(self.pc);
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.push(self.p.into());
        // TODO: Should NMI not set the I flag?
        self.p.set_i(true);
        self.pc = self.read_word(NMI_VECTOR);
    }

    fn rst(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.s = self.s.wrapping_sub(1);
        self.peek();
        self.s = self.s.wrapping_sub(1);
        self.peek();
        self.s = self.s.wrapping_sub(1);
        self.p.set_i(true);
        self.pc = self.read_word(RESET_VECTOR);
    }

    #[inline]
    fn absolute(&mut self) -> u16 {
        self.consume_word()
    }

    #[inline]
    fn absolute_x_read(&mut self) -> u16 {
        let (low, page_cross) = self.consume_byte().overflowing_add(self.x);
        let high = self.consume_byte();
        let addr =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        // If the effective address is invalid, i.e., it crossed a page, then
        // it takes an extra read cycle to fix it.
        if page_cross {
            self.read_byte((high as u16) << 8 | low as u16);
        }

        addr
    }

    #[inline]
    fn absolute_x_write(&mut self) -> u16 {
        let (low, page_cross) = self.consume_byte().overflowing_add(self.x);
        let high = self.consume_byte();
        let addr =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        self.read_byte((high as u16) << 8 | low as u16);

        addr
    }

    #[inline]
    fn absolute_y_read(&mut self) -> u16 {
        let (low, page_cross) = self.consume_byte().overflowing_add(self.y);
        let high = self.consume_byte();
        let addr =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        // If the effective address is invalid, i.e., it crossed a page, then
        // it takes an extra read cycle to fix it.
        if page_cross {
            self.read_byte((high as u16) << 8 | low as u16);
        }

        addr
    }

    #[inline]
    fn absolute_y_write(&mut self) -> u16 {
        let (low, page_cross) = self.consume_byte().overflowing_add(self.y);
        let high = self.consume_byte();
        let addr =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        self.read_byte((high as u16) << 8 | low as u16);

        addr
    }

    #[inline]
    fn immediate(&mut self) -> u16 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        addr
    }

    #[inline]
    fn indexed_indirect(&mut self) -> u16 {
        let ptr = self.consume_byte();
        self.read_byte(ptr as u16);
        self.read_word_bugged(ptr.wrapping_add(self.x) as u16)
    }

    #[inline]
    fn indirect_indexed_read(&mut self) -> u16 {
        let ptr = self.consume_byte();
        let (low, did_cross_page) =
            self.read_byte(ptr as u16).overflowing_add(self.y);
        let high = self.read_byte(ptr.wrapping_add(1) as u16);
        let addr = (high.wrapping_add(did_cross_page as u8) as u16) << 8
            | (low as u16);

        // If the effective address is invalid, i.e., it crossed a page, then
        // it takes an extra read cycle to fix it.
        if did_cross_page {
            self.read_byte((high as u16) << 8 | low as u16);
        }

        addr
    }

    #[inline]
    fn indirect_indexed_write(&mut self) -> u16 {
        let ptr = self.consume_byte();
        let (low, did_cross_page) =
            self.read_byte(ptr as u16).overflowing_add(self.y);
        let high = self.read_byte(ptr.wrapping_add(1) as u16);
        let addr = (high.wrapping_add(did_cross_page as u8) as u16) << 8
            | (low as u16);

        self.read_byte((high as u16) << 8 | low as u16);

        addr
    }

    #[inline]
    fn indirect(&mut self) -> u16 {
        let ptr = self.consume_word();
        self.read_word_bugged(ptr)
    }

    #[inline]
    fn zero_page(&mut self) -> u16 {
        self.consume_byte() as u16
    }

    #[inline]
    fn zero_page_x(&mut self) -> u16 {
        let addr = self.consume_byte();
        self.read_byte(addr as u16);
        addr.wrapping_add(self.x) as u16
    }

    #[inline]
    fn zero_page_y(&mut self) -> u16 {
        let addr = self.consume_byte();
        self.read_byte(addr as u16);
        addr.wrapping_add(self.y) as u16
    }

    fn adc<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);
        self.add(value);
    }

    fn and<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.a &= self.read_byte(addr);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn asl<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x80 != 0;
        value <<= 1;
        self.write_byte(addr, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn asl_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a <<= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn bcc(&mut self) {
        self.branch(!self.p.c());
    }

    fn bcs(&mut self) {
        self.branch(self.p.c());
    }

    fn beq(&mut self) {
        self.branch(self.p.z());
    }

    fn bit<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);

        self.p.set_z(self.a & value == 0);
        self.p.set_v(Status::from(value).v());
        self.p.set_n(Status::from(value).n());
    }

    fn bmi(&mut self) {
        self.branch(self.p.n());
    }

    fn bne(&mut self) {
        self.branch(!self.p.z());
    }

    fn bpl(&mut self) {
        self.branch(!self.p.n());
    }

    fn brk(&mut self) {
        self.consume_byte();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.push(self.p.with_b(true).into());
        self.p.set_i(true);
        self.pc = self.read_word(IRQ_VECTOR);
    }

    fn bvc(&mut self) {
        self.branch(!self.p.v());
    }

    fn bvs(&mut self) {
        self.branch(self.p.v());
    }

    fn clc(&mut self) {
        self.read_byte(self.pc);
        self.p.set_c(false);
    }

    fn cld(&mut self) {
        self.read_byte(self.pc);
        self.p.set_d(false);
    }

    fn cli(&mut self) {
        self.read_byte(self.pc);
        self.p.set_i(false);
    }

    fn clv(&mut self) {
        self.read_byte(self.pc);
        self.p.set_v(false);
    }

    fn cmp<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);
        self.compare(self.a, value);
    }

    fn cpx<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);
        self.compare(self.x, value);
    }

    fn cpy<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);
        self.compare(self.y, value);
    }

    fn dcp<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        value = value.wrapping_sub(1);
        self.write_byte(addr, value);
        self.compare(self.a, value);
    }

    fn dec<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        value = value.wrapping_sub(1);
        self.write_byte(addr, value);

        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn dex(&mut self) {
        self.read_byte(self.pc);
        self.x = self.x.wrapping_sub(1);

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn dey(&mut self) {
        self.read_byte(self.pc);
        self.y = self.y.wrapping_sub(1);

        self.p.set_z(self.y == 0);
        self.p.set_n(self.y & 0x80 != 0);
    }

    fn eor<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.a ^= self.read_byte(addr);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn jmp<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.pc = addr;
    }

    fn inc<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        value = value.wrapping_add(1);
        self.write_byte(addr, value);

        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn inx(&mut self) {
        self.read_byte(self.pc);
        self.x = self.x.wrapping_add(1);

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn iny(&mut self) {
        self.read_byte(self.pc);
        self.y = self.y.wrapping_add(1);

        self.p.set_z(self.y == 0);
        self.p.set_n(self.y & 0x80 != 0);
    }

    fn isb<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        value = value.wrapping_add(1);
        self.write_byte(addr, value);
        self.add(value ^ 0xff);
    }

    fn jam(&mut self) {
        // Treat JAM as a one byte NOP.
        self.read_byte(self.pc);
    }

    fn jsr(&mut self) {
        let pcl = self.consume_byte();
        self.peek();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        let pch = self.consume_byte();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn las(&mut self) {
        let addr = self.absolute_y_read();
        self.a = self.read_byte(addr) & self.s;
        self.x = self.a;
        self.s = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lax<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let value = self.read_byte(addr);
        self.a = value;
        self.x = value;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lda<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.a = self.read_byte(addr);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ldx<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.x = self.read_byte(addr);

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn ldy<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.y = self.read_byte(addr);

        self.p.set_z(self.y == 0);
        self.p.set_n(self.y & 0x80 != 0);
    }

    fn lsr<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        self.write_byte(addr, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn lsr_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x01 != 0;
        self.a >>= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn lxa(&mut self) {
        let addr = self.immediate();
        // This instruction should perform a bitwise AND between a constant and
        // the operand before storing the result. The constant is unreliable
        // though. To remove uncertainty, we have the constant always be 0xff,
        // removing the need for the bitwise AND.
        self.a = self.read_byte(addr);
        self.x = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn nop<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.read_byte(addr);
    }

    fn nop_implied(&mut self) {
        self.read_byte(self.pc);
    }

    fn ora<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.a |= self.read_byte(addr);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn pha(&mut self) {
        self.read_byte(self.pc);
        self.push(self.a);
    }

    fn php(&mut self) {
        self.read_byte(self.pc);
        self.push(self.p.with_b(true).with_u(true).into());
    }

    fn pla(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.a = self.pop();

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn plp(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.p = Status::from(self.pop())
            .with_b(self.p.b())
            .with_u(self.p.u());
    }

    fn rla<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | self.p.c() as u8;
        self.write_byte(addr, value);
        self.a &= value;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn rol<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | self.p.c() as u8;
        self.write_byte(addr, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn rol_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a = ((self.a << 1) & 0xfe) | self.p.c() as u8;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ror<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x01 != 0;
        value = (self.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        self.write_byte(addr, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn ror_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x01 != 0;
        self.a = (self.p.c() as u8) << 7 | ((self.a >> 1) & 0x7f);

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn rra<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        let mut value = self.read_byte(addr);
        self.write_byte(addr, value);
        let carry = value & 0x01 != 0;
        value = (self.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        self.write_byte(addr, value);
        self.p.set_c(carry);
        self.add(value);
    }

    fn rti(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.p = Status::from(self.pop())
            .with_b(self.p.b())
            .with_u(self.p.u());
        let pcl = self.pop();
        let pch = self.pop();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn rts(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        let pcl = self.pop();
        let pch = self.pop();
        self.pc = (pch as u16) << 8 | pcl as u16;
        self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    fn sax<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        self.write_byte(addr, self.a & self.x);
    }

    fn sbc<F>(&mut self, mode: F)
    where
        F: FnOnce(&mut Cpu<B>) -> u16,
    {
        let addr = mode(self);
        // If we reformulate subtraction as addition, then we can use the same
        // logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = self.read_byte(addr) ^ 0xff;
        self.add(value);
    }

    fn sbx(&mut self) {
        let addr = self.immediate();
        let value = self.read_byte(addr);
        let carry = (self.a & self.x) >= value;
        self.x = (self.a & self.x).wrapping_sub(value);

        self.p.set_c(carry);
        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn sec(&mut self) {
        self.read_byte(self.pc);
        self.p.set_c(true);
    }

    fn sed(&mut self) {
        self.read_byte(self.pc);
        self.p.set_d(true);
    }

    fn sei(&mut self) {
        self.read_byte(self.pc);
        self.p.set_i(true);
    }

    // fn absolute(&mut self) -> u16 {
    //     self.consume_word()
    // }

    // fn asl<F>(&mut self, mode: F)
    // where
    //     F: FnOnce(&mut Cpu<B>) -> u16,
    // {
    //     let addr = mode(self);
    //     self.asl_inner(addr);
    // }

    // fn asl_inner(&mut self, addr: u16) {
    //     let mut value = self.read_byte(addr);
    //     self.write_byte(addr, value);
    //     let carry = value & 0x80 != 0;
    //     value <<= 1;
    //     self.write_byte(addr, value);

    //     self.p.set_c(carry);
    //     self.p.set_z(value == 0);
    //     self.p.set_n(value & 0x80 != 0);
    // }

    fn alr_immediate(&mut self) {
        alr!(self, immediate);
    }

    fn anc_immediate(&mut self) {
        anc!(self, immediate);
    }

    fn ane_immediate(&mut self) {
        ane!(self, immediate);
    }

    fn arr_immediate(&mut self) {
        arr!(self, immediate);
    }

    fn sha_absolute_y(&mut self) {
        sha!(self, absolute_y_write);
    }

    fn sha_indirect_indexed(&mut self) {
        sha!(self, indirect_indexed_write);
    }

    fn shx_absolute_y(&mut self) {
        shx!(self, absolute_y_write);
    }

    fn shy_absolute_x(&mut self) {
        shy!(self, absolute_x_write);
    }

    fn slo_absolute(&mut self) {
        slo!(self, absolute);
    }

    fn slo_absolute_x(&mut self) {
        slo!(self, absolute_x_write);
    }

    fn slo_absolute_y(&mut self) {
        slo!(self, absolute_y_write);
    }

    fn slo_indexed_indirect(&mut self) {
        slo!(self, indexed_indirect);
    }

    fn slo_indirect_indexed(&mut self) {
        slo!(self, indirect_indexed_write);
    }

    fn slo_zero_page(&mut self) {
        slo!(self, zero_page);
    }

    fn slo_zero_page_x(&mut self) {
        slo!(self, zero_page_x);
    }

    fn sre_absolute(&mut self) {
        sre!(self, absolute);
    }

    fn sre_absolute_x(&mut self) {
        sre!(self, absolute_x_write);
    }

    fn sre_absolute_y(&mut self) {
        sre!(self, absolute_y_write);
    }

    fn sre_indexed_indirect(&mut self) {
        sre!(self, indexed_indirect);
    }

    fn sre_indirect_indexed(&mut self) {
        sre!(self, indirect_indexed_write);
    }

    fn sre_zero_page(&mut self) {
        sre!(self, zero_page);
    }

    fn sre_zero_page_x(&mut self) {
        sre!(self, zero_page_x);
    }

    fn sta_absolute(&mut self) {
        sta!(self, absolute);
    }

    fn sta_absolute_x(&mut self) {
        sta!(self, absolute_x_write);
    }

    fn sta_absolute_y(&mut self) {
        sta!(self, absolute_y_write);
    }

    fn sta_indexed_indirect(&mut self) {
        sta!(self, indexed_indirect);
    }

    fn sta_indirect_indexed(&mut self) {
        sta!(self, indirect_indexed_write);
    }

    fn sta_zero_page(&mut self) {
        sta!(self, zero_page);
    }

    fn sta_zero_page_x(&mut self) {
        sta!(self, zero_page_x);
    }

    fn stx_absolute(&mut self) {
        stx!(self, absolute);
    }

    fn stx_zero_page(&mut self) {
        stx!(self, zero_page);
    }

    fn stx_zero_page_y(&mut self) {
        stx!(self, zero_page_y);
    }

    fn sty_absolute(&mut self) {
        sty!(self, absolute);
    }

    fn sty_zero_page(&mut self) {
        sty!(self, zero_page);
    }

    fn sty_zero_page_x(&mut self) {
        sty!(self, zero_page_x);
    }

    fn tas_absolute_y(&mut self) {
        tas!(self, absolute_y_write);
    }

    fn tax(&mut self) {
        self.read_byte(self.pc);
        self.x = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn tay(&mut self) {
        self.read_byte(self.pc);
        self.y = self.a;

        self.p.set_z(self.y == 0);
        self.p.set_n(self.y & 0x80 != 0);
    }

    fn tsx(&mut self) {
        self.read_byte(self.pc);
        self.x = self.s;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn txa(&mut self) {
        self.read_byte(self.pc);
        self.a = self.x;

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn txs(&mut self) {
        self.read_byte(self.pc);
        self.s = self.x;
    }

    fn tya(&mut self) {
        self.read_byte(self.pc);
        self.a = self.y;

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }
}

impl Cpu<DuNesBus> {
    pub fn disassemble(&self) -> Vec<String> {
        let bus = &self.bus;
        let read_byte = |pc: &mut u16| -> u8 {
            let byte = bus.read_unclocked(*pc);
            *pc = pc.wrapping_add(1);
            byte
        };
        let read_word = |pc: &mut u16| -> u16 {
            let low = bus.read_unclocked(*pc);
            let high = bus.read_unclocked(*pc + 1);
            *pc = pc.wrapping_add(2);
            (high as u16) << 8 | low as u16
        };

        let mut pc = self.pc;
        // TODO: Pass in a mutable Vec to avoid save an allocation.
        let mut disasm = Vec::with_capacity(DISASSEMBLY_INSTRUCTIONS);
        for _ in 0..DISASSEMBLY_INSTRUCTIONS {
            // Only disassemble RAM and PRG RAM/ROM for now.
            let opcode = if pc <= 0x1fff || 0x4020 <= pc {
                bus.read_unclocked(pc)
            } else {
                0
            };

            let address = pc;
            pc = pc.wrapping_add(1);

            let (name, mode) = &OPCODE_NAMES_AND_MODES[opcode as usize];

            // TODO: Use a single format! instead of two.
            let operand = match mode {
                AddressingMode::Absolute => {
                    format!("${:04X}", read_word(&mut pc))
                }
                AddressingMode::AbsoluteX => {
                    format!("${:04X}, X", read_word(&mut pc))
                }
                AddressingMode::AbsoluteY => {
                    format!("${:04X}, Y", read_word(&mut pc))
                }
                AddressingMode::Accumulator => "A".to_string(),
                AddressingMode::Immediate => {
                    format!("#${:02X}", read_byte(&mut pc))
                }
                AddressingMode::Indirect => {
                    format!("(${:04X})", read_word(&mut pc))
                }
                AddressingMode::Implied => "".to_string(),
                AddressingMode::IndexedIndirect => {
                    format!("(${:02X}, X)", read_byte(&mut pc))
                }
                AddressingMode::IndirectIndexed => {
                    format!("(${:02X}), Y", read_byte(&mut pc))
                }
                AddressingMode::Relative => {
                    let byte = read_byte(&mut pc) as i8 as u16;
                    let target = pc.wrapping_add(byte);
                    format!("${:04X}", target)
                }
                AddressingMode::ZeroPage => {
                    format!("${:02X}", read_byte(&mut pc))
                }
                AddressingMode::ZeroPageX => {
                    format!("${:02X}, X", read_byte(&mut pc))
                }
                AddressingMode::ZeroPageY => {
                    format!("${:02X}, Y", read_byte(&mut pc))
                }
            };

            // Pad the operand with enough spaces to make all lines have the
            // same length.
            disasm.push(format!("{:04X}: {} {:10}", address, name, operand));
        }

        disasm
    }
}
