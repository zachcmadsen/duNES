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
    const OPCODE_LUT: [fn(&mut Cpu<B>); 256] = [
        Cpu::brk,
        Cpu::ora_indexed_indirect,
        Cpu::jam,
        Cpu::slo_indexed_indirect,
        Cpu::nop_zero_page,
        Cpu::ora_zero_page,
        Cpu::asl_zero_page,
        Cpu::slo_zero_page,
        Cpu::php,
        Cpu::ora_immediate,
        Cpu::asl_accumulator,
        Cpu::anc_immediate,
        Cpu::nop_absolute,
        Cpu::ora_absolute,
        Cpu::asl_absolute,
        Cpu::slo_absolute,
        Cpu::bpl,
        Cpu::ora_indirect_indexed,
        Cpu::jam,
        Cpu::slo_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::ora_zero_page_x,
        Cpu::asl_zero_page_x,
        Cpu::slo_zero_page_x,
        Cpu::clc,
        Cpu::ora_absolute_y,
        Cpu::nop_implied,
        Cpu::slo_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::ora_absolute_x,
        Cpu::asl_absolute_x,
        Cpu::slo_absolute_x,
        Cpu::jsr,
        Cpu::and_indexed_indirect,
        Cpu::jam,
        Cpu::rla_indexed_indirect,
        Cpu::bit_zero_page,
        Cpu::and_zero_page,
        Cpu::rol_zero_page,
        Cpu::rla_zero_page,
        Cpu::plp,
        Cpu::and_immediate,
        Cpu::rol_accumulator,
        Cpu::anc_immediate,
        Cpu::bit_absolute,
        Cpu::and_absolute,
        Cpu::rol_absolute,
        Cpu::rla_absolute,
        Cpu::bmi,
        Cpu::and_indirect_indexed,
        Cpu::jam,
        Cpu::rla_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::and_zero_page_x,
        Cpu::rol_zero_page_x,
        Cpu::rla_zero_page_x,
        Cpu::sec,
        Cpu::and_absolute_y,
        Cpu::nop_implied,
        Cpu::rla_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::and_absolute_x,
        Cpu::rol_absolute_x,
        Cpu::rla_absolute_x,
        Cpu::rti,
        Cpu::eor_indexed_indirect,
        Cpu::jam,
        Cpu::sre_indexed_indirect,
        Cpu::nop_zero_page,
        Cpu::eor_zero_page,
        Cpu::lsr_zero_page,
        Cpu::sre_zero_page,
        Cpu::pha,
        Cpu::eor_immediate,
        Cpu::lsr_accumulator,
        Cpu::alr_immediate,
        Cpu::jmp_absolute,
        Cpu::eor_absolute,
        Cpu::lsr_absolute,
        Cpu::sre_absolute,
        Cpu::bvc,
        Cpu::eor_indirect_indexed,
        Cpu::jam,
        Cpu::sre_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::eor_zero_page_x,
        Cpu::lsr_zero_page_x,
        Cpu::sre_zero_page_x,
        Cpu::cli,
        Cpu::eor_absolute_y,
        Cpu::nop_implied,
        Cpu::sre_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::eor_absolute_x,
        Cpu::lsr_absolute_x,
        Cpu::sre_absolute_x,
        Cpu::rts,
        Cpu::adc_indexed_indirect,
        Cpu::jam,
        Cpu::rra_indexed_indirect,
        Cpu::nop_zero_page,
        Cpu::adc_zero_page,
        Cpu::ror_zero_page,
        Cpu::rra_zero_page,
        Cpu::pla,
        Cpu::adc_immediate,
        Cpu::ror_accumulator,
        Cpu::arr_immediate,
        Cpu::jmp_indirect,
        Cpu::adc_absolute,
        Cpu::ror_absolute,
        Cpu::rra_absolute,
        Cpu::bvs,
        Cpu::adc_indirect_indexed,
        Cpu::jam,
        Cpu::rra_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::adc_zero_page_x,
        Cpu::ror_zero_page_x,
        Cpu::rra_zero_page_x,
        Cpu::sei,
        Cpu::adc_absolute_y,
        Cpu::nop_implied,
        Cpu::rra_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::adc_absolute_x,
        Cpu::ror_absolute_x,
        Cpu::rra_absolute_x,
        Cpu::nop_immediate,
        Cpu::sta_indexed_indirect,
        Cpu::nop_immediate,
        Cpu::sax_indexed_indirect,
        Cpu::sty_zero_page,
        Cpu::sta_zero_page,
        Cpu::stx_zero_page,
        Cpu::sax_zero_page,
        Cpu::dey,
        Cpu::nop_immediate,
        Cpu::txa,
        Cpu::ane_immediate,
        Cpu::sty_absolute,
        Cpu::sta_absolute,
        Cpu::stx_absolute,
        Cpu::sax_absolute,
        Cpu::bcc,
        Cpu::sta_indirect_indexed,
        Cpu::jam,
        Cpu::sha_absolute_y,
        Cpu::sty_zero_page_x,
        Cpu::sta_zero_page_x,
        Cpu::stx_zero_page_y,
        Cpu::sax_zero_page_y,
        Cpu::tya,
        Cpu::sta_absolute_y,
        Cpu::txs,
        Cpu::tas_absolute_y,
        Cpu::shy_absolute_x,
        Cpu::sta_absolute_x,
        Cpu::shx_absolute_y,
        Cpu::sha_indirect_indexed,
        Cpu::ldy_immediate,
        Cpu::lda_indexed_indirect,
        Cpu::ldx_immediate,
        Cpu::lax_indexed_indirect,
        Cpu::ldy_zero_page,
        Cpu::lda_zero_page,
        Cpu::ldx_zero_page,
        Cpu::lax_zero_page,
        Cpu::tay,
        Cpu::lda_immediate,
        Cpu::tax,
        Cpu::lxa_immediate,
        Cpu::ldy_absolute,
        Cpu::lda_absolute,
        Cpu::ldx_absolute,
        Cpu::lax_absolute,
        Cpu::bcs,
        Cpu::lda_indirect_indexed,
        Cpu::jam,
        Cpu::lax_indirect_indexed,
        Cpu::ldy_zero_page_x,
        Cpu::lda_zero_page_x,
        Cpu::ldx_zero_page_y,
        Cpu::lax_zero_page_y,
        Cpu::clv,
        Cpu::lda_absolute_y,
        Cpu::tsx,
        Cpu::las_absolute_y,
        Cpu::ldy_absolute_x,
        Cpu::lda_absolute_x,
        Cpu::ldx_absolute_y,
        Cpu::lax_absolute_y,
        Cpu::cpy_immediate,
        Cpu::cmp_indexed_indirect,
        Cpu::nop_immediate,
        Cpu::dcp_indexed_indirect,
        Cpu::cpy_zero_page,
        Cpu::cmp_zero_page,
        Cpu::dec_zero_page,
        Cpu::dcp_zero_page,
        Cpu::iny,
        Cpu::cmp_immediate,
        Cpu::dex,
        Cpu::sbx_immediate,
        Cpu::cpy_absolute,
        Cpu::cmp_absolute,
        Cpu::dec_absolute,
        Cpu::dcp_absolute,
        Cpu::bne,
        Cpu::cmp_indirect_indexed,
        Cpu::jam,
        Cpu::dcp_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::cmp_zero_page_x,
        Cpu::dec_zero_page_x,
        Cpu::dcp_zero_page_x,
        Cpu::cld,
        Cpu::cmp_absolute_y,
        Cpu::nop_implied,
        Cpu::dcp_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::cmp_absolute_x,
        Cpu::dec_absolute_x,
        Cpu::dcp_absolute_x,
        Cpu::cpx_immediate,
        Cpu::sbc_indexed_indirect,
        Cpu::nop_immediate,
        Cpu::isb_indexed_indirect,
        Cpu::cpx_zero_page,
        Cpu::sbc_zero_page,
        Cpu::inc_zero_page,
        Cpu::isb_zero_page,
        Cpu::inx,
        Cpu::sbc_immediate,
        Cpu::nop_implied,
        Cpu::sbc_immediate,
        Cpu::cpx_absolute,
        Cpu::sbc_absolute,
        Cpu::inc_absolute,
        Cpu::isb_absolute,
        Cpu::beq,
        Cpu::sbc_indirect_indexed,
        Cpu::jam,
        Cpu::isb_indirect_indexed,
        Cpu::nop_zero_page_x,
        Cpu::sbc_zero_page_x,
        Cpu::inc_zero_page_x,
        Cpu::isb_zero_page_x,
        Cpu::sed,
        Cpu::sbc_absolute_y,
        Cpu::nop_implied,
        Cpu::isb_absolute_y,
        Cpu::nop_absolute_x,
        Cpu::sbc_absolute_x,
        Cpu::inc_absolute_x,
        Cpu::isb_absolute_x,
    ];

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
            Cpu::OPCODE_LUT[opcode as usize](self);
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
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

    fn adc_absolute(&mut self) {
        adc!(self, absolute);
    }

    fn adc_absolute_x(&mut self) {
        adc!(self, absolute_x_read);
    }

    fn adc_absolute_y(&mut self) {
        adc!(self, absolute_y_read);
    }

    fn adc_immediate(&mut self) {
        adc!(self, immediate);
    }

    fn adc_indexed_indirect(&mut self) {
        adc!(self, indexed_indirect);
    }

    fn adc_indirect_indexed(&mut self) {
        adc!(self, indirect_indexed_read);
    }

    fn adc_zero_page(&mut self) {
        adc!(self, zero_page);
    }

    fn adc_zero_page_x(&mut self) {
        adc!(self, zero_page_x);
    }

    fn alr_immediate(&mut self) {
        alr!(self, immediate);
    }

    fn anc_immediate(&mut self) {
        anc!(self, immediate);
    }

    fn and_absolute(&mut self) {
        and!(self, absolute);
    }

    fn and_absolute_x(&mut self) {
        and!(self, absolute_x_read);
    }

    fn and_absolute_y(&mut self) {
        and!(self, absolute_y_read);
    }

    fn and_immediate(&mut self) {
        and!(self, immediate);
    }

    fn and_indexed_indirect(&mut self) {
        and!(self, indexed_indirect);
    }

    fn and_indirect_indexed(&mut self) {
        and!(self, indirect_indexed_read);
    }

    fn and_zero_page(&mut self) {
        and!(self, zero_page);
    }

    fn and_zero_page_x(&mut self) {
        and!(self, zero_page_x);
    }

    fn ane_immediate(&mut self) {
        ane!(self, immediate);
    }

    fn arr_immediate(&mut self) {
        arr!(self, immediate);
    }

    fn asl_absolute(&mut self) {
        asl!(self, absolute);
    }

    fn asl_absolute_x(&mut self) {
        asl!(self, absolute_x_write);
    }

    fn asl_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a <<= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn asl_zero_page(&mut self) {
        asl!(self, zero_page);
    }

    fn asl_zero_page_x(&mut self) {
        asl!(self, zero_page_x);
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

    fn bit_absolute(&mut self) {
        bit!(self, absolute);
    }

    fn bit_zero_page(&mut self) {
        bit!(self, zero_page);
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

    fn cmp_absolute(&mut self) {
        cmp!(self, absolute);
    }

    fn cmp_absolute_x(&mut self) {
        cmp!(self, absolute_x_read);
    }

    fn cmp_absolute_y(&mut self) {
        cmp!(self, absolute_y_read);
    }

    fn cmp_immediate(&mut self) {
        cmp!(self, immediate);
    }

    fn cmp_indexed_indirect(&mut self) {
        cmp!(self, indexed_indirect);
    }

    fn cmp_indirect_indexed(&mut self) {
        cmp!(self, indirect_indexed_read);
    }

    fn cmp_zero_page(&mut self) {
        cmp!(self, zero_page);
    }

    fn cmp_zero_page_x(&mut self) {
        cmp!(self, zero_page_x);
    }

    fn cpx_absolute(&mut self) {
        cpx!(self, absolute);
    }

    fn cpx_immediate(&mut self) {
        cpx!(self, immediate);
    }

    fn cpx_zero_page(&mut self) {
        cpx!(self, zero_page);
    }

    fn cpy_absolute(&mut self) {
        cpy!(self, absolute);
    }

    fn cpy_immediate(&mut self) {
        cpy!(self, immediate);
    }

    fn cpy_zero_page(&mut self) {
        cpy!(self, zero_page);
    }

    fn dcp_absolute(&mut self) {
        dcp!(self, absolute);
    }

    fn dcp_absolute_x(&mut self) {
        dcp!(self, absolute_x_write);
    }

    fn dcp_absolute_y(&mut self) {
        dcp!(self, absolute_y_write);
    }

    fn dcp_indexed_indirect(&mut self) {
        dcp!(self, indexed_indirect);
    }

    fn dcp_indirect_indexed(&mut self) {
        dcp!(self, indirect_indexed_write);
    }

    fn dcp_zero_page(&mut self) {
        dcp!(self, zero_page);
    }

    fn dcp_zero_page_x(&mut self) {
        dcp!(self, zero_page_x);
    }

    fn dec_absolute(&mut self) {
        dec!(self, absolute);
    }

    fn dec_absolute_x(&mut self) {
        dec!(self, absolute_x_write);
    }

    fn dec_zero_page(&mut self) {
        dec!(self, zero_page);
    }

    fn dec_zero_page_x(&mut self) {
        dec!(self, zero_page_x);
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

    fn eor_absolute(&mut self) {
        eor!(self, absolute);
    }

    fn eor_absolute_x(&mut self) {
        eor!(self, absolute_x_read);
    }

    fn eor_absolute_y(&mut self) {
        eor!(self, absolute_y_read);
    }

    fn eor_immediate(&mut self) {
        eor!(self, immediate);
    }

    fn eor_indexed_indirect(&mut self) {
        eor!(self, indexed_indirect);
    }

    fn eor_indirect_indexed(&mut self) {
        eor!(self, indirect_indexed_read);
    }

    fn eor_zero_page(&mut self) {
        eor!(self, zero_page);
    }

    fn eor_zero_page_x(&mut self) {
        eor!(self, zero_page_x);
    }

    fn inc_absolute(&mut self) {
        inc!(self, absolute);
    }

    fn inc_absolute_x(&mut self) {
        inc!(self, absolute_x_write);
    }

    fn inc_zero_page(&mut self) {
        inc!(self, zero_page);
    }

    fn inc_zero_page_x(&mut self) {
        inc!(self, zero_page_x);
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

    fn isb_absolute(&mut self) {
        isb!(self, absolute);
    }

    fn isb_absolute_x(&mut self) {
        isb!(self, absolute_x_write);
    }

    fn isb_absolute_y(&mut self) {
        isb!(self, absolute_y_write);
    }

    fn isb_indexed_indirect(&mut self) {
        isb!(self, indexed_indirect);
    }

    fn isb_indirect_indexed(&mut self) {
        isb!(self, indirect_indexed_write);
    }

    fn isb_zero_page(&mut self) {
        isb!(self, zero_page);
    }

    fn isb_zero_page_x(&mut self) {
        isb!(self, zero_page_x);
    }

    fn jam(&mut self) {
        // Treat JAM as a one byte NOP.
        self.read_byte(self.pc);
    }

    fn jmp_absolute(&mut self) {
        jmp!(self, absolute);
    }

    fn jmp_indirect(&mut self) {
        jmp!(self, indirect);
    }

    fn jsr(&mut self) {
        let pcl = self.consume_byte();
        self.peek();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        let pch = self.consume_byte();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn las_absolute_y(&mut self) {
        las!(self, absolute_y_read);
    }

    fn lax_absolute(&mut self) {
        lax!(self, absolute);
    }

    fn lax_absolute_y(&mut self) {
        lax!(self, absolute_y_read);
    }

    fn lax_indexed_indirect(&mut self) {
        lax!(self, indexed_indirect);
    }

    fn lax_indirect_indexed(&mut self) {
        lax!(self, indirect_indexed_read);
    }

    fn lax_zero_page(&mut self) {
        lax!(self, zero_page);
    }

    fn lax_zero_page_y(&mut self) {
        lax!(self, zero_page_y);
    }

    fn lda_absolute(&mut self) {
        lda!(self, absolute);
    }

    fn lda_absolute_x(&mut self) {
        lda!(self, absolute_x_read);
    }

    fn lda_absolute_y(&mut self) {
        lda!(self, absolute_y_read);
    }

    fn lda_immediate(&mut self) {
        lda!(self, immediate);
    }

    fn lda_indexed_indirect(&mut self) {
        lda!(self, indexed_indirect);
    }

    fn lda_indirect_indexed(&mut self) {
        lda!(self, indirect_indexed_read);
    }

    fn lda_zero_page(&mut self) {
        lda!(self, zero_page);
    }

    fn lda_zero_page_x(&mut self) {
        lda!(self, zero_page_x);
    }

    fn ldx_absolute(&mut self) {
        ldx!(self, absolute);
    }

    fn ldx_absolute_y(&mut self) {
        ldx!(self, absolute_y_read);
    }

    fn ldx_immediate(&mut self) {
        ldx!(self, immediate);
    }

    fn ldx_zero_page(&mut self) {
        ldx!(self, zero_page);
    }

    fn ldx_zero_page_y(&mut self) {
        ldx!(self, zero_page_y);
    }

    fn ldy_absolute(&mut self) {
        ldy!(self, absolute);
    }

    fn ldy_absolute_x(&mut self) {
        ldy!(self, absolute_x_read);
    }

    fn ldy_immediate(&mut self) {
        ldy!(self, immediate);
    }

    fn ldy_zero_page(&mut self) {
        ldy!(self, zero_page);
    }

    fn ldy_zero_page_x(&mut self) {
        ldy!(self, zero_page_x);
    }

    fn lsr_absolute(&mut self) {
        lsr!(self, absolute);
    }

    fn lsr_absolute_x(&mut self) {
        lsr!(self, absolute_x_write);
    }

    fn lsr_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x01 != 0;
        self.a >>= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn lsr_zero_page(&mut self) {
        lsr!(self, zero_page);
    }

    fn lsr_zero_page_x(&mut self) {
        lsr!(self, zero_page_x);
    }

    fn lxa_immediate(&mut self) {
        lxa!(self, immediate);
    }

    fn nop_absolute(&mut self) {
        nop!(self, absolute);
    }

    fn nop_absolute_x(&mut self) {
        nop!(self, absolute_x_read);
    }

    fn nop_immediate(&mut self) {
        nop!(self, immediate);
    }

    fn nop_implied(&mut self) {
        self.read_byte(self.pc);
    }

    fn nop_zero_page(&mut self) {
        nop!(self, zero_page);
    }

    fn nop_zero_page_x(&mut self) {
        nop!(self, zero_page_x);
    }

    fn ora_absolute(&mut self) {
        ora!(self, absolute);
    }

    fn ora_absolute_x(&mut self) {
        ora!(self, absolute_x_read);
    }

    fn ora_absolute_y(&mut self) {
        ora!(self, absolute_y_read);
    }

    fn ora_immediate(&mut self) {
        ora!(self, immediate);
    }

    fn ora_indexed_indirect(&mut self) {
        ora!(self, indexed_indirect);
    }

    fn ora_indirect_indexed(&mut self) {
        ora!(self, indirect_indexed_read);
    }

    fn ora_zero_page(&mut self) {
        ora!(self, zero_page);
    }

    fn ora_zero_page_x(&mut self) {
        ora!(self, zero_page_x);
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

    fn rla_absolute(&mut self) {
        rla!(self, absolute);
    }

    fn rla_absolute_x(&mut self) {
        rla!(self, absolute_x_write);
    }

    fn rla_absolute_y(&mut self) {
        rla!(self, absolute_y_write);
    }

    fn rla_indexed_indirect(&mut self) {
        rla!(self, indexed_indirect);
    }

    fn rla_indirect_indexed(&mut self) {
        rla!(self, indirect_indexed_write);
    }

    fn rla_zero_page(&mut self) {
        rla!(self, zero_page);
    }

    fn rla_zero_page_x(&mut self) {
        rla!(self, zero_page_x);
    }

    fn rol_absolute(&mut self) {
        rol!(self, absolute);
    }

    fn rol_absolute_x(&mut self) {
        rol!(self, absolute_x_write);
    }

    fn rol_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a = ((self.a << 1) & 0xfe) | self.p.c() as u8;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn rol_zero_page(&mut self) {
        rol!(self, zero_page);
    }

    fn rol_zero_page_x(&mut self) {
        rol!(self, zero_page_x);
    }

    fn ror_absolute(&mut self) {
        ror!(self, absolute);
    }

    fn ror_absolute_x(&mut self) {
        ror!(self, absolute_x_write);
    }

    fn ror_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x01 != 0;
        self.a = (self.p.c() as u8) << 7 | ((self.a >> 1) & 0x7f);

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ror_zero_page(&mut self) {
        ror!(self, zero_page);
    }

    fn ror_zero_page_x(&mut self) {
        ror!(self, zero_page_x);
    }

    fn rra_absolute(&mut self) {
        rra!(self, absolute);
    }

    fn rra_absolute_x(&mut self) {
        rra!(self, absolute_x_write);
    }

    fn rra_absolute_y(&mut self) {
        rra!(self, absolute_y_write);
    }

    fn rra_indexed_indirect(&mut self) {
        rra!(self, indexed_indirect);
    }

    fn rra_indirect_indexed(&mut self) {
        rra!(self, indirect_indexed_write);
    }

    fn rra_zero_page(&mut self) {
        rra!(self, zero_page);
    }

    fn rra_zero_page_x(&mut self) {
        rra!(self, zero_page_x);
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

    fn sax_absolute(&mut self) {
        sax!(self, absolute);
    }

    fn sax_indexed_indirect(&mut self) {
        sax!(self, indexed_indirect);
    }

    fn sax_zero_page(&mut self) {
        sax!(self, zero_page);
    }

    fn sax_zero_page_y(&mut self) {
        sax!(self, zero_page_y);
    }

    fn sbc_absolute(&mut self) {
        sbc!(self, absolute);
    }

    fn sbc_absolute_x(&mut self) {
        sbc!(self, absolute_x_read);
    }

    fn sbc_absolute_y(&mut self) {
        sbc!(self, absolute_y_read);
    }

    fn sbc_immediate(&mut self) {
        sbc!(self, immediate);
    }

    fn sbc_indexed_indirect(&mut self) {
        sbc!(self, indexed_indirect);
    }

    fn sbc_indirect_indexed(&mut self) {
        sbc!(self, indirect_indexed_read);
    }

    fn sbc_zero_page(&mut self) {
        sbc!(self, zero_page);
    }

    fn sbc_zero_page_x(&mut self) {
        sbc!(self, zero_page_x);
    }

    fn sbx_immediate(&mut self) {
        sbx!(self, immediate);
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
