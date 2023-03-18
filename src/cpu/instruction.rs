macro_rules! adc {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        $cpu.add(value);
    };
}

macro_rules! alr {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a &= $cpu.read_byte($cpu.pins.address);
        let carry = $cpu.a & 0x01 != 0;
        $cpu.a = $cpu.a.wrapping_shr(1);

        $cpu.p.set_c(carry);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! anc {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a &= $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_c($cpu.a & 0x80 != 0);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! and {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a &= $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! ane {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        // Treat ANE as a NOP since it's unstable.
        $cpu.read_byte($cpu.pins.address);
    };
}

macro_rules! arr {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a &= $cpu.read_byte($cpu.pins.address);
        $cpu.a = ($cpu.p.c() as u8) << 7 | $cpu.a.wrapping_shr(1);

        // TODO: Explain how the carry and overflow flag are set.
        $cpu.p.set_c($cpu.a & 0x40 != 0);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p
            .set_v((($cpu.p.c() as u8) ^ (($cpu.a >> 5) & 0x01)) != 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! asl {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x80 != 0;
        value <<= 1;
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_c(carry);
        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! bit {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.a & value == 0);
        $cpu.p.set_v(Status::from(value).v());
        $cpu.p.set_n(Status::from(value).n());
    };
}

macro_rules! cmp {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        $cpu.compare($cpu.a, value);
    };
}

macro_rules! cpx {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        $cpu.compare($cpu.x, value);
    };
}

macro_rules! cpy {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        $cpu.compare($cpu.y, value);
    };
}

macro_rules! dcp {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        value = value.wrapping_sub(1);
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.compare($cpu.a, value);
    };
}

macro_rules! dec {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        value = value.wrapping_sub(1);
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! eor {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a ^= $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! inc {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        value = value.wrapping_add(1);
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! isb {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        value = value.wrapping_add(1);
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.add(value ^ 0xff);
    };
}

macro_rules! jmp {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.pc = $cpu.pins.address;
    };
}

macro_rules! las {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a = $cpu.read_byte($cpu.pins.address) & $cpu.s;
        $cpu.x = $cpu.a;
        $cpu.s = $cpu.a;

        $cpu.p.set_z($cpu.x == 0);
        $cpu.p.set_n($cpu.x & 0x80 != 0);
    };
}

macro_rules! lax {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        $cpu.a = value;
        $cpu.x = value;

        $cpu.p.set_z($cpu.x == 0);
        $cpu.p.set_n($cpu.x & 0x80 != 0);
    };
}

macro_rules! lda {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a = $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! ldx {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.x = $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.x == 0);
        $cpu.p.set_n($cpu.x & 0x80 != 0);
    };
}

macro_rules! ldy {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.y = $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.y == 0);
        $cpu.p.set_n($cpu.y & 0x80 != 0);
    };
}

macro_rules! lsr {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_c(carry);
        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! lxa {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        // This instruction should perform a bitwise AND between a constant and
        // the operand before storing the result. The constant is unreliable
        // though. To remove uncertainty, we have the constant always be 0xff,
        // removing the need for the bitwise AND.
        $cpu.a = $cpu.read_byte($cpu.pins.address);
        $cpu.x = $cpu.a;

        $cpu.p.set_z($cpu.x == 0);
        $cpu.p.set_n($cpu.x & 0x80 != 0);
    };
}

macro_rules! nop {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.read_byte($cpu.pins.address);
    };
}

macro_rules! ora {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.a |= $cpu.read_byte($cpu.pins.address);

        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! rla {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | $cpu.p.c() as u8;
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.a &= value;

        $cpu.p.set_c(carry);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! rol {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x80 != 0;
        value = ((value << 1) & 0xfe) | $cpu.p.c() as u8;
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_c(carry);
        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! ror {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x01 != 0;
        value = ($cpu.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        $cpu.write_byte($cpu.pins.address, value);

        $cpu.p.set_c(carry);
        $cpu.p.set_z(value == 0);
        $cpu.p.set_n(value & 0x80 != 0);
    };
}

macro_rules! rra {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x01 != 0;
        value = ($cpu.p.c() as u8) << 7 | ((value >> 1) & 0x7f);
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.p.set_c(carry);
        $cpu.add(value);
    };
}

macro_rules! sax {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.write_byte($cpu.pins.address, $cpu.a & $cpu.x);
    };
}

macro_rules! sbc {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        // If we reformulate subtraction as addition, then we can use the same
        // logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = $cpu.read_byte($cpu.pins.address) ^ 0xff;
        $cpu.add(value);
    };
}

macro_rules! sbx {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let value = $cpu.read_byte($cpu.pins.address);
        let carry = ($cpu.a & $cpu.x) >= value;
        $cpu.x = ($cpu.a & $cpu.x).wrapping_sub(value);

        $cpu.p.set_c(carry);
        $cpu.p.set_z($cpu.x == 0);
        $cpu.p.set_n($cpu.x & 0x80 != 0);
    };
}

macro_rules! sha {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let high_byte = ($cpu.pins.address & 0xff00) >> 8;
        let low_byte = $cpu.pins.address & 0x00ff;
        let value = $cpu.a & $cpu.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        $cpu.write_byte(
            (($cpu.a as u16 & $cpu.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
    };
}

macro_rules! shx {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let high_byte = ($cpu.pins.address & 0xff00) >> 8;
        let low_byte = $cpu.pins.address & 0x00ff;
        let value = $cpu.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        $cpu.write_byte(
            (($cpu.x as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    };
}

macro_rules! shy {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let high_byte = ($cpu.pins.address & 0xff00) >> 8;
        let low_byte = $cpu.pins.address & 0x00ff;
        let value = $cpu.y & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        $cpu.write_byte(
            (($cpu.y as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    };
}

macro_rules! slo {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x80 != 0;
        value <<= 1;
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.a |= value;

        $cpu.p.set_c(carry);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! sre {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let mut value = $cpu.read_byte($cpu.pins.address);
        $cpu.write_byte($cpu.pins.address, value);
        let carry = value & 0x01 != 0;
        value >>= 1;
        $cpu.write_byte($cpu.pins.address, value);
        $cpu.a ^= value;

        $cpu.p.set_c(carry);
        $cpu.p.set_z($cpu.a == 0);
        $cpu.p.set_n($cpu.a & 0x80 != 0);
    };
}

macro_rules! sta {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.write_byte($cpu.pins.address, $cpu.a);
    };
}

macro_rules! stx {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.write_byte($cpu.pins.address, $cpu.x);
    };
}

macro_rules! sty {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        $cpu.write_byte($cpu.pins.address, $cpu.y);
    };
}

macro_rules! tas {
    ($cpu:ident, $mode:tt) => {
        $mode!($cpu);

        let high_byte = ($cpu.pins.address & 0xff00) >> 8;
        let low_byte = $cpu.pins.address & 0x00ff;
        let value = $cpu.a & $cpu.x & (high_byte as u8).wrapping_add(1);
        $cpu.s = $cpu.a & $cpu.x;

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        $cpu.write_byte(
            (($cpu.a as u16 & $cpu.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
    };
}

pub(crate) use adc;
pub(crate) use alr;
pub(crate) use anc;
pub(crate) use and;
pub(crate) use ane;
pub(crate) use arr;
pub(crate) use asl;
pub(crate) use bit;
pub(crate) use cmp;
pub(crate) use cpx;
pub(crate) use cpy;
pub(crate) use dcp;
pub(crate) use dec;
pub(crate) use eor;
pub(crate) use inc;
pub(crate) use isb;
pub(crate) use jmp;
pub(crate) use las;
pub(crate) use lax;
pub(crate) use lda;
pub(crate) use ldx;
pub(crate) use ldy;
pub(crate) use lsr;
pub(crate) use lxa;
pub(crate) use nop;
pub(crate) use ora;
pub(crate) use rla;
pub(crate) use rol;
pub(crate) use ror;
pub(crate) use rra;
pub(crate) use sax;
pub(crate) use sbc;
pub(crate) use sbx;
pub(crate) use sha;
pub(crate) use shx;
pub(crate) use shy;
pub(crate) use slo;
pub(crate) use sre;
pub(crate) use sta;
pub(crate) use stx;
pub(crate) use sty;
pub(crate) use tas;
