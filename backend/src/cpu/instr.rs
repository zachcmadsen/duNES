use crate::{bus, cpu::next_byte, Emu, Status};

const STACK_BASE_ADDR: u16 = 0x0100;

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

pub fn and(emu: &mut Emu) {
    emu.cpu.a &= bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, a);
}

pub fn and_imm(emu: &mut Emu) {
    imm(emu);
    and(emu);
}

pub fn asl(emu: &mut Emu) {
    let carry = emu.cpu.data & 0x80 != 0;
    emu.cpu.data <<= 1;
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry);
    set_zn!(emu, data);
}

pub fn asl_a(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    let carry = emu.cpu.a & 0x80 != 0;
    emu.cpu.a <<= 1;
    emu.cpu.p.set_c(carry);
    set_zn!(emu, a);
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
    let data = bus::read_byte(emu, emu.cpu.addr);
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

pub fn bvc(emu: &mut Emu) {
    branch(emu, !emu.cpu.p.v());
}

pub fn bvs(emu: &mut Emu) {
    branch(emu, emu.cpu.p.v());
}

pub fn clc(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_c(false);
}

pub fn cld(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_d(false);
}

pub fn cli(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_i(false);
}

pub fn clv(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_v(false);
}

pub fn cmp(emu: &mut Emu) {
    compare(emu, emu.cpu.a);
}

pub fn cmp_imm(emu: &mut Emu) {
    imm(emu);
    cmp(emu);
}

pub fn cpx(emu: &mut Emu) {
    compare(emu, emu.cpu.x);
}

pub fn cpx_imm(emu: &mut Emu) {
    imm(emu);
    cpx(emu);
}

pub fn cpy(emu: &mut Emu) {
    compare(emu, emu.cpu.y);
}

pub fn cpy_imm(emu: &mut Emu) {
    imm(emu);
    cpy(emu);
}

pub fn dec(emu: &mut Emu) {
    emu.cpu.data = emu.cpu.data.wrapping_sub(1);
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    set_zn!(emu, data);
}

pub fn dex(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.x.wrapping_sub(1);
    set_zn!(emu, x);
}

pub fn dey(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.y.wrapping_sub(1);
    set_zn!(emu, y);
}

pub fn eor(emu: &mut Emu) {
    emu.cpu.a ^= bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, a);
}

pub fn eor_imm(emu: &mut Emu) {
    imm(emu);
    eor(emu);
}

pub fn inc(emu: &mut Emu) {
    emu.cpu.data = emu.cpu.data.wrapping_add(1);
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    set_zn!(emu, data);
}

pub fn inx(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.x.wrapping_add(1);
    set_zn!(emu, x);
}

pub fn iny(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.y.wrapping_add(1);
    set_zn!(emu, y);
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
    let carry = emu.cpu.data & 0x01 != 0;
    emu.cpu.data >>= 1;
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry);
    set_zn!(emu, data);
}

pub fn lsr_a(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    let carry = emu.cpu.a & 0x01 != 0;
    emu.cpu.a >>= 1;
    emu.cpu.p.set_c(carry);
    set_zn!(emu, a);
}

pub fn nop(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
}

pub fn nop_imm(emu: &mut Emu) {
    imm(emu);
    nop(emu);
}

pub fn nop_imp(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
}

pub fn ora(emu: &mut Emu) {
    emu.cpu.a |= bus::read_byte(emu, emu.cpu.addr);
    set_zn!(emu, a);
}

pub fn ora_imm(emu: &mut Emu) {
    imm(emu);
    ora(emu);
}

pub fn pha(emu: &mut Emu) {
    push(emu, emu.cpu.a);
}

pub fn php(emu: &mut Emu) {
    push(emu, emu.cpu.p.with_b(true).with_u(true).0);
}

pub fn pla(emu: &mut Emu) {
    emu.cpu.a = pop(emu);
    set_zn!(emu, a);
}

pub fn plp(emu: &mut Emu) {
    emu.cpu.p = Status(pop(emu)).with_b(emu.cpu.p.b()).with_u(emu.cpu.p.u());
}

pub fn rol(emu: &mut Emu) {
    let carry = emu.cpu.data & 0x80 != 0;
    emu.cpu.data = ((emu.cpu.data << 1) & 0xfe) | emu.cpu.p.c() as u8;
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry);
    set_zn!(emu, data);
}

pub fn rol_a(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    let carry = emu.cpu.a & 0x80 != 0;
    emu.cpu.a = ((emu.cpu.a << 1) & 0xfe) | emu.cpu.p.c() as u8;
    emu.cpu.p.set_c(carry);
    set_zn!(emu, a);
}

pub fn ror(emu: &mut Emu) {
    let carry = emu.cpu.data & 0x01 != 0;
    emu.cpu.data = (emu.cpu.p.c() as u8) << 7 | ((emu.cpu.data >> 1) & 0x7f);
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry);
    set_zn!(emu, data);
}

pub fn ror_a(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    let carry = emu.cpu.a & 0x01 != 0;
    emu.cpu.a = (emu.cpu.p.c() as u8) << 7 | ((emu.cpu.a >> 1) & 0x7f);
    emu.cpu.p.set_c(carry);
    set_zn!(emu, a);
}

pub fn sbc(emu: &mut Emu) {
    let val = bus::read_byte(emu, emu.cpu.addr);
    add(emu, val ^ 0xFF);
}

pub fn sbc_imm(emu: &mut Emu) {
    imm(emu);
    sbc(emu);
}

pub fn sec(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_c(true);
}

pub fn sed(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_d(true);
}

pub fn sei(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.p.set_i(true);
}

pub fn slo(emu: &mut Emu) {
    let carry = emu.cpu.data & 0x80 != 0;
    emu.cpu.data <<= 1;
    emu.cpu.a |= emu.cpu.data;
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    emu.cpu.p.set_c(carry);
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

pub fn tax(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.a;
    set_zn!(emu, x);
}

pub fn tay(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.y = emu.cpu.a;
    set_zn!(emu, y);
}

pub fn tsx(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.x = emu.cpu.s;
    set_zn!(emu, x);
}

pub fn txa(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.a = emu.cpu.x;
    set_zn!(emu, a);
}

pub fn txs(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.s = emu.cpu.x;
}

pub fn tya(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.a = emu.cpu.y;
    set_zn!(emu, a);
}

pub fn read_pc_and_inc_pc(emu: &mut Emu) {
    next_byte(emu);
}

pub fn push_pcl(emu: &mut Emu) {
    push(emu, emu.cpu.pc as u8);
}

pub fn push_pch(emu: &mut Emu) {
    push(emu, (emu.cpu.pc >> 8) as u8);
}

pub fn push_p(emu: &mut Emu) {
    push(emu, emu.cpu.p.with_b(true).0);
    emu.cpu.p.set_i(true);
}

pub fn pull_p(emu: &mut Emu) {
    emu.cpu.p = Status(pop(emu)).with_b(emu.cpu.p.b()).with_u(emu.cpu.p.u());
}

pub fn pull_pcl(emu: &mut Emu) {
    emu.cpu.pc = pop(emu) as u16;
}

pub fn pull_pch(emu: &mut Emu) {
    emu.cpu.pc |= (pop(emu) as u16) << 8;
}

pub fn set_pcl<const V: u16>(emu: &mut Emu) {
    emu.cpu.pc = bus::read_byte(emu, V) as u16;
}

pub fn set_pch<const V: u16>(emu: &mut Emu) {
    emu.cpu.pc |= (bus::read_byte(emu, V + 1) as u16) << 8;
}

pub fn read_pc(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.pc);
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

fn compare(emu: &mut Emu, reg: u8) {
    let data = bus::read_byte(emu, emu.cpu.addr);
    let (res, carry) = reg.overflowing_sub(data);
    emu.cpu.p.set_c(!carry);
    emu.cpu.p.set_z(res == 0);
    emu.cpu.p.set_n(res & 0x80 != 0);
}

fn imm(emu: &mut Emu) {
    emu.cpu.addr = emu.cpu.pc;
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
}

fn push(emu: &mut Emu, data: u8) {
    bus::write_byte(emu, STACK_BASE_ADDR + emu.cpu.s as u16, data);
    emu.cpu.s = emu.cpu.s.wrapping_sub(1);
}

fn pop(emu: &mut Emu) -> u8 {
    emu.cpu.s = emu.cpu.s.wrapping_add(1);
    bus::read_byte(emu, STACK_BASE_ADDR + emu.cpu.s as u16)
}

pub fn peek(emu: &mut Emu) {
    bus::read_byte(emu, STACK_BASE_ADDR + emu.cpu.s as u16);
}
