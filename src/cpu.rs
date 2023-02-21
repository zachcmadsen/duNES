use std::collections::BTreeMap;

use proc_bitfield::bitfield;

use crate::bus::{Bus, DuNesBus, Pins};

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;
const STACK_BASE: u16 = 0x0100;

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

#[derive(PartialEq, Eq)]
enum Interrupt {
    Brk,
    Irq,
    Nmi,
    Rst,
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

    pub bus: B,
}

impl<B: Bus> Cpu<B> {
    const OPCODE_LUT: [fn(&mut Cpu<B>); 256] = [
        Cpu::brk::<{ Interrupt::Brk }>,
        Cpu::ora::<{ AddressingMode::IndexedIndirect }>,
        Cpu::jam,
        Cpu::slo::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::ZeroPage }>,
        Cpu::ora::<{ AddressingMode::ZeroPage }>,
        Cpu::asl::<{ AddressingMode::ZeroPage }>,
        Cpu::slo::<{ AddressingMode::ZeroPage }>,
        Cpu::php,
        Cpu::ora::<{ AddressingMode::Immediate }>,
        Cpu::asl_accumulator,
        Cpu::anc::<{ AddressingMode::Immediate }>,
        Cpu::nop::<{ AddressingMode::Absolute }>,
        Cpu::ora::<{ AddressingMode::Absolute }>,
        Cpu::asl::<{ AddressingMode::Absolute }>,
        Cpu::slo::<{ AddressingMode::Absolute }>,
        Cpu::bpl,
        Cpu::ora::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::slo::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::ora::<{ AddressingMode::ZeroPageX }>,
        Cpu::asl::<{ AddressingMode::ZeroPageX }>,
        Cpu::slo::<{ AddressingMode::ZeroPageX }>,
        Cpu::clc,
        Cpu::ora::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::slo::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::ora::<{ AddressingMode::AbsoluteX }>,
        Cpu::asl::<{ AddressingMode::AbsoluteX }>,
        Cpu::slo::<{ AddressingMode::AbsoluteX }>,
        Cpu::jsr,
        Cpu::and::<{ AddressingMode::IndexedIndirect }>,
        Cpu::jam,
        Cpu::rla::<{ AddressingMode::IndexedIndirect }>,
        Cpu::bit::<{ AddressingMode::ZeroPage }>,
        Cpu::and::<{ AddressingMode::ZeroPage }>,
        Cpu::rol::<{ AddressingMode::ZeroPage }>,
        Cpu::rla::<{ AddressingMode::ZeroPage }>,
        Cpu::plp,
        Cpu::and::<{ AddressingMode::Immediate }>,
        Cpu::rol_accumulator,
        Cpu::anc::<{ AddressingMode::Immediate }>,
        Cpu::bit::<{ AddressingMode::Absolute }>,
        Cpu::and::<{ AddressingMode::Absolute }>,
        Cpu::rol::<{ AddressingMode::Absolute }>,
        Cpu::rla::<{ AddressingMode::Absolute }>,
        Cpu::bmi,
        Cpu::and::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::rla::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::and::<{ AddressingMode::ZeroPageX }>,
        Cpu::rol::<{ AddressingMode::ZeroPageX }>,
        Cpu::rla::<{ AddressingMode::ZeroPageX }>,
        Cpu::sec,
        Cpu::and::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::rla::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::and::<{ AddressingMode::AbsoluteX }>,
        Cpu::rol::<{ AddressingMode::AbsoluteX }>,
        Cpu::rla::<{ AddressingMode::AbsoluteX }>,
        Cpu::rti,
        Cpu::eor::<{ AddressingMode::IndexedIndirect }>,
        Cpu::jam,
        Cpu::sre::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::ZeroPage }>,
        Cpu::eor::<{ AddressingMode::ZeroPage }>,
        Cpu::lsr::<{ AddressingMode::ZeroPage }>,
        Cpu::sre::<{ AddressingMode::ZeroPage }>,
        Cpu::pha,
        Cpu::eor::<{ AddressingMode::Immediate }>,
        Cpu::lsr_accumulator,
        Cpu::alr::<{ AddressingMode::Immediate }>,
        Cpu::jmp::<{ AddressingMode::Absolute }>,
        Cpu::eor::<{ AddressingMode::Absolute }>,
        Cpu::lsr::<{ AddressingMode::Absolute }>,
        Cpu::sre::<{ AddressingMode::Absolute }>,
        Cpu::bvc,
        Cpu::eor::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::sre::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::eor::<{ AddressingMode::ZeroPageX }>,
        Cpu::lsr::<{ AddressingMode::ZeroPageX }>,
        Cpu::sre::<{ AddressingMode::ZeroPageX }>,
        Cpu::cli,
        Cpu::eor::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::sre::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::eor::<{ AddressingMode::AbsoluteX }>,
        Cpu::lsr::<{ AddressingMode::AbsoluteX }>,
        Cpu::sre::<{ AddressingMode::AbsoluteX }>,
        Cpu::rts,
        Cpu::adc::<{ AddressingMode::IndexedIndirect }>,
        Cpu::jam,
        Cpu::rra::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::ZeroPage }>,
        Cpu::adc::<{ AddressingMode::ZeroPage }>,
        Cpu::ror::<{ AddressingMode::ZeroPage }>,
        Cpu::rra::<{ AddressingMode::ZeroPage }>,
        Cpu::pla,
        Cpu::adc::<{ AddressingMode::Immediate }>,
        Cpu::ror_accumulator,
        Cpu::arr::<{ AddressingMode::Immediate }>,
        Cpu::jmp::<{ AddressingMode::Indirect }>,
        Cpu::adc::<{ AddressingMode::Absolute }>,
        Cpu::ror::<{ AddressingMode::Absolute }>,
        Cpu::rra::<{ AddressingMode::Absolute }>,
        Cpu::bvs,
        Cpu::adc::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::rra::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::adc::<{ AddressingMode::ZeroPageX }>,
        Cpu::ror::<{ AddressingMode::ZeroPageX }>,
        Cpu::rra::<{ AddressingMode::ZeroPageX }>,
        Cpu::sei,
        Cpu::adc::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::rra::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::adc::<{ AddressingMode::AbsoluteX }>,
        Cpu::ror::<{ AddressingMode::AbsoluteX }>,
        Cpu::rra::<{ AddressingMode::AbsoluteX }>,
        Cpu::nop::<{ AddressingMode::Immediate }>,
        Cpu::sta::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::Immediate }>,
        Cpu::sax::<{ AddressingMode::IndexedIndirect }>,
        Cpu::sty::<{ AddressingMode::ZeroPage }>,
        Cpu::sta::<{ AddressingMode::ZeroPage }>,
        Cpu::stx::<{ AddressingMode::ZeroPage }>,
        Cpu::sax::<{ AddressingMode::ZeroPage }>,
        Cpu::dey,
        Cpu::nop::<{ AddressingMode::Immediate }>,
        Cpu::txa,
        Cpu::ane::<{ AddressingMode::Immediate }>,
        Cpu::sty::<{ AddressingMode::Absolute }>,
        Cpu::sta::<{ AddressingMode::Absolute }>,
        Cpu::stx::<{ AddressingMode::Absolute }>,
        Cpu::sax::<{ AddressingMode::Absolute }>,
        Cpu::bcc,
        Cpu::sta::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::sha::<{ AddressingMode::AbsoluteY }>,
        Cpu::sty::<{ AddressingMode::ZeroPageX }>,
        Cpu::sta::<{ AddressingMode::ZeroPageX }>,
        Cpu::stx::<{ AddressingMode::ZeroPageY }>,
        Cpu::sax::<{ AddressingMode::ZeroPageY }>,
        Cpu::tya,
        Cpu::sta::<{ AddressingMode::AbsoluteY }>,
        Cpu::txs,
        Cpu::tas::<{ AddressingMode::AbsoluteY }>,
        Cpu::shy::<{ AddressingMode::AbsoluteX }>,
        Cpu::sta::<{ AddressingMode::AbsoluteX }>,
        Cpu::shx::<{ AddressingMode::AbsoluteY }>,
        Cpu::sha::<{ AddressingMode::IndirectIndexed }>,
        Cpu::ldy::<{ AddressingMode::Immediate }>,
        Cpu::lda::<{ AddressingMode::IndexedIndirect }>,
        Cpu::ldx::<{ AddressingMode::Immediate }>,
        Cpu::lax::<{ AddressingMode::IndexedIndirect }>,
        Cpu::ldy::<{ AddressingMode::ZeroPage }>,
        Cpu::lda::<{ AddressingMode::ZeroPage }>,
        Cpu::ldx::<{ AddressingMode::ZeroPage }>,
        Cpu::lax::<{ AddressingMode::ZeroPage }>,
        Cpu::tay,
        Cpu::lda::<{ AddressingMode::Immediate }>,
        Cpu::tax,
        Cpu::lxa::<{ AddressingMode::Immediate }>,
        Cpu::ldy::<{ AddressingMode::Absolute }>,
        Cpu::lda::<{ AddressingMode::Absolute }>,
        Cpu::ldx::<{ AddressingMode::Absolute }>,
        Cpu::lax::<{ AddressingMode::Absolute }>,
        Cpu::bcs,
        Cpu::lda::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::lax::<{ AddressingMode::IndirectIndexed }>,
        Cpu::ldy::<{ AddressingMode::ZeroPageX }>,
        Cpu::lda::<{ AddressingMode::ZeroPageX }>,
        Cpu::ldx::<{ AddressingMode::ZeroPageY }>,
        Cpu::lax::<{ AddressingMode::ZeroPageY }>,
        Cpu::clv,
        Cpu::lda::<{ AddressingMode::AbsoluteY }>,
        Cpu::tsx,
        Cpu::las::<{ AddressingMode::AbsoluteY }>,
        Cpu::ldy::<{ AddressingMode::AbsoluteX }>,
        Cpu::lda::<{ AddressingMode::AbsoluteX }>,
        Cpu::ldx::<{ AddressingMode::AbsoluteY }>,
        Cpu::lax::<{ AddressingMode::AbsoluteY }>,
        Cpu::cpy::<{ AddressingMode::Immediate }>,
        Cpu::cmp::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::Immediate }>,
        Cpu::dcp::<{ AddressingMode::IndexedIndirect }>,
        Cpu::cpy::<{ AddressingMode::ZeroPage }>,
        Cpu::cmp::<{ AddressingMode::ZeroPage }>,
        Cpu::dec::<{ AddressingMode::ZeroPage }>,
        Cpu::dcp::<{ AddressingMode::ZeroPage }>,
        Cpu::iny,
        Cpu::cmp::<{ AddressingMode::Immediate }>,
        Cpu::dex,
        Cpu::sbx::<{ AddressingMode::Immediate }>,
        Cpu::cpy::<{ AddressingMode::Absolute }>,
        Cpu::cmp::<{ AddressingMode::Absolute }>,
        Cpu::dec::<{ AddressingMode::Absolute }>,
        Cpu::dcp::<{ AddressingMode::Absolute }>,
        Cpu::bne,
        Cpu::cmp::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::dcp::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::cmp::<{ AddressingMode::ZeroPageX }>,
        Cpu::dec::<{ AddressingMode::ZeroPageX }>,
        Cpu::dcp::<{ AddressingMode::ZeroPageX }>,
        Cpu::cld,
        Cpu::cmp::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::dcp::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::cmp::<{ AddressingMode::AbsoluteX }>,
        Cpu::dec::<{ AddressingMode::AbsoluteX }>,
        Cpu::dcp::<{ AddressingMode::AbsoluteX }>,
        Cpu::cpx::<{ AddressingMode::Immediate }>,
        Cpu::sbc::<{ AddressingMode::IndexedIndirect }>,
        Cpu::nop::<{ AddressingMode::Immediate }>,
        Cpu::isb::<{ AddressingMode::IndexedIndirect }>,
        Cpu::cpx::<{ AddressingMode::ZeroPage }>,
        Cpu::sbc::<{ AddressingMode::ZeroPage }>,
        Cpu::inc::<{ AddressingMode::ZeroPage }>,
        Cpu::isb::<{ AddressingMode::ZeroPage }>,
        Cpu::inx,
        Cpu::sbc::<{ AddressingMode::Immediate }>,
        Cpu::nop_implied,
        Cpu::sbc::<{ AddressingMode::Immediate }>,
        Cpu::cpx::<{ AddressingMode::Absolute }>,
        Cpu::sbc::<{ AddressingMode::Absolute }>,
        Cpu::inc::<{ AddressingMode::Absolute }>,
        Cpu::isb::<{ AddressingMode::Absolute }>,
        Cpu::beq,
        Cpu::sbc::<{ AddressingMode::IndirectIndexed }>,
        Cpu::jam,
        Cpu::isb::<{ AddressingMode::IndirectIndexed }>,
        Cpu::nop::<{ AddressingMode::ZeroPageX }>,
        Cpu::sbc::<{ AddressingMode::ZeroPageX }>,
        Cpu::inc::<{ AddressingMode::ZeroPageX }>,
        Cpu::isb::<{ AddressingMode::ZeroPageX }>,
        Cpu::sed,
        Cpu::sbc::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop_implied,
        Cpu::isb::<{ AddressingMode::AbsoluteY }>,
        Cpu::nop::<{ AddressingMode::AbsoluteX }>,
        Cpu::sbc::<{ AddressingMode::AbsoluteX }>,
        Cpu::inc::<{ AddressingMode::AbsoluteX }>,
        Cpu::isb::<{ AddressingMode::AbsoluteX }>,
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
                Cpu::brk::<{ Interrupt::Rst }>
            } else if self.prev_need_nmi {
                self.need_nmi = false;
                Cpu::brk::<{ Interrupt::Nmi }>
            } else {
                Cpu::brk::<{ Interrupt::Irq }>
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

    fn effective_address<const MODE: AddressingMode, const WRITE: bool>(
        &mut self,
    ) -> u16 {
        match MODE {
            AddressingMode::Absolute => self.consume_word(),
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                let index = if MODE == AddressingMode::AbsoluteX {
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
                if page_cross || WRITE {
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
                if did_cross_page || WRITE {
                    self.read_byte((high as u16) << 8 | low as u16);
                }

                effective_address
            }
            AddressingMode::ZeroPage => self.consume_byte() as u16,
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
                let index = if MODE == AddressingMode::ZeroPageX {
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
    fn adc<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        let value = self.read_byte(effective_address);
        self.add(value);
    }

    fn anc<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.a &= self.read_byte(effective_address);

        self.p.set_c(self.a & 0x80 != 0);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn and<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.a &= self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn alr<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.a &= self.read_byte(effective_address);
        let carry = self.a & 0x01 != 0;
        self.a = self.a.wrapping_shr(1);

        self.p.set_c(carry);
        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ane<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        // Treat ANE as a NOP since it's unstable.
        self.read_byte(effective_address);
    }

    fn arr<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
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

    fn asl<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn bit<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

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

    fn brk<const KIND: Interrupt>(&mut self) {
        self.read_byte(self.pc);
        if KIND == Interrupt::Brk {
            self.pc += 1;
        }

        if KIND == Interrupt::Rst {
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
        } else {
            self.push((self.pc >> 8) as u8);
            self.push(self.pc as u8);
            self.push(self.p.with_b(KIND == Interrupt::Brk).into());
        }

        // TODO: Implement interrupt hijacking.
        // TODO: Should NMI not set the I flag?
        self.p.set_i(true);
        let vector = match KIND {
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

    fn cmp<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.a, value);
    }

    fn cpx<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.x, value);
    }

    fn cpy<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.y, value);
    }

    fn dcp<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        value = value.wrapping_sub(1);
        self.write_byte(effective_address, value);
        self.compare(self.a, value);
    }

    fn dec<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn eor<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.a ^= self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn inc<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn isb<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn jmp<const MODE: AddressingMode>(&mut self) {
        self.pc = self.effective_address::<MODE, false>();
    }

    fn jsr(&mut self) {
        let pcl = self.consume_byte();
        self.peek();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        let pch = self.consume_byte();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn las<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        self.a = self.read_byte(effective_address) & self.s;
        self.x = self.a;
        self.s = self.a;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lax<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        let value = self.read_byte(effective_address);
        self.a = value;
        self.x = value;

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn lda<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.a = self.read_byte(effective_address);

        self.p.set_z(self.a == 0);
        self.p.set_n(self.a & 0x80 != 0);
    }

    fn ldx<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.x = self.read_byte(effective_address);

        self.p.set_z(self.x == 0);
        self.p.set_n(self.x & 0x80 != 0);
    }

    fn ldy<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
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

    fn lsr<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        self.write_byte(effective_address, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn lxa<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

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

    fn nop<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
        self.read_byte(effective_address);
    }

    fn ora<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();
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

    fn rla<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn rol<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn ror<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
        let mut value = self.read_byte(effective_address);
        self.write_byte(effective_address, value);
        let carry = value & 0x01 != 0;
        value = (self.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        self.write_byte(effective_address, value);

        self.p.set_c(carry);
        self.p.set_z(value == 0);
        self.p.set_n(value & 0x80 != 0);
    }

    fn rra<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn sax<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        self.write_byte(effective_address, self.a & self.x);
    }

    fn sbc<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

        // If we reformulate subtraction as addition, then we can use the same
        // logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = self.read_byte(effective_address) ^ 0xff;
        self.add(value);
    }

    fn sbx<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, false>();

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

    fn sha<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

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

    fn shx<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.x as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn shy<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.y & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.y as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn slo<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn sre<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();
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

    fn sta<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        self.write_byte(effective_address, self.a);
    }

    fn stx<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        self.write_byte(effective_address, self.x);
    }

    fn sty<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

        self.write_byte(effective_address, self.y);
    }

    fn tas<const MODE: AddressingMode>(&mut self) {
        let effective_address = self.effective_address::<MODE, true>();

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

impl Cpu<DuNesBus> {
    pub fn disassemble(&self) -> BTreeMap<u16, String> {
        let mut disasm = BTreeMap::new();

        let mut pc = 0;

        while pc < 0xffff {
            let opcode = self.bus.read_unclocked(pc);
            let (name, addresing_mode) = &INSTRUCTIONS[opcode as usize];
            let pc_to_insert = pc;
            pc += 1;

            let string_stuff = match addresing_mode {
                AddressingMode::Absolute => {
                    let low = self.bus.read_unclocked(pc);
                    let high = self.bus.read_unclocked(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}")
                }
                AddressingMode::AbsoluteX => {
                    let low = self.bus.read_unclocked(pc);
                    let high = self.bus.read_unclocked(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}, X")
                }
                AddressingMode::AbsoluteY => {
                    let low = self.bus.read_unclocked(pc);
                    let high = self.bus.read_unclocked(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("${ea:04X}, Y")
                }
                AddressingMode::Accumulator => "A".to_string(),
                AddressingMode::Immediate => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("#${value:02X}")
                }
                AddressingMode::Indirect => {
                    let low = self.bus.read_unclocked(pc);
                    let high = self.bus.read_unclocked(pc + 1);
                    pc = pc + 2;
                    let ea = (high as u16) << 8 | low as u16;
                    format!("(${ea:04X})")
                }
                AddressingMode::Implied => "".to_string(),
                AddressingMode::IndexedIndirect => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("(${value:02X}, X)")
                }
                AddressingMode::IndirectIndexed => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("(${value:02X}), Y")
                }
                AddressingMode::Relative => {
                    let value = self.bus.read_unclocked(pc) as i8 as u16;
                    pc += 1;
                    let target = pc.wrapping_add(value);
                    format!("${target:04X}")
                }
                AddressingMode::ZeroPage => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("${value:02X}")
                }
                AddressingMode::ZeroPageX => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("${value:02X}, X")
                }
                AddressingMode::ZeroPageY => {
                    let value = self.bus.read_unclocked(pc);
                    pc += 1;
                    format!("${value:02X}, Y")
                }
            };

            disasm.insert(pc_to_insert, format!("{} {}", name, string_stuff));
        }

        disasm
    }
}
