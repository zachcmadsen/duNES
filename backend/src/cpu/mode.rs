use crate::{bus, cpu::next_byte, Emu};

macro_rules! abs {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use abs;

macro_rules! abs_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use abs_rmw;

macro_rules! abx_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high_and_add_index::<
                true,
                true,
            >,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use abx_r;

macro_rules! abx_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high_and_add_index::<
                true,
                false,
            >,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use abx_w;

macro_rules! abx_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high_and_add_index::<
                true,
                false,
            >,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use abx_rmw;

macro_rules! aby_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high_and_add_index::<
                false,
                true,
            >,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use aby_r;

macro_rules! aby_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_pc_and_set_addr_high_and_add_index::<
                false,
                false,
            >,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use aby_w;

macro_rules! idx {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $crate::cpu::mode::read_addr_and_inc_addr_low_and_set_addr_high,
            $crate::cpu::mode::read_addr_and_set_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use idx;

macro_rules! idy_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_inc_addr_low_and_set_addr_high,
            $crate::cpu::mode::read_addr_and_add_y_and_set_addr::<true>,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use idy_r;

macro_rules! idy_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_inc_addr_low_and_set_addr_high,
            $crate::cpu::mode::read_addr_and_add_y_and_set_addr::<false>,
            $crate::cpu::mode::read_addr_and_fix_addr_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use idy_w;

macro_rules! rel {
    ($f:ident) => {
        &[
            $f,
            $crate::cpu::mode::read_pc_and_add_data,
            $crate::cpu::mode::read_pc_and_fix_pch,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use rel;

macro_rules! zpg {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use zpg;

macro_rules! zpg_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use zpg_rmw;

macro_rules! zpx {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use zpx;

macro_rules! zpx_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use zpx_rmw;

macro_rules! zpy {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_addr_low,
            $crate::cpu::mode::read_addr_and_add_index::<false>,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}
pub(crate) use zpy;

pub fn read_pc_and_set_opc(emu: &mut Emu) {
    emu.cpu.opc = next_byte(emu);
    emu.cpu.cyc = -1;
}

pub fn read_pc_and_set_addr_low(emu: &mut Emu) {
    emu.cpu.addr = next_byte(emu) as u16;
}

pub fn read_pc_and_set_addr_high(emu: &mut Emu) {
    emu.cpu.addr |= (next_byte(emu) as u16) << 8;
}

pub fn read_addr_and_add_index<const X: bool>(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    let index = if X { emu.cpu.x } else { emu.cpu.y };
    emu.cpu.addr = (emu.cpu.addr as u8).wrapping_add(index) as u16
}

pub fn read_pc_and_set_addr_high_and_add_index<
    const X: bool,
    const R: bool,
>(
    emu: &mut Emu,
) {
    let high = next_byte(emu);
    let index = if X { emu.cpu.x } else { emu.cpu.y };
    let (low, carry) = (emu.cpu.addr as u8).overflowing_add(index);
    emu.cpu.addr = low as u16 | (high as u16) << 8;
    emu.cpu.carry = carry;

    if R && !carry {
        emu.cpu.cyc += 1;
    }
}

pub fn read_addr_and_fix_addr_high(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    if emu.cpu.carry {
        let high = ((emu.cpu.addr & 0xFF00) >> 8) as u8;
        emu.cpu.addr &= 0x00FF;
        emu.cpu.addr |= (high.wrapping_add(1) as u16) << 8;
    }
}

pub fn read_addr_and_inc_addr_low_and_set_addr_high(emu: &mut Emu) {
    let low = bus::read_byte(emu, emu.cpu.addr);
    // TODO(zach): Explain why we don't incremet the page if ptr wraps.
    let ptr = emu.cpu.addr as u8;
    emu.cpu.addr = ptr.wrapping_add(1) as u16;
    emu.cpu.addr |= (low as u16) << 8;
}

pub fn read_addr_and_add_y_and_set_addr<const R: bool>(emu: &mut Emu) {
    let ptr = emu.cpu.addr as u8;
    let high = bus::read_byte(emu, ptr as u16);
    let low = (emu.cpu.addr >> 8) as u8;
    let (low, carry) = low.overflowing_add(emu.cpu.y);
    emu.cpu.addr = low as u16 | (high as u16) << 8;
    emu.cpu.carry = carry;

    if R && !carry {
        emu.cpu.cyc += 1;
    }
}

pub fn read_addr_and_set_addr(emu: &mut Emu) {
    let ptr = emu.cpu.addr as u8;
    let high = bus::read_byte(emu, ptr as u16);
    let low = (emu.cpu.addr >> 8) as u8;
    emu.cpu.addr = low as u16 | (high as u16) << 8;
}

pub fn read_addr_and_set_data(emu: &mut Emu) {
    emu.cpu.data = bus::read_byte(emu, emu.cpu.addr);
}

pub fn write_data_to_addr(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
}

pub fn read_pc_and_add_data(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.addr = emu.cpu.pc;
    emu.cpu.pc = emu.cpu.pc.wrapping_add(emu.cpu.data as i8 as u16);

    if emu.cpu.addr & 0xFF00 == emu.cpu.pc & 0xFF00 {
        emu.cpu.cyc += 1;
    }
}

pub fn read_pc_and_fix_pch(emu: &mut Emu) {
    bus::read_byte(
        emu,
        (emu.cpu.addr & 0xFF00)
            | (emu.cpu.addr as u8).wrapping_add(emu.cpu.data) as u16,
    );
}
