use crate::{
    cpu::{self, bus},
    emu::Emu,
};

pub fn abs(emu: &mut Emu) {
    emu.cpu.addr = cpu::eat_word(emu);
}

pub fn abx<const WRITE: bool>(emu: &mut Emu) {
    let (low, carry) = cpu::eat_byte(emu).overflowing_add(emu.cpu.x);
    let high = cpu::eat_byte(emu);
    emu.cpu.addr = low as u16 | (high.wrapping_add(carry as u8) as u16) << 8;
    emu.cpu.carry = carry;

    if WRITE || carry {
        bus::read(emu, low as u16 | (high as u16) << 8);
    }
}

pub fn aby<const WRITE: bool>(emu: &mut Emu) {
    let (low, carry) = cpu::eat_byte(emu).overflowing_add(emu.cpu.y);
    let high = cpu::eat_byte(emu);
    emu.cpu.addr = low as u16 | (high.wrapping_add(carry as u8) as u16) << 8;
    emu.cpu.carry = carry;

    if WRITE || carry {
        bus::read(emu, low as u16 | (high as u16) << 8);
    }
}

pub fn imm(emu: &mut Emu) {
    emu.cpu.addr = emu.cpu.pc;
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
}

// Only used for NOP.
pub fn imp(emu: &mut Emu) {
    emu.cpu.addr = emu.cpu.pc;
}

pub fn ind(emu: &mut Emu) {
    let ptr_low = cpu::eat_byte(emu);
    let ptr_high = cpu::eat_byte(emu);
    let low = bus::read(emu, ptr_low as u16 | (ptr_high as u16) << 8);
    let high = bus::read(
        emu,
        ptr_low.wrapping_add(1) as u16 | (ptr_high as u16) << 8,
    );
    emu.cpu.addr = low as u16 | (high as u16) << 8;
}

pub fn idx(emu: &mut Emu) {
    let ptr = cpu::eat_byte(emu);
    bus::read(emu, ptr as u16);
    let ptr = ptr.wrapping_add(emu.cpu.x);
    let low = bus::read(emu, ptr as u16);
    let high = bus::read(emu, ptr.wrapping_add(1) as u16);
    emu.cpu.addr = low as u16 | (high as u16) << 8;
}

pub fn idy<const WRITE: bool>(emu: &mut Emu) {
    let ptr = cpu::eat_byte(emu);
    let (low, carry) = bus::read(emu, ptr as u16).overflowing_add(emu.cpu.y);
    let high = bus::read(emu, ptr.wrapping_add(1) as u16);
    emu.cpu.addr = low as u16 | (high.wrapping_add(carry as u8) as u16) << 8;
    emu.cpu.carry = carry;

    if WRITE || carry {
        bus::read(emu, low as u16 | (high as u16) << 8);
    }
}

pub fn zpg(emu: &mut Emu) {
    emu.cpu.addr = cpu::eat_byte(emu) as u16;
}

pub fn zpx(emu: &mut Emu) {
    let addr = cpu::eat_byte(emu);
    bus::read(emu, addr as u16);
    emu.cpu.addr = addr.wrapping_add(emu.cpu.x) as u16;
}

pub fn zpy(emu: &mut Emu) {
    let addr = cpu::eat_byte(emu);
    bus::read(emu, addr as u16);
    emu.cpu.addr = addr.wrapping_add(emu.cpu.y) as u16;
}
