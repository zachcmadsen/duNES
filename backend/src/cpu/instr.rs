use crate::{bus, cpu::next_byte, Emu};

macro_rules! set_zn {
    ($emu:expr, $field:ident) => {
        $emu.cpu.p.set_z($emu.cpu.$field == 0);
        $emu.cpu.p.set_n(($emu.cpu.$field & 0x80) != 0);
    };
}

pub fn adc(emu: &mut Emu) {
    let val = bus::read_byte(emu, emu.cpu.addr);
    add(emu, val);
}

pub fn adc_imm(emu: &mut Emu) {
    imm(emu);
    adc(emu);
}

pub fn beq(emu: &mut Emu) {
    branch(emu, emu.cpu.p.z());
}

pub fn inc(emu: &mut Emu) {
    emu.cpu.data = emu.cpu.data.wrapping_add(1);
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    set_zn!(emu, data);
}

pub fn lda(emu: &mut Emu) {
    emu.cpu.a = bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, a);
}

pub fn lda_imm(emu: &mut Emu) {
    imm(emu);
    lda(emu);
}

pub fn ldx(emu: &mut Emu) {
    emu.cpu.x = bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, x);
}

pub fn ldx_imm(emu: &mut Emu) {
    imm(emu);
    ldx(emu);
}

pub fn ldy(emu: &mut Emu) {
    emu.cpu.y = bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, y);
}

pub fn ldy_imm(emu: &mut Emu) {
    imm(emu);
    ldy(emu);
}

pub fn lsr(emu: &mut Emu) {
    let carry = emu.cpu.data & 0x01;
    emu.cpu.data >>= 1;
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry != 0);
    set_zn!(emu, data);
}

pub fn lsr_accumulator(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    let carry = emu.cpu.a & 0x01;
    emu.cpu.a >>= 1;
    emu.cpu.p.set_c(carry != 0);
    set_zn!(emu, a);
}

pub fn sta(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.a);
}

pub fn stx(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.x);
}

pub fn sty(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.y);
}

fn add(emu: &mut Emu, val: u8) {
    let prev_a = emu.cpu.a;
    let res = (emu.cpu.a as u16)
        .wrapping_add(val as u16)
        .wrapping_add(emu.cpu.p.c() as u16);
    emu.cpu.a = res as u8;
    emu.cpu.p.set_c(res > 0xFF);
    emu.cpu.p.set_v(((prev_a ^ emu.cpu.a) & (val ^ emu.cpu.a) & 0x80) != 0);
    set_zn!(emu, a);
}

fn branch(emu: &mut Emu, cond: bool) {
    emu.cpu.data = next_byte(emu);
    if !cond {
        emu.cpu.cyc += 2;
    }
}

fn imm(emu: &mut Emu) {
    emu.cpu.addr = emu.cpu.pc;
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
}