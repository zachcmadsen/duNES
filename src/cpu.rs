use std::collections::BTreeMap;

use proc_bitfield::bitfield;

use crate::bus::{Bus, Pins};

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;

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

#[derive(PartialEq)]
enum AddressingMode {
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

#[derive(PartialEq)]
enum Interrupt {
    Brk,
    Irq,
    Nmi,
    Rst,
}

const STACK_BASE: u16 = 0x0100;

/// A 6502 CPU.
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

    pub bus: B,
}

impl<B: Bus> Cpu<B> {
    /// Constructs a new `Cpu` in a power-up state.
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

    /// Executes the next instruction.
    pub fn step(&mut self) {
        if self.rst || self.prev_need_nmi || self.prev_irq {
            let kind = if self.rst {
                // TODO: Reset CPU struct fields?
                self.rst = false;
                Interrupt::Rst
            } else if self.prev_need_nmi {
                self.need_nmi = false;
                Interrupt::Nmi
            } else {
                Interrupt::Irq
            };

            self.read_byte(self.pc);
            self.brk(kind);
        } else {
            let opcode = self.consume_byte();
            match opcode {
                0x00 => self.brk(Interrupt::Brk),
                0x01 => self.ora(AddressingMode::IndexedIndirect),
                0x02 => self.jam(),
                0x03 => self.slo(AddressingMode::IndexedIndirect),
                0x04 => self.nop(AddressingMode::ZeroPage),
                0x05 => self.ora(AddressingMode::ZeroPage),
                0x06 => self.asl(AddressingMode::ZeroPage),
                0x07 => self.slo(AddressingMode::ZeroPage),
                0x08 => self.php(),
                0x09 => self.ora(AddressingMode::Immediate),
                0x0a => self.asl_accumulator(),
                0x0b => self.anc(AddressingMode::Immediate),
                0x0c => self.nop(AddressingMode::Absolute),
                0x0d => self.ora(AddressingMode::Absolute),
                0x0e => self.asl(AddressingMode::Absolute),
                0x0f => self.slo(AddressingMode::Absolute),
                0x10 => self.bpl(),
                0x11 => self.ora(AddressingMode::IndirectIndexed),
                0x12 => self.jam(),
                0x13 => self.slo(AddressingMode::IndirectIndexed),
                0x14 => self.nop(AddressingMode::ZeroPageX),
                0x15 => self.ora(AddressingMode::ZeroPageX),
                0x16 => self.asl(AddressingMode::ZeroPageX),
                0x17 => self.slo(AddressingMode::ZeroPageX),
                0x18 => self.clc(),
                0x19 => self.ora(AddressingMode::AbsoluteY),
                0x1a => self.nop_implied(),
                0x1b => self.slo(AddressingMode::AbsoluteY),
                0x1c => self.nop(AddressingMode::AbsoluteX),
                0x1d => self.ora(AddressingMode::AbsoluteX),
                0x1e => self.asl(AddressingMode::AbsoluteX),
                0x1f => self.slo(AddressingMode::AbsoluteX),
                0x20 => self.jsr(),
                0x21 => self.and(AddressingMode::IndexedIndirect),
                0x22 => self.jam(),
                0x23 => self.rla(AddressingMode::IndexedIndirect),
                0x24 => self.bit(AddressingMode::ZeroPage),
                0x25 => self.and(AddressingMode::ZeroPage),
                0x26 => self.rol(AddressingMode::ZeroPage),
                0x27 => self.rla(AddressingMode::ZeroPage),
                0x28 => self.plp(),
                0x29 => self.and(AddressingMode::Immediate),
                0x2a => self.rol_accumulator(),
                0x2b => self.anc(AddressingMode::Immediate),
                0x2c => self.bit(AddressingMode::Absolute),
                0x2d => self.and(AddressingMode::Absolute),
                0x2e => self.rol(AddressingMode::Absolute),
                0x2f => self.rla(AddressingMode::Absolute),
                0x30 => self.bmi(),
                0x31 => self.and(AddressingMode::IndirectIndexed),
                0x32 => self.jam(),
                0x33 => self.rla(AddressingMode::IndirectIndexed),
                0x34 => self.nop(AddressingMode::ZeroPageX),
                0x35 => self.and(AddressingMode::ZeroPageX),
                0x36 => self.rol(AddressingMode::ZeroPageX),
                0x37 => self.rla(AddressingMode::ZeroPageX),
                0x38 => self.sec(),
                0x39 => self.and(AddressingMode::AbsoluteY),
                0x3a => self.nop_implied(),
                0x3b => self.rla(AddressingMode::AbsoluteY),
                0x3c => self.nop(AddressingMode::AbsoluteX),
                0x3d => self.and(AddressingMode::AbsoluteX),
                0x3e => self.rol(AddressingMode::AbsoluteX),
                0x3f => self.rla(AddressingMode::AbsoluteX),
                0x40 => self.rti(),
                0x41 => self.eor(AddressingMode::IndexedIndirect),
                0x42 => self.jam(),
                0x43 => self.sre(AddressingMode::IndexedIndirect),
                0x44 => self.nop(AddressingMode::ZeroPage),
                0x45 => self.eor(AddressingMode::ZeroPage),
                0x46 => self.lsr(AddressingMode::ZeroPage),
                0x47 => self.sre(AddressingMode::ZeroPage),
                0x48 => self.pha(),
                0x49 => self.eor(AddressingMode::Immediate),
                0x4a => self.lsr_accumulator(),
                0x4b => self.alr(AddressingMode::Immediate),
                0x4c => self.jmp(AddressingMode::Absolute),
                0x4d => self.eor(AddressingMode::Absolute),
                0x4e => self.lsr(AddressingMode::Absolute),
                0x4f => self.sre(AddressingMode::Absolute),
                0x50 => self.bvc(),
                0x51 => self.eor(AddressingMode::IndirectIndexed),
                0x52 => self.jam(),
                0x53 => self.sre(AddressingMode::IndirectIndexed),
                0x54 => self.nop(AddressingMode::ZeroPageX),
                0x55 => self.eor(AddressingMode::ZeroPageX),
                0x56 => self.lsr(AddressingMode::ZeroPageX),
                0x57 => self.sre(AddressingMode::ZeroPageX),
                0x58 => self.cli(),
                0x59 => self.eor(AddressingMode::AbsoluteY),
                0x5a => self.nop_implied(),
                0x5b => self.sre(AddressingMode::AbsoluteY),
                0x5c => self.nop(AddressingMode::AbsoluteX),
                0x5d => self.eor(AddressingMode::AbsoluteX),
                0x5e => self.lsr(AddressingMode::AbsoluteX),
                0x5f => self.sre(AddressingMode::AbsoluteX),
                0x60 => self.rts(),
                0x61 => self.adc(AddressingMode::IndexedIndirect),
                0x62 => self.jam(),
                0x63 => self.rra(AddressingMode::IndexedIndirect),
                0x64 => self.nop(AddressingMode::ZeroPage),
                0x65 => self.adc(AddressingMode::ZeroPage),
                0x66 => self.ror(AddressingMode::ZeroPage),
                0x67 => self.rra(AddressingMode::ZeroPage),
                0x68 => self.pla(),
                0x69 => self.adc(AddressingMode::Immediate),
                0x6a => self.ror_accumulator(),
                0x6b => self.arr(AddressingMode::Immediate),
                0x6c => self.jmp(AddressingMode::Indirect),
                0x6d => self.adc(AddressingMode::Absolute),
                0x6e => self.ror(AddressingMode::Absolute),
                0x6f => self.rra(AddressingMode::Absolute),
                0x70 => self.bvs(),
                0x71 => self.adc(AddressingMode::IndirectIndexed),
                0x72 => self.jam(),
                0x73 => self.rra(AddressingMode::IndirectIndexed),
                0x74 => self.nop(AddressingMode::ZeroPageX),
                0x75 => self.adc(AddressingMode::ZeroPageX),
                0x76 => self.ror(AddressingMode::ZeroPageX),
                0x77 => self.rra(AddressingMode::ZeroPageX),
                0x78 => self.sei(),
                0x79 => self.adc(AddressingMode::AbsoluteY),
                0x7a => self.nop_implied(),
                0x7b => self.rra(AddressingMode::AbsoluteY),
                0x7c => self.nop(AddressingMode::AbsoluteX),
                0x7d => self.adc(AddressingMode::AbsoluteX),
                0x7e => self.ror(AddressingMode::AbsoluteX),
                0x7f => self.rra(AddressingMode::AbsoluteX),
                0x80 => self.nop(AddressingMode::Immediate),
                0x81 => self.sta(AddressingMode::IndexedIndirect),
                0x82 => self.nop(AddressingMode::Immediate),
                0x83 => self.sax(AddressingMode::IndexedIndirect),
                0x84 => self.sty(AddressingMode::ZeroPage),
                0x85 => self.sta(AddressingMode::ZeroPage),
                0x86 => self.stx(AddressingMode::ZeroPage),
                0x87 => self.sax(AddressingMode::ZeroPage),
                0x88 => self.dey(),
                0x89 => self.nop(AddressingMode::Immediate),
                0x8a => self.txa(),
                0x8b => self.ane(AddressingMode::Immediate),
                0x8c => self.sty(AddressingMode::Absolute),
                0x8d => self.sta(AddressingMode::Absolute),
                0x8e => self.stx(AddressingMode::Absolute),
                0x8f => self.sax(AddressingMode::Absolute),
                0x90 => self.bcc(),
                0x91 => self.sta(AddressingMode::IndirectIndexed),
                0x92 => self.jam(),
                0x93 => self.sha(AddressingMode::AbsoluteY),
                0x94 => self.sty(AddressingMode::ZeroPageX),
                0x95 => self.sta(AddressingMode::ZeroPageX),
                0x96 => self.stx(AddressingMode::ZeroPageY),
                0x97 => self.sax(AddressingMode::ZeroPageY),
                0x98 => self.tya(),
                0x99 => self.sta(AddressingMode::AbsoluteY),
                0x9a => self.txs(),
                0x9b => self.tas(AddressingMode::AbsoluteY),
                0x9c => self.shy(AddressingMode::AbsoluteX),
                0x9d => self.sta(AddressingMode::AbsoluteX),
                0x9e => self.shx(AddressingMode::AbsoluteY),
                0x9f => self.sha(AddressingMode::IndirectIndexed),
                0xa0 => self.ldy(AddressingMode::Immediate),
                0xa1 => self.lda(AddressingMode::IndexedIndirect),
                0xa2 => self.ldx(AddressingMode::Immediate),
                0xa3 => self.lax(AddressingMode::IndexedIndirect),
                0xa4 => self.ldy(AddressingMode::ZeroPage),
                0xa5 => self.lda(AddressingMode::ZeroPage),
                0xa6 => self.ldx(AddressingMode::ZeroPage),
                0xa7 => self.lax(AddressingMode::ZeroPage),
                0xa8 => self.tay(),
                0xa9 => self.lda(AddressingMode::Immediate),
                0xaa => self.tax(),
                0xab => self.lxa(AddressingMode::Immediate),
                0xac => self.ldy(AddressingMode::Absolute),
                0xad => self.lda(AddressingMode::Absolute),
                0xae => self.ldx(AddressingMode::Absolute),
                0xaf => self.lax(AddressingMode::Absolute),
                0xb0 => self.bcs(),
                0xb1 => self.lda(AddressingMode::IndirectIndexed),
                0xb2 => self.jam(),
                0xb3 => self.lax(AddressingMode::IndirectIndexed),
                0xb4 => self.ldy(AddressingMode::ZeroPageX),
                0xb5 => self.lda(AddressingMode::ZeroPageX),
                0xb6 => self.ldx(AddressingMode::ZeroPageY),
                0xb7 => self.lax(AddressingMode::ZeroPageY),
                0xb8 => self.clv(),
                0xb9 => self.lda(AddressingMode::AbsoluteY),
                0xba => self.tsx(),
                0xbb => self.las(AddressingMode::AbsoluteY),
                0xbc => self.ldy(AddressingMode::AbsoluteX),
                0xbd => self.lda(AddressingMode::AbsoluteX),
                0xbe => self.ldx(AddressingMode::AbsoluteY),
                0xbf => self.lax(AddressingMode::AbsoluteY),
                0xc0 => self.cpy(AddressingMode::Immediate),
                0xc1 => self.cmp(AddressingMode::IndexedIndirect),
                0xc2 => self.nop(AddressingMode::Immediate),
                0xc3 => self.dcp(AddressingMode::IndexedIndirect),
                0xc4 => self.cpy(AddressingMode::ZeroPage),
                0xc5 => self.cmp(AddressingMode::ZeroPage),
                0xc6 => self.dec(AddressingMode::ZeroPage),
                0xc7 => self.dcp(AddressingMode::ZeroPage),
                0xc8 => self.iny(),
                0xc9 => self.cmp(AddressingMode::Immediate),
                0xca => self.dex(),
                0xcb => self.sbx(AddressingMode::Immediate),
                0xcc => self.cpy(AddressingMode::Absolute),
                0xcd => self.cmp(AddressingMode::Absolute),
                0xce => self.dec(AddressingMode::Absolute),
                0xcf => self.dcp(AddressingMode::Absolute),
                0xd0 => self.bne(),
                0xd1 => self.cmp(AddressingMode::IndirectIndexed),
                0xd2 => self.jam(),
                0xd3 => self.dcp(AddressingMode::IndirectIndexed),
                0xd4 => self.nop(AddressingMode::ZeroPageX),
                0xd5 => self.cmp(AddressingMode::ZeroPageX),
                0xd6 => self.dec(AddressingMode::ZeroPageX),
                0xd7 => self.dcp(AddressingMode::ZeroPageX),
                0xd8 => self.cld(),
                0xd9 => self.cmp(AddressingMode::AbsoluteY),
                0xda => self.nop_implied(),
                0xdb => self.dcp(AddressingMode::AbsoluteY),
                0xdc => self.nop(AddressingMode::AbsoluteX),
                0xdd => self.cmp(AddressingMode::AbsoluteX),
                0xde => self.dec(AddressingMode::AbsoluteX),
                0xdf => self.dcp(AddressingMode::AbsoluteX),
                0xe0 => self.cpx(AddressingMode::Immediate),
                0xe1 => self.sbc(AddressingMode::IndexedIndirect),
                0xe2 => self.nop(AddressingMode::Immediate),
                0xe3 => self.isb(AddressingMode::IndexedIndirect),
                0xe4 => self.cpx(AddressingMode::ZeroPage),
                0xe5 => self.sbc(AddressingMode::ZeroPage),
                0xe6 => self.inc(AddressingMode::ZeroPage),
                0xe7 => self.isb(AddressingMode::ZeroPage),
                0xe8 => self.inx(),
                0xe9 => self.sbc(AddressingMode::Immediate),
                0xea => self.nop_implied(),
                0xeb => self.sbc(AddressingMode::Immediate),
                0xec => self.cpx(AddressingMode::Absolute),
                0xed => self.sbc(AddressingMode::Absolute),
                0xee => self.inc(AddressingMode::Absolute),
                0xef => self.isb(AddressingMode::Absolute),
                0xf0 => self.beq(),
                0xf1 => self.sbc(AddressingMode::IndirectIndexed),
                0xf2 => self.jam(),
                0xf3 => self.isb(AddressingMode::IndirectIndexed),
                0xf4 => self.nop(AddressingMode::ZeroPageX),
                0xf5 => self.sbc(AddressingMode::ZeroPageX),
                0xf6 => self.inc(AddressingMode::ZeroPageX),
                0xf7 => self.isb(AddressingMode::ZeroPageX),
                0xf8 => self.sed(),
                0xf9 => self.sbc(AddressingMode::AbsoluteY),
                0xfa => self.nop_implied(),
                0xfb => self.isb(AddressingMode::AbsoluteY),
                0xfc => self.nop(AddressingMode::AbsoluteX),
                0xfd => self.sbc(AddressingMode::AbsoluteX),
                0xfe => self.inc(AddressingMode::AbsoluteX),
                0xff => self.isb(AddressingMode::AbsoluteX),
            }
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
}

// Instruction helpers
impl<B: Bus> Cpu<B> {
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

    fn effective_address(
        &mut self,
        mode: AddressingMode,
        is_write_instr: bool,
    ) -> u16 {
        match mode {
            AddressingMode::Absolute => self.consume_word(),
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                let index = if mode == AddressingMode::AbsoluteX {
                    self.x
                } else {
                    self.y
                };

                let (low, page_cross) =
                    self.consume_byte().overflowing_add(index);
                let high = self.consume_byte();

                let effective_address =
                    (high.wrapping_add(page_cross as u8) as u16) << 8
                        | (low as u16);

                // If the effective address is invalid, i.e., it crossed a
                // page, then it takes an extra read cycle to fix it. Write
                // instructions always have the extra read since they can't
                // undo a write to an invalid address.
                if page_cross || is_write_instr {
                    self.read_byte((high as u16) << 8 | low as u16);
                }

                effective_address
            }
            AddressingMode::Immediate => {
                let effective_address = self.pc;
                self.pc = self.pc.wrapping_add(1);
                effective_address
            }
            AddressingMode::Indirect => {
                let ptr = self.consume_word();
                self.read_word_bugged(ptr)
            }
            AddressingMode::IndexedIndirect => {
                let ptr = self.consume_byte();
                self.read_byte(ptr as u16);
                self.read_word_bugged(ptr.wrapping_add(self.x) as u16)
            }
            AddressingMode::IndirectIndexed => {
                let ptr = self.consume_byte();

                let (low, did_cross_page) =
                    self.read_byte(ptr as u16).overflowing_add(self.y);
                let high = self.read_byte(ptr.wrapping_add(1) as u16);

                let effective_address =
                    (high.wrapping_add(did_cross_page as u8) as u16) << 8
                        | (low as u16);

                // If the effective address is invalid, i.e., it crossed a
                // page, then it takes an extra read cycle to fix it. Write
                // instructions always have the extra read since they can't
                // undo a write to an invalid address.
                if did_cross_page || is_write_instr {
                    self.read_byte((high as u16) << 8 | low as u16);
                }

                effective_address
            }
            AddressingMode::ZeroPage => self.consume_byte() as u16,
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
                let index = if mode == AddressingMode::ZeroPageX {
                    self.x
                } else {
                    self.y
                };

                let address = self.consume_byte();
                self.read_byte(address as u16);

                address.wrapping_add(index) as u16
            }
            _ => unreachable!(),
        }
    }
}

// Instructions
impl<B: Bus> Cpu<B> {
    fn adc(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
        self.add(value);
    }

    fn anc(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a &= self.read_byte(effective_address);

        self.p.set_c(self.a & 0x80 != 0);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn and(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a &= self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn alr(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a &= self.read_byte(effective_address);
        let carry = self.a & 0x01 != 0;
        self.a = self.a.wrapping_shr(1);

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ane(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        // Treat ANE as a NOP since it's unstable.
        self.read_byte(effective_address);
    }

    fn arr(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a &= self.read_byte(effective_address);
        self.a = (self.p.c() as u8) << 7 | self.a.wrapping_shr(1);

        // TODO: Explain how the carry and overflow flag are set.
        self.p.set_c(self.a & 0x40 != 0);
        self.p.set_z(self.a == 0);
        self.p
            .set_v(((self.p.c() as u8) ^ ((self.a >> 5) & 0x01)) != 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn asl_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a <<= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn asl(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x80 != 0;
        value <<= 1;
        self.write_byte(effective_address, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
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

    fn bit(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);

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

    fn brk(&mut self, kind: Interrupt) {
        self.read_byte(self.pc);
        if kind == Interrupt::Brk {
            self.pc += 1;
        }

        if kind == Interrupt::Rst {
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
        } else {
            self.push((self.pc >> 8) as u8);
            self.push(self.pc as u8);
            self.push(self.p.with_b(kind == Interrupt::Brk).into());
        }

        // TODO: Implement interrupt hijacking.
        // TODO: Should NMI not set the I flag?
        self.p.set_i(true);
        let vector = match kind {
            Interrupt::Brk | Interrupt::Irq => IRQ_VECTOR,
            Interrupt::Nmi => NMI_VECTOR,
            Interrupt::Rst => RESET_VECTOR,
        };
        self.pc = self.read_word(vector);
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

    fn cmp(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
        self.compare(self.a, value);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
        self.compare(self.x, value);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
        self.compare(self.y, value);
    }

    fn dcp(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        value = value.wrapping_sub(1);
        self.write_byte(effective_address, value);
        self.compare(self.a, value);
    }

    fn dec(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        value = value.wrapping_sub(1);
        self.write_byte(effective_address, value);

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

    fn eor(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a ^= self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn inc(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        value = value.wrapping_add(1);
        self.write_byte(effective_address, value);

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

    fn isb(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        value = value.wrapping_add(1);
        self.write_byte(effective_address, value);
        self.add(value ^ 0xff);
    }

    fn jam(&mut self) {
        // Treat JAM as a one byte NOP.
        self.read_byte(self.pc);
    }

    fn jmp(&mut self, mode: AddressingMode) {
        self.pc = self.effective_address(mode, false);
    }

    fn jsr(&mut self) {
        let pcl = self.consume_byte();
        self.peek();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        let pch = self.consume_byte();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn las(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        self.a = self.read_byte(effective_address) & self.s;
        self.x = self.a;
        self.s = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lax(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
        self.a = value;
        self.x = value;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lda(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a = self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.x = self.read_byte(effective_address);

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        let operand = self.read_byte(effective_address);
        self.y = operand;

        self.p.set_z(self.y == 0);
        self.p.set_n(self.y & 0x80 != 0);
    }

    fn lsr_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x01 != 0;
        self.a >>= 1;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn lsr(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        self.write_byte(effective_address, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn lxa(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        // This instruction should perform a bitwise AND between a constant and
        // the operand before storing the result. The constant is unreliable
        // though. To remove uncertainty, we have the constant always be 0xff,
        // removing the need for the bitwise AND.
        self.a = self.read_byte(effective_address);
        self.x = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn nop_implied(&mut self) {
        self.read_byte(self.pc);
    }

    fn nop(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.read_byte(effective_address);
    }

    fn ora(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);
        self.a |= self.read_byte(effective_address);

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

    fn rla(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | self.p.c() as u8;
        self.write_byte(effective_address, value);
        self.a &= value;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn rol_accumulator(&mut self) {
        self.read_byte(self.pc);
        let carry = self.a & 0x80 != 0;
        self.a = ((self.a << 1) & 0xfe) | self.p.c() as u8;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn rol(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | self.p.c() as u8;
        self.write_byte(effective_address, value);

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

    fn ror(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value = (self.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        self.write_byte(effective_address, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn rra(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value = (self.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        self.write_byte(effective_address, value);
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

    fn sax(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        self.write_byte(effective_address, self.a & self.x);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        // If we reformulate subtraction as addition, then we can use the same
        // logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = self.read_byte(effective_address) ^ 0xff;
        self.add(value);
    }

    fn sbx(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, false);

        let value = self.read_byte(effective_address);
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

    fn sha(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.a & self.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.a as u16 & self.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
    }

    fn shx(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.x as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn shy(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.y & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.y as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn slo(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x80 != 0;
        value <<= 1;
        self.write_byte(effective_address, value);
        self.a |= value;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn sre(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        self.write_byte(effective_address, value);
        self.a ^= value;

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        self.write_byte(effective_address, self.a);
    }

    fn stx(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        self.write_byte(effective_address, self.x);
    }

    fn sty(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        self.write_byte(effective_address, self.y);
    }

    fn tas(&mut self, mode: AddressingMode) {
        let effective_address = self.effective_address(mode, true);

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.a & self.x & (high_byte as u8).wrapping_add(1);
        self.s = self.a & self.x;

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.a as u16 & self.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
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

const INSTRUCTIONS: [(&str, AddressingMode); 256] = [
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

impl<B: Bus> Cpu<B> {
    pub fn disassemble(&self) -> BTreeMap<u16, String> {
        let mut disasm = BTreeMap::new();

        let mut pc = 0;

        while pc < 0xffff {
            let opcode = self.bus.hidden_read(pc);
            let (name, addresing_mode) = &INSTRUCTIONS[opcode as usize];
            let pc_to_insert = pc;
            pc += 1;

            let string_stuff = match addresing_mode {
                AddressingMode::Absolute => {
                    let low = self.bus.hidden_read(pc);
                    let high = self.bus.hidden_read(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}")
                }
                AddressingMode::AbsoluteX => {
                    let low = self.bus.hidden_read(pc);
                    let high = self.bus.hidden_read(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}, X")
                }
                AddressingMode::AbsoluteY => {
                    let low = self.bus.hidden_read(pc);
                    let high = self.bus.hidden_read(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}, Y")
                }
                AddressingMode::Accumulator => "A".to_string(),
                AddressingMode::Immediate => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("#${value:02X}")
                }
                AddressingMode::Indirect => {
                    let low = self.bus.hidden_read(pc);
                    let high = self.bus.hidden_read(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("(${ea:04X})")
                }
                AddressingMode::Implied => "".to_string(),
                AddressingMode::IndexedIndirect => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("(${value:02X}, X)")
                }
                AddressingMode::IndirectIndexed => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("(${value:02X}), Y")
                }
                AddressingMode::Relative => {
                    let value = self.bus.hidden_read(pc) as i8 as u16;
                    pc += 1;
                    let target = pc.wrapping_add(value);
                    format!("${target:04X}")
                }
                AddressingMode::ZeroPage => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("${value:02X}")
                }
                AddressingMode::ZeroPageX => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("${value:02X}, X")
                }
                AddressingMode::ZeroPageY => {
                    let value = self.bus.hidden_read(pc);
                    pc += 1;
                    format!("${value:02X}, Y")
                }
            };

            disasm.insert(pc_to_insert, format!("{} {}", name, string_stuff));
        }

        disasm
    }
}
