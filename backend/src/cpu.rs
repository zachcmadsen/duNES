mod instr;
mod mode;

use proc_bitfield::bitfield;

use crate::{
    bus,
    cpu::{
        instr::{
            adc, adc_imm, and, and_imm, beq, inc, lda, lda_imm, ldx, ldx_imm,
            ldy, ldy_imm, lsr, lsr_accumulator, sta, stx, sty,
        },
        mode::{
            abs, abs_rmw, abx_r, abx_rmw, abx_w, aby_r, aby_w, idx, idy_r,
            idy_w, read_pc_and_set_opc, rel, zpg, zpg_rmw, zpx, zpx_rmw, zpy,
        },
    },
    Emu,
};

static OPC_LUT: [&[fn(&mut Emu)]; 0x100] = [
    &[],                                     // 0x00
    &[],                                     // 0x01
    &[],                                     // 0x02
    &[],                                     // 0x03
    &[],                                     // 0x04
    &[],                                     // 0x05
    &[],                                     // 0x06
    &[],                                     // 0x07
    &[],                                     // 0x08
    &[],                                     // 0x09
    &[],                                     // 0x0A
    &[],                                     // 0x0B
    &[],                                     // 0x0C
    &[],                                     // 0x0D
    &[],                                     // 0x0E
    &[],                                     // 0x0F
    &[],                                     // 0x10
    &[],                                     // 0x11
    &[],                                     // 0x12
    &[],                                     // 0x13
    &[],                                     // 0x14
    &[],                                     // 0x15
    &[],                                     // 0x16
    &[],                                     // 0x17
    &[],                                     // 0x18
    &[],                                     // 0x19
    &[],                                     // 0x1A
    &[],                                     // 0x1B
    &[],                                     // 0x1C
    &[],                                     // 0x1D
    &[],                                     // 0x1E
    &[],                                     // 0x1F
    &[],                                     // 0x20
    idx!(and),                               // 0x21
    &[],                                     // 0x22
    &[],                                     // 0x23
    &[],                                     // 0x24
    zpg!(and),                               // 0x25
    &[],                                     // 0x26
    &[],                                     // 0x27
    &[],                                     // 0x28
    &[and_imm, read_pc_and_set_opc],         // 0x29
    &[],                                     // 0x2A
    &[],                                     // 0x2B
    &[],                                     // 0x2C
    abs!(and),                               // 0x2D
    &[],                                     // 0x2E
    &[],                                     // 0x2F
    &[],                                     // 0x30
    idy_r!(and),                             // 0x31
    &[],                                     // 0x32
    &[],                                     // 0x33
    &[],                                     // 0x34
    zpx!(and),                               // 0x35
    &[],                                     // 0x36
    &[],                                     // 0x37
    &[],                                     // 0x38
    aby_r!(and),                             // 0x39
    &[],                                     // 0x3A
    &[],                                     // 0x3B
    &[],                                     // 0x3C
    abx_r!(and),                             // 0x3D
    &[],                                     // 0x3E
    &[],                                     // 0x3F
    &[],                                     // 0x40
    &[],                                     // 0x41
    &[],                                     // 0x42
    &[],                                     // 0x43
    &[],                                     // 0x44
    &[],                                     // 0x45
    zpg_rmw!(lsr),                           // 0x46
    &[],                                     // 0x47
    &[],                                     // 0x48
    &[],                                     // 0x49
    &[lsr_accumulator, read_pc_and_set_opc], // 0x4A
    &[],                                     // 0x4B
    &[],                                     // 0x4C
    &[],                                     // 0x4D
    abs_rmw!(lsr),                           // 0x4E
    &[],                                     // 0x4F
    &[],                                     // 0x50
    &[],                                     // 0x51
    &[],                                     // 0x52
    &[],                                     // 0x53
    &[],                                     // 0x54
    &[],                                     // 0x55
    zpx_rmw!(lsr),                           // 0x56
    &[],                                     // 0x57
    &[],                                     // 0x58
    &[],                                     // 0x59
    &[],                                     // 0x5A
    &[],                                     // 0x5B
    &[],                                     // 0x5C
    &[],                                     // 0x5D
    abx_rmw!(lsr),                           // 0x5E
    &[],                                     // 0x5F
    &[],                                     // 0x60
    idx!(adc),                               // 0x61
    &[],                                     // 0x62
    &[],                                     // 0x63
    &[],                                     // 0x64
    zpg!(adc),                               // 0x65
    &[],                                     // 0x66
    &[],                                     // 0x67
    &[],                                     // 0x68
    &[adc_imm, read_pc_and_set_opc],         // 0x69
    &[],                                     // 0x6A
    &[],                                     // 0x6B
    &[],                                     // 0x6C
    abs!(adc),                               // 0x6D
    &[],                                     // 0x6E
    &[],                                     // 0x6F
    &[],                                     // 0x70
    idy_r!(adc),                             // 0x71
    &[],                                     // 0x72
    &[],                                     // 0x73
    &[],                                     // 0x74
    zpx!(adc),                               // 0x75
    &[],                                     // 0x76
    &[],                                     // 0x77
    &[],                                     // 0x78
    aby_r!(adc),                             // 0x79
    &[],                                     // 0x7A
    &[],                                     // 0x7B
    &[],                                     // 0x7C
    abx_r!(adc),                             // 0x7D
    &[],                                     // 0x7E
    &[],                                     // 0x7F
    &[],                                     // 0x80
    idx!(sta),                               // 0x81
    &[],                                     // 0x82
    &[],                                     // 0x83
    zpg!(sty),                               // 0x84
    zpg!(sta),                               // 0x85
    zpg!(stx),                               // 0x86
    &[],                                     // 0x87
    &[],                                     // 0x88
    &[],                                     // 0x89
    &[],                                     // 0x8A
    &[],                                     // 0x8B
    abs!(sty),                               // 0x8C
    abs!(sta),                               // 0x8D
    abs!(stx),                               // 0x8E
    &[],                                     // 0x8F
    &[],                                     // 0x90
    idy_w!(sta),                             // 0x91
    &[],                                     // 0x92
    &[],                                     // 0x93
    zpx!(sty),                               // 0x94
    zpx!(sta),                               // 0x95
    zpy!(stx),                               // 0x96
    &[],                                     // 0x97
    &[],                                     // 0x98
    aby_w!(sta),                             // 0x99
    &[],                                     // 0x9A
    &[],                                     // 0x9B
    &[],                                     // 0x9C
    abx_w!(sta),                             // 0x9D
    &[],                                     // 0x9E
    &[],                                     // 0x9F
    &[ldy_imm, read_pc_and_set_opc],         // 0xA0
    idx!(lda),                               // 0xA1
    &[ldx_imm, read_pc_and_set_opc],         // 0xA2
    &[],                                     // 0xA3
    zpg!(ldy),                               // 0xA4
    zpg!(lda),                               // 0xA5
    zpg!(ldx),                               // 0xA6
    &[],                                     // 0xA7
    &[],                                     // 0xA8
    &[lda_imm, read_pc_and_set_opc],         // 0xA9
    &[],                                     // 0xAA
    &[],                                     // 0xAB
    abs!(ldy),                               // 0xAC
    abs!(lda),                               // 0xAD
    abs!(ldx),                               // 0xAE
    &[],                                     // 0xAF
    &[],                                     // 0xB0
    idy_r!(lda),                             // 0xB1
    &[],                                     // 0xB2
    &[],                                     // 0xB3
    zpx!(ldy),                               // 0xB4
    zpx!(lda),                               // 0xB5
    zpy!(ldx),                               // 0xB6
    &[],                                     // 0xB7
    &[],                                     // 0xB8
    aby_r!(lda),                             // 0xB9
    &[],                                     // 0xBA
    &[],                                     // 0xBB
    abx_r!(ldy),                             // 0xBC
    abx_r!(lda),                             // 0xBD
    aby_r!(ldx),                             // 0xBE
    &[],                                     // 0xBF
    &[],                                     // 0xC0
    &[],                                     // 0xC1
    &[],                                     // 0xC2
    &[],                                     // 0xC3
    &[],                                     // 0xC4
    &[],                                     // 0xC5
    &[],                                     // 0xC6
    &[],                                     // 0xC7
    &[],                                     // 0xC8
    &[],                                     // 0xC9
    &[],                                     // 0xCA
    &[],                                     // 0xCB
    &[],                                     // 0xCC
    &[],                                     // 0xCD
    &[],                                     // 0xCE
    &[],                                     // 0xCF
    &[],                                     // 0xD0
    &[],                                     // 0xD1
    &[],                                     // 0xD2
    &[],                                     // 0xD3
    &[],                                     // 0xD4
    &[],                                     // 0xD5
    &[],                                     // 0xD6
    &[],                                     // 0xD7
    &[],                                     // 0xD8
    &[],                                     // 0xD9
    &[],                                     // 0xDA
    &[],                                     // 0xDB
    &[],                                     // 0xDC
    &[],                                     // 0xDD
    &[],                                     // 0xDE
    &[],                                     // 0xDF
    &[],                                     // 0xE0
    &[],                                     // 0xE1
    &[],                                     // 0xE2
    &[],                                     // 0xE3
    &[],                                     // 0xE4
    &[],                                     // 0xE5
    zpg_rmw!(inc),                           // 0xE6
    &[],                                     // 0xE7
    &[],                                     // 0xE8
    &[],                                     // 0xE9
    &[],                                     // 0xEA
    &[],                                     // 0xEB
    &[],                                     // 0xEC
    &[],                                     // 0xED
    abs_rmw!(inc),                           // 0xEE
    &[],                                     // 0xEF
    rel!(beq),                               // 0xF0
    &[],                                     // 0xF1
    &[],                                     // 0xF2
    &[],                                     // 0xF3
    &[],                                     // 0xF4
    &[],                                     // 0xF5
    zpx_rmw!(inc),                           // 0xF6
    &[],                                     // 0xF7
    &[],                                     // 0xF8
    &[],                                     // 0xF9
    &[],                                     // 0xFA
    &[],                                     // 0xFB
    &[],                                     // 0xFC
    &[],                                     // 0xFD
    abx_rmw!(inc),                           // 0xFE
    &[],                                     // 0xFF
];

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
