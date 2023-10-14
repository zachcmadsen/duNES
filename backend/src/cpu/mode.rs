use crate::{bus, cpu::next_byte, Emu};

pub fn read_pc_and_set_opc(emu: &mut Emu) {
    emu.cpu.opc = next_byte(emu);
    emu.cpu.cyc = -1;
}

pub fn read_pc_and_set_low(emu: &mut Emu) {
    emu.cpu.addr = next_byte(emu) as u16;
}

pub fn read_pc_and_set_high(emu: &mut Emu) {
    emu.cpu.addr |= (next_byte(emu) as u16) << 8;
}

pub fn read_pc_and_set_high_and_tpc(emu: &mut Emu) {
    emu.cpu.pc = (next_byte(emu) as u16) << 8 | emu.cpu.addr;
}

pub fn read_addr_and_add_index<const X: bool>(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    let index = if X { emu.cpu.x } else { emu.cpu.y };
    emu.cpu.addr = (emu.cpu.addr as u8).wrapping_add(index) as u16
}

pub fn read_pc_and_add_index_to_low_and_set_high<
    const X: bool,
    const R: bool,
>(
    emu: &mut Emu,
) {
    let index = if X { emu.cpu.x } else { emu.cpu.y };
    let (low, carry) = (emu.cpu.addr as u8).overflowing_add(index);
    let high = next_byte(emu);
    emu.cpu.addr = low as u16 | (high as u16) << 8;
    emu.cpu.carry = carry;

    if R && !carry {
        emu.cpu.cyc += 1;
    }
}

pub fn read_addr_and_opt_fix_high(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    if emu.cpu.carry {
        let high = ((emu.cpu.addr & 0xFF00) >> 8) as u8;
        emu.cpu.addr &= 0x00FF;
        emu.cpu.addr |= (high.wrapping_add(1) as u16) << 8;
    }
}

pub fn read_addr_and_add_y_to_low_and_set_high<const R: bool>(emu: &mut Emu) {
    let low = emu.cpu.data;
    let (low, carry) = low.overflowing_add(emu.cpu.y);
    let high =
        bus::read_byte(emu, (emu.cpu.addr as u8).wrapping_add(1) as u16);
    emu.cpu.addr = low as u16 | (high as u16) << 8;
    emu.cpu.carry = carry;

    if R && !carry {
        emu.cpu.cyc += 1;
    }
}

pub fn read_addr_and_set_high(emu: &mut Emu) {
    let low = emu.cpu.data;
    let high = bus::read_byte(
        emu,
        (emu.cpu.addr & 0xFF00) | (emu.cpu.addr as u8).wrapping_add(1) as u16,
    );
    emu.cpu.addr = low as u16 | (high as u16) << 8;
}

pub fn read_addr_and_set_data(emu: &mut Emu) {
    emu.cpu.data = bus::read_byte(emu, emu.cpu.addr);
}

pub fn read_addr_and_set_pc(emu: &mut Emu) {
    let pcl = emu.cpu.data;
    let pch = bus::read_byte(
        emu,
        (emu.cpu.addr & 0xFF00) | (emu.cpu.addr as u8).wrapping_add(1) as u16,
    );
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
}

pub fn write_data_to_addr(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
}

pub fn read_pc_and_add_data_to_pc(emu: &mut Emu) {
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
