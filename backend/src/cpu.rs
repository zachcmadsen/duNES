use proc_bitfield::bitfield;

use crate::{bus, Emu};

macro_rules! update_zn {
    ($cpu:expr, $reg:ident) => {
        $cpu.p.set_z($cpu.$reg == 0);
        $cpu.p.set_n(($cpu.$reg & 0x80) != 0);
    };
}

bitfield! {
    pub struct Status(pub u8) {
        c: bool @ 0,
        z: bool @ 1,
        i: bool @ 2,
        d: bool @ 3,
        b: bool @ 4,
        u: bool @ 5,
        v: bool @ 6,
        n: bool @ 7,
    }
}

const X: bool = true;
const Y: bool = false;
const READ: bool = true;
const WRITE: bool = false;

macro_rules! abs {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! abs_rmw {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high,
            read_addr_and_set_data,
            write_data_to_addr,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_r {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high_and_add_index::<X, READ>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_w {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high_and_add_index::<X, WRITE>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_rmw {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high_and_add_index::<X, WRITE>,
            read_addr_and_fix_addr_high,
            read_addr_and_set_data,
            write_data_to_addr,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! aby_r {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high_and_add_index::<Y, READ>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! aby_w {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_pc_and_set_addr_high_and_add_index::<Y, WRITE>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! idx {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_add_index::<X>,
            read_addr_and_inc_addr_low_and_set_addr_high,
            read_addr_and_set_addr,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! idy_r {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_inc_addr_low_and_set_addr_high,
            read_addr_and_add_y_and_set_addr::<READ>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! idy_w {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_inc_addr_low_and_set_addr_high,
            read_addr_and_add_y_and_set_addr::<WRITE>,
            read_addr_and_fix_addr_high,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpg {
    ($f:ident) => {
        &[read_pc_and_set_addr_low, $f, read_pc_and_set_opc]
    };
}

macro_rules! zpg_rmw {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_set_data,
            write_data_to_addr,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpx {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_add_index::<X>,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpx_rmw {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_add_index::<X>,
            read_addr_and_set_data,
            write_data_to_addr,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpy {
    ($f:ident) => {
        &[
            read_pc_and_set_addr_low,
            read_addr_and_add_index::<Y>,
            $f,
            read_pc_and_set_opc,
        ]
    };
}

static OPC_LUT: [&[fn(&mut Emu)]; 0x100] = [
    &[],                             // 0x00
    &[],                             // 0x01
    &[],                             // 0x02
    &[],                             // 0x03
    &[],                             // 0x04
    &[],                             // 0x05
    &[],                             // 0x06
    &[],                             // 0x07
    &[],                             // 0x08
    &[],                             // 0x09
    &[],                             // 0x0A
    &[],                             // 0x0B
    &[],                             // 0x0C
    &[],                             // 0x0D
    &[],                             // 0x0E
    &[],                             // 0x0F
    &[],                             // 0x10
    &[],                             // 0x11
    &[],                             // 0x12
    &[],                             // 0x13
    &[],                             // 0x14
    &[],                             // 0x15
    &[],                             // 0x16
    &[],                             // 0x17
    &[],                             // 0x18
    &[],                             // 0x19
    &[],                             // 0x1A
    &[],                             // 0x1B
    &[],                             // 0x1C
    &[],                             // 0x1D
    &[],                             // 0x1E
    &[],                             // 0x1F
    &[],                             // 0x20
    &[],                             // 0x21
    &[],                             // 0x22
    &[],                             // 0x23
    &[],                             // 0x24
    &[],                             // 0x25
    &[],                             // 0x26
    &[],                             // 0x27
    &[],                             // 0x28
    &[],                             // 0x29
    &[],                             // 0x2A
    &[],                             // 0x2B
    &[],                             // 0x2C
    &[],                             // 0x2D
    &[],                             // 0x2E
    &[],                             // 0x2F
    &[],                             // 0x30
    &[],                             // 0x31
    &[],                             // 0x32
    &[],                             // 0x33
    &[],                             // 0x34
    &[],                             // 0x35
    &[],                             // 0x36
    &[],                             // 0x37
    &[],                             // 0x38
    &[],                             // 0x39
    &[],                             // 0x3A
    &[],                             // 0x3B
    &[],                             // 0x3C
    &[],                             // 0x3D
    &[],                             // 0x3E
    &[],                             // 0x3F
    &[],                             // 0x40
    &[],                             // 0x41
    &[],                             // 0x42
    &[],                             // 0x43
    &[],                             // 0x44
    &[],                             // 0x45
    &[],                             // 0x46
    &[],                             // 0x47
    &[],                             // 0x48
    &[],                             // 0x49
    &[],                             // 0x4A
    &[],                             // 0x4B
    &[],                             // 0x4C
    &[],                             // 0x4D
    &[],                             // 0x4E
    &[],                             // 0x4F
    &[],                             // 0x50
    &[],                             // 0x51
    &[],                             // 0x52
    &[],                             // 0x53
    &[],                             // 0x54
    &[],                             // 0x55
    &[],                             // 0x56
    &[],                             // 0x57
    &[],                             // 0x58
    &[],                             // 0x59
    &[],                             // 0x5A
    &[],                             // 0x5B
    &[],                             // 0x5C
    &[],                             // 0x5D
    &[],                             // 0x5E
    &[],                             // 0x5F
    &[],                             // 0x60
    &[],                             // 0x61
    &[],                             // 0x62
    &[],                             // 0x63
    &[],                             // 0x64
    &[],                             // 0x65
    &[],                             // 0x66
    &[],                             // 0x67
    &[],                             // 0x68
    &[],                             // 0x69
    &[],                             // 0x6A
    &[],                             // 0x6B
    &[],                             // 0x6C
    &[],                             // 0x6D
    &[],                             // 0x6E
    &[],                             // 0x6F
    &[],                             // 0x70
    &[],                             // 0x71
    &[],                             // 0x72
    &[],                             // 0x73
    &[],                             // 0x74
    &[],                             // 0x75
    &[],                             // 0x76
    &[],                             // 0x77
    &[],                             // 0x78
    &[],                             // 0x79
    &[],                             // 0x7A
    &[],                             // 0x7B
    &[],                             // 0x7C
    &[],                             // 0x7D
    &[],                             // 0x7E
    &[],                             // 0x7F
    &[],                             // 0x80
    idx!(sta),                       // 0x81
    &[],                             // 0x82
    &[],                             // 0x83
    zpg!(sty),                       // 0x84
    zpg!(sta),                       // 0x85
    zpg!(stx),                       // 0x86
    &[],                             // 0x87
    &[],                             // 0x88
    &[],                             // 0x89
    &[],                             // 0x8A
    &[],                             // 0x8B
    abs!(sty),                       // 0x8C
    abs!(sta),                       // 0x8D
    abs!(stx),                       // 0x8E
    &[],                             // 0x8F
    &[],                             // 0x90
    idy_w!(sta),                     // 0x91
    &[],                             // 0x92
    &[],                             // 0x93
    zpx!(sty),                       // 0x94
    zpx!(sta),                       // 0x95
    zpy!(stx),                       // 0x96
    &[],                             // 0x97
    &[],                             // 0x98
    aby_w!(sta),                     // 0x99
    &[],                             // 0x9A
    &[],                             // 0x9B
    &[],                             // 0x9C
    abx_w!(sta),                     // 0x9D
    &[],                             // 0x9E
    &[],                             // 0x9F
    &[ldy_imm, read_pc_and_set_opc], // 0xA0
    idx!(lda),                       // 0xA1
    &[ldx_imm, read_pc_and_set_opc], // 0xA2
    &[],                             // 0xA3
    zpg!(ldy),                       // 0xA4
    zpg!(lda),                       // 0xA5
    zpg!(ldx),                       // 0xA6
    &[],                             // 0xA7
    &[],                             // 0xA8
    &[lda_imm, read_pc_and_set_opc], // 0xA9
    &[],                             // 0xAA
    &[],                             // 0xAB
    abs!(ldy),                       // 0xAC
    abs!(lda),                       // 0xAD
    abs!(ldx),                       // 0xAE
    &[],                             // 0xAF
    &[],                             // 0xB0
    idy_r!(lda),                     // 0xB1
    &[],                             // 0xB2
    &[],                             // 0xB3
    zpx!(ldy),                       // 0xB4
    zpx!(lda),                       // 0xB5
    zpy!(ldx),                       // 0xB6
    &[],                             // 0xB7
    &[],                             // 0xB8
    aby_r!(lda),                     // 0xB9
    &[],                             // 0xBA
    &[],                             // 0xBB
    abx_r!(ldy),                     // 0xBC
    abx_r!(lda),                     // 0xBD
    aby_r!(ldx),                     // 0xBE
    &[],                             // 0xBF
    &[],                             // 0xC0
    &[],                             // 0xC1
    &[],                             // 0xC2
    &[],                             // 0xC3
    &[],                             // 0xC4
    &[],                             // 0xC5
    &[],                             // 0xC6
    &[],                             // 0xC7
    &[],                             // 0xC8
    &[],                             // 0xC9
    &[],                             // 0xCA
    &[],                             // 0xCB
    &[],                             // 0xCC
    &[],                             // 0xCD
    &[],                             // 0xCE
    &[],                             // 0xCF
    &[],                             // 0xD0
    &[],                             // 0xD1
    &[],                             // 0xD2
    &[],                             // 0xD3
    &[],                             // 0xD4
    &[],                             // 0xD5
    &[],                             // 0xD6
    &[],                             // 0xD7
    &[],                             // 0xD8
    &[],                             // 0xD9
    &[],                             // 0xDA
    &[],                             // 0xDB
    &[],                             // 0xDC
    &[],                             // 0xDD
    &[],                             // 0xDE
    &[],                             // 0xDF
    &[],                             // 0xE0
    &[],                             // 0xE1
    &[],                             // 0xE2
    &[],                             // 0xE3
    &[],                             // 0xE4
    &[],                             // 0xE5
    zpg_rmw!(inc),                   // 0xE6
    &[],                             // 0xE7
    &[],                             // 0xE8
    &[],                             // 0xE9
    &[],                             // 0xEA
    &[],                             // 0xEB
    &[],                             // 0xEC
    &[],                             // 0xED
    abs_rmw!(inc),                   // 0xEE
    &[],                             // 0xEF
    &[],                             // 0xF0
    &[],                             // 0xF1
    &[],                             // 0xF2
    &[],                             // 0xF3
    &[],                             // 0xF4
    &[],                             // 0xF5
    zpx_rmw!(inc),                   // 0xF6
    &[],                             // 0xF7
    &[],                             // 0xF8
    &[],                             // 0xF9
    &[],                             // 0xFA
    &[],                             // 0xFB
    &[],                             // 0xFC
    &[],                             // 0xFD
    abx_rmw!(inc),                   // 0xFE
    &[],                             // 0xFF
];

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: Status,

    opc: u8,
    cyc: i8,
    addr: u16,
    carry: bool,
    data: u8,
}

impl Cpu {
    /// Constructs a new `Cpu` in a power up state.
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            p: Status(0x34),

            // TODO(zach): Explain the initial values of `opc` and `cyc`.
            opc: 0xA5,
            cyc: 1,
            addr: 0,
            carry: false,
            data: 0,
        }
    }
}

pub fn step(emu: &mut Emu) {
    emu.cpu.cyc += 1;
    OPC_LUT[emu.cpu.opc as usize][emu.cpu.cyc as usize](emu);
}

fn next_byte(emu: &mut Emu) -> u8 {
    let byte = bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
    byte
}

fn read_pc_and_set_opc(emu: &mut Emu) {
    emu.cpu.opc = next_byte(emu);
    emu.cpu.cyc = -1;
}

fn read_pc_and_set_addr_low(emu: &mut Emu) {
    emu.cpu.addr = next_byte(emu) as u16;
}

fn read_pc_and_set_addr_high(emu: &mut Emu) {
    emu.cpu.addr |= (next_byte(emu) as u16) << 8;
}

fn read_addr_and_add_index<const X: bool>(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    let index = if X { emu.cpu.x } else { emu.cpu.y };
    emu.cpu.addr = (emu.cpu.addr as u8).wrapping_add(index) as u16
}

fn read_pc_and_set_addr_high_and_add_index<const X: bool, const R: bool>(
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

fn read_addr_and_fix_addr_high(emu: &mut Emu) {
    bus::read_byte(emu, emu.cpu.addr);
    if emu.cpu.carry {
        let high = ((emu.cpu.addr & 0xFF00) >> 8) as u8;
        emu.cpu.addr &= 0x00FF;
        emu.cpu.addr |= (high.wrapping_add(1) as u16) << 8;
    }
}

fn read_addr_and_inc_addr_low_and_set_addr_high(emu: &mut Emu) {
    let low = bus::read_byte(emu, emu.cpu.addr);
    // TODO(zach): Explain why we don't incremet the page if ptr wraps.
    let ptr = emu.cpu.addr as u8;
    emu.cpu.addr = ptr.wrapping_add(1) as u16;
    emu.cpu.addr |= (low as u16) << 8;
}

fn read_addr_and_add_y_and_set_addr<const R: bool>(emu: &mut Emu) {
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

fn read_addr_and_set_addr(emu: &mut Emu) {
    let ptr = emu.cpu.addr as u8;
    let high = bus::read_byte(emu, ptr as u16);
    let low = (emu.cpu.addr >> 8) as u8;
    emu.cpu.addr = low as u16 | (high as u16) << 8;
}

fn read_addr_and_set_data(emu: &mut Emu) {
    emu.cpu.data = bus::read_byte(emu, emu.cpu.addr);
}

fn write_data_to_addr(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
}

fn imm(emu: &mut Emu) {
    emu.cpu.addr = emu.cpu.pc;
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
}

fn inc(emu: &mut Emu) {
    emu.cpu.data = emu.cpu.data.wrapping_add(1);
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.data);
    update_zn!(emu.cpu, data);
}

fn lda(emu: &mut Emu) {
    emu.cpu.a = bus::read_byte(emu, emu.cpu.addr);
    update_zn!(emu.cpu, a);
}

fn lda_imm(emu: &mut Emu) {
    imm(emu);
    lda(emu);
}

fn ldx(emu: &mut Emu) {
    emu.cpu.x = bus::read_byte(emu, emu.cpu.addr);
    update_zn!(emu.cpu, x);
}

fn ldx_imm(emu: &mut Emu) {
    imm(emu);
    ldx(emu);
}

fn ldy(emu: &mut Emu) {
    emu.cpu.y = bus::read_byte(emu, emu.cpu.addr);
    update_zn!(emu.cpu, y);
}

fn ldy_imm(emu: &mut Emu) {
    imm(emu);
    ldy(emu);
}

fn sta(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.a);
}

fn stx(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.x);
}

fn sty(emu: &mut Emu) {
    bus::write_byte(emu, emu.cpu.addr, emu.cpu.y);
}
