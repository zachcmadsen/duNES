use proc_bitfield::Bit;

use crate::{
    cpu::{self, bus, stack, Status, IRQ_VECTOR},
    emu::Emu,
};

pub fn adc(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    add(emu, data);
}

pub fn alr(emu: &mut Emu) {
    emu.cpu.a &= bus::read(emu, emu.cpu.addr);
    let carry = emu.cpu.a.bit::<0>();
    emu.cpu.a >>= 1;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn anc(emu: &mut Emu) {
    emu.cpu.a &= bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_c(emu.cpu.a.bit::<7>());
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn and(emu: &mut Emu) {
    emu.cpu.a &= bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn ane(_: &mut Emu) {
    todo!();
}

pub fn arr(emu: &mut Emu) {
    emu.cpu.a &= bus::read(emu, emu.cpu.addr);
    emu.cpu.a = (emu.cpu.a >> 1).set_bit::<7>(emu.cpu.p.c());
    emu.cpu.p.set_c(emu.cpu.a.bit::<6>());
    emu.cpu.p.set_v(emu.cpu.p.c() ^ emu.cpu.a.bit::<5>());
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn asl(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<7>();
    data <<= 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(data);
}

pub fn asl_a(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    let carry = emu.cpu.a.bit::<7>();
    emu.cpu.a <<= 1;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn bcc(emu: &mut Emu) {
    branch(emu, !emu.cpu.p.c());
}

pub fn bcs(emu: &mut Emu) {
    branch(emu, emu.cpu.p.c());
}

pub fn beq(emu: &mut Emu) {
    branch(emu, emu.cpu.p.z());
}

pub fn bit(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    let status = Status(data);
    emu.cpu.p.set_z(emu.cpu.a & data == 0);
    emu.cpu.p.set_v(status.v());
    emu.cpu.p.set_n(status.n());
}

pub fn bmi(emu: &mut Emu) {
    branch(emu, emu.cpu.p.n());
}

pub fn bne(emu: &mut Emu) {
    branch(emu, !emu.cpu.p.z());
}

pub fn bpl(emu: &mut Emu) {
    branch(emu, !emu.cpu.p.n());
}

pub fn brk(emu: &mut Emu) {
    cpu::eat_byte(emu);
    stack::push(emu, (emu.cpu.pc >> 8) as u8);
    stack::push(emu, emu.cpu.pc as u8);
    stack::push(emu, emu.cpu.p.with_b(true).0);
    emu.cpu.p.set_i(true);
    let pcl = bus::read(emu, IRQ_VECTOR);
    let pch = bus::read(emu, IRQ_VECTOR + 1);
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
}

pub fn bvc(emu: &mut Emu) {
    branch(emu, !emu.cpu.p.v());
}

pub fn bvs(emu: &mut Emu) {
    branch(emu, emu.cpu.p.v());
}

pub fn clc(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_c(false);
}

pub fn cld(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_d(false);
}

pub fn cli(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_i(false);
}

pub fn clv(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_v(false);
}

pub fn cmp(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    compare(emu, emu.cpu.a, data);
}

pub fn cpx(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    compare(emu, emu.cpu.x, data);
}

pub fn cpy(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    compare(emu, emu.cpu.y, data);
}

pub fn dcp(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    data = data.wrapping_sub(1);
    bus::write(emu, emu.cpu.addr, data);
    compare(emu, emu.cpu.a, data);
}

pub fn dec(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    data = data.wrapping_sub(1);
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_z_and_n(data);
}

pub fn dex(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.x.wrapping_sub(1);
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn dey(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.y.wrapping_sub(1);
    emu.cpu.p.set_z_and_n(emu.cpu.y);
}

pub fn eor(emu: &mut Emu) {
    emu.cpu.a ^= bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn inc(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    data = data.wrapping_add(1);
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_z_and_n(data);
}

pub fn inx(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.x.wrapping_add(1);
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn iny(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.y.wrapping_add(1);
    emu.cpu.p.set_z_and_n(emu.cpu.y);
}

pub fn isc(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    data = data.wrapping_add(1);
    bus::write(emu, emu.cpu.addr, data);
    add(emu, data ^ 0xFF);
}

pub fn jmp(emu: &mut Emu) {
    emu.cpu.pc = emu.cpu.addr;
}

pub fn jsr(emu: &mut Emu) {
    let pcl = cpu::eat_byte(emu);
    stack::peek(emu);
    stack::push(emu, (emu.cpu.pc >> 8) as u8);
    stack::push(emu, emu.cpu.pc as u8);
    let pch = cpu::eat_byte(emu);
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
}

pub fn las(emu: &mut Emu) {
    emu.cpu.a = bus::read(emu, emu.cpu.addr) & emu.cpu.s;
    emu.cpu.x = emu.cpu.a;
    emu.cpu.s = emu.cpu.a;
    emu.cpu.p.set_z_and_n(emu.cpu.s);
}

pub fn lax(emu: &mut Emu) {
    emu.cpu.a = bus::read(emu, emu.cpu.addr);
    emu.cpu.x = emu.cpu.a;
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn lda(emu: &mut Emu) {
    emu.cpu.a = bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn ldx(emu: &mut Emu) {
    emu.cpu.x = bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn ldy(emu: &mut Emu) {
    emu.cpu.y = bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.y);
}

pub fn lsr(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<0>();
    data >>= 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(data);
}

pub fn lsr_a(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    let carry = emu.cpu.a.bit::<0>();
    emu.cpu.a >>= 1;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn lxa(_: &mut Emu) {
    todo!("lxa");
}

pub fn nop(emu: &mut Emu) {
    bus::read(emu, emu.cpu.addr);
}

pub fn ora(emu: &mut Emu) {
    emu.cpu.a |= bus::read(emu, emu.cpu.addr);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn pha(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::push(emu, emu.cpu.a);
}

pub fn php(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::push(emu, emu.cpu.p.with_b(true).with_u(true).0);
}

pub fn pla(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::peek(emu);
    emu.cpu.a = stack::pop(emu);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn plp(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::peek(emu);
    emu.cpu.p =
        Status(stack::pop(emu)).with_b(emu.cpu.p.b()).with_u(emu.cpu.p.u());
}

pub fn rla(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<7>();
    data = emu.cpu.p.c() as u8 | data << 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.a &= data;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn rol(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<7>();
    data = emu.cpu.p.c() as u8 | data << 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(data);
}

pub fn rol_a(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    let carry = emu.cpu.a.bit::<7>();
    emu.cpu.a = emu.cpu.p.c() as u8 | emu.cpu.a << 1;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn ror(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<0>();
    data = data >> 1 | (emu.cpu.p.c() as u8) << 7;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(data);
}

pub fn ror_a(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    let carry = emu.cpu.a.bit::<0>();
    emu.cpu.a = emu.cpu.a >> 1 | (emu.cpu.p.c() as u8) << 7;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn rra(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<0>();
    data = data >> 1 | (emu.cpu.p.c() as u8) << 7;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.p.set_c(carry);
    add(emu, data);
}

pub fn rti(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::peek(emu);
    emu.cpu.p =
        Status(stack::pop(emu)).with_b(emu.cpu.p.b()).with_u(emu.cpu.p.u());
    let pcl = stack::pop(emu);
    let pch = stack::pop(emu);
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
}

pub fn rts(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    stack::peek(emu);
    let pcl = stack::pop(emu);
    let pch = stack::pop(emu);
    emu.cpu.pc = pcl as u16 | (pch as u16) << 8;
    cpu::eat_byte(emu);
}

pub fn sax(emu: &mut Emu) {
    bus::write(emu, emu.cpu.addr, emu.cpu.a & emu.cpu.x);
}

pub fn sbc(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    add(emu, data ^ 0xFF);
}

pub fn sbx(emu: &mut Emu) {
    let data = bus::read(emu, emu.cpu.addr);
    let (res, carry) = (emu.cpu.a & emu.cpu.x).overflowing_sub(data);
    emu.cpu.x = res;
    emu.cpu.p.set_c(!carry);
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn sec(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_c(true);
}

pub fn sed(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_d(true);
}

pub fn sei(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.p.set_i(true);
}

pub fn sha(emu: &mut Emu) {
    sh_inner(emu, emu.cpu.a & emu.cpu.x);
}

pub fn shx(emu: &mut Emu) {
    sh_inner(emu, emu.cpu.x);
}

pub fn shy(emu: &mut Emu) {
    sh_inner(emu, emu.cpu.y);
}

pub fn slo(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<7>();
    data <<= 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.a |= data;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn sre(emu: &mut Emu) {
    let mut data = bus::read(emu, emu.cpu.addr);
    bus::write(emu, emu.cpu.addr, data);
    let carry = data.bit::<0>();
    data >>= 1;
    bus::write(emu, emu.cpu.addr, data);
    emu.cpu.a ^= data;
    emu.cpu.p.set_c(carry);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn sta(emu: &mut Emu) {
    bus::write(emu, emu.cpu.addr, emu.cpu.a);
}

pub fn stx(emu: &mut Emu) {
    bus::write(emu, emu.cpu.addr, emu.cpu.x);
}

pub fn sty(emu: &mut Emu) {
    bus::write(emu, emu.cpu.addr, emu.cpu.y);
}

pub fn tas(emu: &mut Emu) {
    emu.cpu.s = emu.cpu.a & emu.cpu.x;
    sh_inner(emu, emu.cpu.s);
}

pub fn tax(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.a;
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn tay(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.a;
    emu.cpu.p.set_z_and_n(emu.cpu.y);
}

pub fn tsx(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.s;
    emu.cpu.p.set_z_and_n(emu.cpu.x);
}

pub fn txa(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.a = emu.cpu.x;
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

pub fn txs(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.s = emu.cpu.x;
}

pub fn tya(emu: &mut Emu) {
    bus::read(emu, emu.cpu.pc);
    emu.cpu.a = emu.cpu.y;
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

fn add(emu: &mut Emu, val: u8) {
    let prev_a = emu.cpu.a;
    let sum = (emu.cpu.a as u16)
        .wrapping_add(val as u16)
        .wrapping_add(emu.cpu.p.c() as u16);
    emu.cpu.a = sum as u8;
    emu.cpu.p.set_c(sum > 0xFF);
    emu.cpu.p.set_v(((prev_a ^ emu.cpu.a) & (val ^ emu.cpu.a) & 0x80) != 0);
    emu.cpu.p.set_z_and_n(emu.cpu.a);
}

fn branch(emu: &mut Emu, cond: bool) {
    let offset = cpu::eat_byte(emu) as i8 as i16;
    if cond {
        bus::read(emu, emu.cpu.pc);

        let prev_pc = emu.cpu.pc;
        emu.cpu.pc = emu.cpu.pc.wrapping_add_signed(offset);

        if prev_pc & 0xFF00 != emu.cpu.pc & 0xFF00 {
            bus::read(
                emu,
                (prev_pc as u8).wrapping_add(offset as u8) as u16
                    | (prev_pc & 0xFF00),
            );
        }
    }
}

fn compare(emu: &mut Emu, reg: u8, data: u8) {
    let (res, carry) = reg.overflowing_sub(data);
    emu.cpu.p.set_c(!carry);
    emu.cpu.p.set_z_and_n(res);
}

// https://github.com/TomHarte/ProcessorTests/issues/61
fn sh_inner(emu: &mut Emu, reg: u8) {
    let high = (emu.cpu.addr >> 8) as u8;
    // Increment the high byte if there wasn't a page cross.
    let data = reg & high.wrapping_add(!emu.cpu.carry as u8);
    // Use the value as the high byte of the address if there was a page cross.
    let high = if emu.cpu.carry { data } else { high };
    bus::write(emu, emu.cpu.addr & 0x00FF | (high as u16) << 8, data);
}
