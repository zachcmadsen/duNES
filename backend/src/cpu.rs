#[cfg(not(test))]
#[path = "cpu/bus.rs"]
mod bus;
mod instruction;
mod mode;
mod stack;

#[cfg(test)]
#[path = "cpu/tests/bus.rs"]
mod bus;
#[cfg(test)]
mod tests;

use proc_bitfield::bitfield;

use crate::{cpu::bus::Bus, emu::Emu};

const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

bitfield! {
    #[derive(Clone, Copy)]
    struct Status(u8) {
        /// The carry flag.
        c: bool @ 0,
        /// The zero flag.
        z: bool @ 1,
        /// The interrupt disable flag.
        i: bool @ 2,
        d: bool @ 3,
        // TODO: Can the B and U flags be read/write only?
        /// The B flag.
        b: bool @ 4,
        u: bool @ 5,
        /// The overflow flag.
        v: bool @ 6,
        /// The negative flag.
        n: bool @ 7,
    }
}

impl Status {
    fn set_z_and_n(&mut self, data: u8) {
        self.set_z(data == 0);
        self.set_n(data & 0x80 != 0);
    }
}

pub struct Cpu {
    /// The accumulator.
    a: u8,
    /// The X register.
    x: u8,
    /// The Y register.
    y: u8,
    /// The program counter.
    pc: u16,
    /// The stack pointer.
    s: u8,
    /// The status register.
    p: Status,

    bus: Bus,

    addr: u16,
    carry: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            p: Status(0x34),

            bus: Bus::new(),

            addr: 0,
            carry: false,
        }
    }
}

/// Steps the CPU by one instruction.
pub fn step(emu: &mut Emu) {
    const R: bool = false;
    const W: bool = true;

    let opc = eat_byte(emu);
    #[rustfmt::skip]
    match opc {
        0x00 => {                      instruction::brk(emu); }
        0x01 => { mode::idx(emu);      instruction::ora(emu); }
        0x03 => { mode::idx(emu);      instruction::slo(emu); }
        0x04 => { mode::zpg(emu);      instruction::nop(emu); }
        0x05 => { mode::zpg(emu);      instruction::ora(emu); }
        0x06 => { mode::zpg(emu);      instruction::asl(emu); }
        0x07 => { mode::zpg(emu);      instruction::slo(emu); }
        0x08 => {                      instruction::php(emu); }
        0x09 => { mode::imm(emu);      instruction::ora(emu); }
        0x0A => {                      instruction::asl_a(emu); }
        0x0B => { mode::imm(emu);      instruction::anc(emu); }
        0x0C => { mode::abs(emu);      instruction::nop(emu); }
        0x0D => { mode::abs(emu);      instruction::ora(emu); }
        0x0E => { mode::abs(emu);      instruction::asl(emu); }
        0x0F => { mode::abs(emu);      instruction::slo(emu); }
        0x10 => {                      instruction::bpl(emu); }
        0x11 => { mode::idy::<R>(emu); instruction::ora(emu); }
        0x13 => { mode::idy::<W>(emu); instruction::slo(emu); }
        0x14 => { mode::zpx(emu);      instruction::nop(emu); }
        0x15 => { mode::zpx(emu);      instruction::ora(emu); }
        0x16 => { mode::zpx(emu);      instruction::asl(emu); }
        0x17 => { mode::zpx(emu);      instruction::slo(emu); }
        0x18 => {                      instruction::clc(emu); }
        0x19 => { mode::aby::<R>(emu); instruction::ora(emu); }
        0x1A => { mode::imp(emu);      instruction::nop(emu); }
        0x1B => { mode::aby::<W>(emu); instruction::slo(emu); }
        0x1C => { mode::abx::<R>(emu); instruction::nop(emu); }
        0x1D => { mode::abx::<R>(emu); instruction::ora(emu); }
        0x1E => { mode::abx::<W>(emu); instruction::asl(emu); }
        0x1F => { mode::abx::<W>(emu); instruction::slo(emu); }
        0x20 => {                      instruction::jsr(emu); }
        0x21 => { mode::idx(emu);      instruction::and(emu); }
        0x23 => { mode::idx(emu);      instruction::rla(emu); }
        0x24 => { mode::zpg(emu);      instruction::bit(emu); }
        0x25 => { mode::zpg(emu);      instruction::and(emu); }
        0x26 => { mode::zpg(emu);      instruction::rol(emu); }
        0x27 => { mode::zpg(emu);      instruction::rla(emu); }
        0x28 => {                      instruction::plp(emu); }
        0x29 => { mode::imm(emu);      instruction::and(emu); }
        0x2A => {                      instruction::rol_a(emu); }
        0x2B => { mode::imm(emu);      instruction::anc(emu); }
        0x2C => { mode::abs(emu);      instruction::bit(emu); }
        0x2D => { mode::abs(emu);      instruction::and(emu); }
        0x2E => { mode::abs(emu);      instruction::rol(emu); }
        0x2F => { mode::abs(emu);      instruction::rla(emu); }
        0x30 => {                      instruction::bmi(emu); }
        0x31 => { mode::idy::<R>(emu); instruction::and(emu); }
        0x33 => { mode::idy::<W>(emu); instruction::rla(emu); }
        0x34 => { mode::zpx(emu);      instruction::nop(emu); }
        0x35 => { mode::zpx(emu);      instruction::and(emu); }
        0x36 => { mode::zpx(emu);      instruction::rol(emu); }
        0x37 => { mode::zpx(emu);      instruction::rla(emu); }
        0x38 => {                      instruction::sec(emu); }
        0x39 => { mode::aby::<R>(emu); instruction::and(emu); }
        0x3A => { mode::imp(emu);      instruction::nop(emu); }
        0x3B => { mode::aby::<W>(emu); instruction::rla(emu); }
        0x3C => { mode::abx::<R>(emu); instruction::nop(emu); }
        0x3D => { mode::abx::<R>(emu); instruction::and(emu); }
        0x3E => { mode::abx::<W>(emu); instruction::rol(emu); }
        0x3F => { mode::abx::<W>(emu); instruction::rla(emu); }
        0x40 => {                      instruction::rti(emu); }
        0x41 => { mode::idx(emu);      instruction::eor(emu); }
        0x43 => { mode::idx(emu);      instruction::sre(emu); }
        0x44 => { mode::zpg(emu);      instruction::nop(emu); }
        0x45 => { mode::zpg(emu);      instruction::eor(emu); }
        0x46 => { mode::zpg(emu);      instruction::lsr(emu); }
        0x47 => { mode::zpg(emu);      instruction::sre(emu); }
        0x48 => {                      instruction::pha(emu); }
        0x49 => { mode::imm(emu);      instruction::eor(emu); }
        0x4A => {                      instruction::lsr_a(emu); }
        0x4B => { mode::imm(emu);      instruction::alr(emu); }
        0x4C => { mode::abs(emu);      instruction::jmp(emu); }
        0x4D => { mode::abs(emu);      instruction::eor(emu); }
        0x4E => { mode::abs(emu);      instruction::lsr(emu); }
        0x4F => { mode::abs(emu);      instruction::sre(emu); }
        0x50 => {                      instruction::bvc(emu); }
        0x51 => { mode::idy::<R>(emu); instruction::eor(emu); }
        0x53 => { mode::idy::<W>(emu); instruction::sre(emu); }
        0x54 => { mode::zpx(emu);      instruction::nop(emu); }
        0x55 => { mode::zpx(emu);      instruction::eor(emu); }
        0x56 => { mode::zpx(emu);      instruction::lsr(emu); }
        0x57 => { mode::zpx(emu);      instruction::sre(emu); }
        0x58 => {                      instruction::cli(emu); }
        0x59 => { mode::aby::<R>(emu); instruction::eor(emu); }
        0x5A => { mode::imp(emu);      instruction::nop(emu); }
        0x5B => { mode::aby::<W>(emu); instruction::sre(emu); }
        0x5C => { mode::abx::<R>(emu); instruction::nop(emu); }
        0x5D => { mode::abx::<R>(emu); instruction::eor(emu); }
        0x5E => { mode::abx::<W>(emu); instruction::lsr(emu); }
        0x5F => { mode::abx::<W>(emu); instruction::sre(emu); }
        0x60 => {                      instruction::rts(emu); }
        0x61 => { mode::idx(emu);      instruction::adc(emu); }
        0x63 => { mode::idx(emu);      instruction::rra(emu); }
        0x64 => { mode::zpg(emu);      instruction::nop(emu); }
        0x65 => { mode::zpg(emu);      instruction::adc(emu); }
        0x66 => { mode::zpg(emu);      instruction::ror(emu); }
        0x67 => { mode::zpg(emu);      instruction::rra(emu); }
        0x68 => {                      instruction::pla(emu); }
        0x69 => { mode::imm(emu);      instruction::adc(emu); }
        0x6A => {                      instruction::ror_a(emu); }
        0x6B => { mode::imm(emu);      instruction::arr(emu); }
        0x6C => { mode::ind(emu);      instruction::jmp(emu); }
        0x6D => { mode::abs(emu);      instruction::adc(emu); }
        0x6E => { mode::abs(emu);      instruction::ror(emu); }
        0x6F => { mode::abs(emu);      instruction::rra(emu); }
        0x70 => {                      instruction::bvs(emu); }
        0x71 => { mode::idy::<R>(emu); instruction::adc(emu); }
        0x73 => { mode::idy::<W>(emu); instruction::rra(emu); }
        0x74 => { mode::zpx(emu);      instruction::nop(emu); }
        0x75 => { mode::zpx(emu);      instruction::adc(emu); }
        0x76 => { mode::zpx(emu);      instruction::ror(emu); }
        0x77 => { mode::zpx(emu);      instruction::rra(emu); }
        0x78 => {                      instruction::sei(emu); }
        0x79 => { mode::aby::<R>(emu); instruction::adc(emu); }
        0x7A => { mode::imp(emu);      instruction::nop(emu); }
        0x7B => { mode::aby::<W>(emu); instruction::rra(emu); }
        0x7C => { mode::abx::<R>(emu); instruction::nop(emu); }
        0x7D => { mode::abx::<R>(emu); instruction::adc(emu); }
        0x7E => { mode::abx::<W>(emu); instruction::ror(emu); }
        0x7F => { mode::abx::<W>(emu); instruction::rra(emu); }
        0x80 => { mode::imm(emu);      instruction::nop(emu); }
        0x81 => { mode::idx(emu);      instruction::sta(emu); }
        0x82 => { mode::imm(emu);      instruction::nop(emu); }
        0x83 => { mode::idx(emu);      instruction::sax(emu); }
        0x84 => { mode::zpg(emu);      instruction::sty(emu); }
        0x85 => { mode::zpg(emu);      instruction::sta(emu); }
        0x86 => { mode::zpg(emu);      instruction::stx(emu); }
        0x87 => { mode::zpg(emu);      instruction::sax(emu); }
        0x88 => {                      instruction::dey(emu); }
        0x89 => { mode::imm(emu);      instruction::nop(emu); }
        0x8A => {                      instruction::txa(emu); }
        0x8B => {                      instruction::ane(emu); }
        0x8C => { mode::abs(emu);      instruction::sty(emu); }
        0x8D => { mode::abs(emu);      instruction::sta(emu); }
        0x8E => { mode::abs(emu);      instruction::stx(emu); }
        0x8F => { mode::abs(emu);      instruction::sax(emu); }
        0x90 => {                      instruction::bcc(emu); }
        0x91 => { mode::idy::<W>(emu); instruction::sta(emu); }
        0x93 => { mode::idy::<W>(emu); instruction::sha(emu); }
        0x94 => { mode::zpx(emu);      instruction::sty(emu); }
        0x95 => { mode::zpx(emu);      instruction::sta(emu); }
        0x96 => { mode::zpy(emu);      instruction::stx(emu); }
        0x97 => { mode::zpy(emu);      instruction::sax(emu); }
        0x98 => {                      instruction::tya(emu); }
        0x99 => { mode::aby::<W>(emu); instruction::sta(emu); }
        0x9A => {                      instruction::txs(emu); }
        0x9B => { mode::aby::<W>(emu); instruction::tas(emu); }
        0x9C => { mode::abx::<W>(emu); instruction::shy(emu); }
        0x9D => { mode::abx::<W>(emu); instruction::sta(emu); }
        0x9E => { mode::aby::<W>(emu); instruction::shx(emu); }
        0x9F => { mode::aby::<W>(emu); instruction::sha(emu); }
        0xA0 => { mode::imm(emu);      instruction::ldy(emu); }
        0xA1 => { mode::idx(emu);      instruction::lda(emu); }
        0xA2 => { mode::imm(emu);      instruction::ldx(emu); }
        0xA3 => { mode::idx(emu);      instruction::lax(emu); }
        0xA4 => { mode::zpg(emu);      instruction::ldy(emu); }
        0xA5 => { mode::zpg(emu);      instruction::lda(emu); }
        0xA6 => { mode::zpg(emu);      instruction::ldx(emu); }
        0xA7 => { mode::zpg(emu);      instruction::lax(emu); }
        0xA8 => {                      instruction::tay(emu); }
        0xA9 => { mode::imm(emu);      instruction::lda(emu); }
        0xAA => {                      instruction::tax(emu); }
        0xAB => { mode::imm(emu);      instruction::lxa(emu); }
        0xAC => { mode::abs(emu);      instruction::ldy(emu); }
        0xAD => { mode::abs(emu);      instruction::lda(emu); }
        0xAE => { mode::abs(emu);      instruction::ldx(emu); }
        0xAF => { mode::abs(emu);      instruction::lax(emu); }
        0xB0 => {                      instruction::bcs(emu); }
        0xB1 => { mode::idy::<R>(emu); instruction::lda(emu); }
        0xB3 => { mode::idy::<R>(emu); instruction::lax(emu); }
        0xB4 => { mode::zpx(emu);      instruction::ldy(emu); }
        0xB5 => { mode::zpx(emu);      instruction::lda(emu); }
        0xB6 => { mode::zpy(emu);      instruction::ldx(emu); }
        0xB7 => { mode::zpy(emu);      instruction::lax(emu); }
        0xB8 => {                      instruction::clv(emu); }
        0xB9 => { mode::aby::<R>(emu); instruction::lda(emu); }
        0xBA => {                      instruction::tsx(emu); }
        0xBB => { mode::aby::<R>(emu); instruction::las(emu); }
        0xBC => { mode::abx::<R>(emu); instruction::ldy(emu); }
        0xBD => { mode::abx::<R>(emu); instruction::lda(emu); }
        0xBE => { mode::aby::<R>(emu); instruction::ldx(emu); }
        0xBF => { mode::aby::<R>(emu); instruction::lax(emu); }
        0xC0 => { mode::imm(emu);      instruction::cpy(emu); }
        0xC1 => { mode::idx(emu);      instruction::cmp(emu); }
        0xC2 => { mode::imm(emu);      instruction::nop(emu); }
        0xC3 => { mode::idx(emu);      instruction::dcp(emu); }
        0xC4 => { mode::zpg(emu);      instruction::cpy(emu); }
        0xC5 => { mode::zpg(emu);      instruction::cmp(emu); }
        0xC6 => { mode::zpg(emu);      instruction::dec(emu); }
        0xC7 => { mode::zpg(emu);      instruction::dcp(emu); }
        0xC8 => {                      instruction::iny(emu); }
        0xC9 => { mode::imm(emu);      instruction::cmp(emu); }
        0xCA => {                      instruction::dex(emu); }
        0xCB => { mode::imm(emu);      instruction::sbx(emu); },
        0xCC => { mode::abs(emu);      instruction::cpy(emu); }
        0xCD => { mode::abs(emu);      instruction::cmp(emu); }
        0xCE => { mode::abs(emu);      instruction::dec(emu); }
        0xCF => { mode::abs(emu);      instruction::dcp(emu); }
        0xD0 => {                      instruction::bne(emu); }
        0xD1 => { mode::idy::<R>(emu); instruction::cmp(emu); }
        0xD3 => { mode::idy::<W>(emu); instruction::dcp(emu); }
        0xD4 => { mode::zpx(emu);      instruction::nop(emu); }
        0xD5 => { mode::zpx(emu);      instruction::cmp(emu); }
        0xD6 => { mode::zpx(emu);      instruction::dec(emu); }
        0xD7 => { mode::zpx(emu);      instruction::dcp(emu); }
        0xD8 => {                      instruction::cld(emu); }
        0xD9 => { mode::aby::<R>(emu); instruction::cmp(emu); }
        0xDA => { mode::imp(emu);      instruction::nop(emu); }
        0xDB => { mode::aby::<W>(emu); instruction::dcp(emu); }
        0xDC => { mode::abx::<R>(emu); instruction::nop(emu); }
        0xDD => { mode::abx::<R>(emu); instruction::cmp(emu); }
        0xDE => { mode::abx::<W>(emu); instruction::dec(emu); }
        0xDF => { mode::abx::<W>(emu); instruction::dcp(emu); }
        0xE0 => { mode::imm(emu);      instruction::cpx(emu); }
        0xE1 => { mode::idx(emu);      instruction::sbc(emu); }
        0xE2 => { mode::imm(emu);      instruction::nop(emu); }
        0xE3 => { mode::idx(emu);      instruction::isc(emu); }
        0xE4 => { mode::zpg(emu);      instruction::cpx(emu); }
        0xE5 => { mode::zpg(emu);      instruction::sbc(emu); }
        0xE6 => { mode::zpg(emu);      instruction::inc(emu); }
        0xE7 => { mode::zpg(emu);      instruction::isc(emu); }
        0xE8 => {                      instruction::inx(emu); }
        0xE9 => { mode::imm(emu);      instruction::sbc(emu); }
        0xEA => { mode::imp(emu);      instruction::nop(emu); }
        0xEB => { mode::imm(emu);      instruction::sbc(emu); }
        0xEC => { mode::abs(emu);      instruction::cpx(emu); }
        0xED => { mode::abs(emu);      instruction::sbc(emu); }
        0xEE => { mode::abs(emu);      instruction::inc(emu); }
        0xEF => { mode::abs(emu);      instruction::isc(emu); }
        0xF0 => {                      instruction::beq(emu); }
        0xF1 => { mode::idy::<R>(emu); instruction::sbc(emu); }
        0xF3 => { mode::idy::<W>(emu); instruction::isc(emu); }
        0xF4 => { mode::zpx(emu);      instruction::nop(emu); }
        0xF5 => { mode::zpx(emu);      instruction::sbc(emu); }
        0xF6 => { mode::zpx(emu);      instruction::inc(emu); }
        0xF7 => { mode::zpx(emu);      instruction::isc(emu); }
        0xF8 => {                      instruction::sed(emu); }
        0xF9 => { mode::aby::<R>(emu); instruction::sbc(emu); }
        0xFA => { mode::imp(emu);      instruction::nop(emu); }
        0xFB => { mode::aby::<W>(emu); instruction::isc(emu); }
        0xFC => { mode::abx::<R>(emu); instruction::nop(emu); }
        0xFD => { mode::abx::<R>(emu); instruction::sbc(emu); }
        0xFE => { mode::abx::<W>(emu); instruction::inc(emu); }
        0xFF => { mode::abx::<W>(emu); instruction::isc(emu); }
        _ => unreachable!("unexpected opcode: 0x{:04X}", opc),
    };
}

pub fn peek(emu: &mut Emu, addr: u16) -> Option<u8> {
    bus::peek(emu, addr)
}

pub fn reset(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::peek(emu);
    emu.cpu.s = emu.cpu.s.wrapping_sub(1);
    stack::peek(emu);
    emu.cpu.s = emu.cpu.s.wrapping_sub(1);
    stack::peek(emu);
    emu.cpu.s = emu.cpu.s.wrapping_sub(1);
    emu.cpu.p.set_i(true);
    let pcl = bus::read(emu, RESET_VECTOR);
    let pch = bus::read(emu, RESET_VECTOR + 1);
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
}

/// Returns the byte at PC and increments PC.
fn eat_byte(emu: &mut Emu) -> u8 {
    let data = bus::read(emu, emu.cpu.pc);
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
    data
}

/// Returns the word at PC and increments PC twice.
fn eat_word(emu: &mut Emu) -> u16 {
    let low = bus::read(emu, emu.cpu.pc);
    let high = bus::read(emu, emu.cpu.pc.wrapping_add(1));
    emu.cpu.pc = emu.cpu.pc.wrapping_add(2);
    low as u16 | (high as u16) << 8
}
