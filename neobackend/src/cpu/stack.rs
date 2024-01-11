use crate::{cpu::bus, emu::Emu};

const STACK_BASE_ADDR: u16 = 0x0100;

pub fn peek(emu: &mut Emu) {
    bus::read(emu, STACK_BASE_ADDR + emu.cpu.s as u16);
}

pub fn pop(emu: &mut Emu) -> u8 {
    emu.cpu.s = emu.cpu.s.wrapping_add(1);
    bus::read(emu, STACK_BASE_ADDR + emu.cpu.s as u16)
}

pub fn push(emu: &mut Emu, data: u8) {
    bus::write(emu, STACK_BASE_ADDR + emu.cpu.s as u16, data);
    emu.cpu.s = emu.cpu.s.wrapping_sub(1);
}
