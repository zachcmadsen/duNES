use crate::{
    cpu::{
        instr::{
            adc, adc_imm, and, and_imm, asl, asl_accumulator, bcc, bcs, beq,
            bit, bmi, bne, bpl, inc, lda, lda_imm, ldx, ldx_imm, ldy, ldy_imm,
            lsr, lsr_accumulator, sta, stx, sty,
        },
        mode::read_pc_and_set_opc,
    },
    Emu,
};

macro_rules! abs {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_set_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! abs_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_set_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                true,
                true,
            >,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                true,
                false,
            >,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! abx_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                true,
                false,
            >,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! aby_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                false,
                true,
            >,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! aby_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                false,
                false,
            >,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! idx {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::read_addr_and_set_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! idy_r {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::read_addr_and_add_y_to_low_and_set_high::<true>,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! idy_w {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::read_addr_and_add_y_to_low_and_set_high::<false>,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! rel {
    ($f:ident) => {
        &[
            $f,
            $crate::cpu::mode::read_pc_and_add_data_to_pc,
            $crate::cpu::mode::read_pc_and_fix_pch,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpg {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpg_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpx {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpx_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! zpy {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_add_index::<false>,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

pub static OPC_LUT: [&[fn(&mut Emu)]; 0x100] = [
    &[],                                     // 0x00
    &[],                                     // 0x01
    &[],                                     // 0x02
    &[],                                     // 0x03
    &[],                                     // 0x04
    &[],                                     // 0x05
    zpg_rmw!(asl),                           // 0x06
    &[],                                     // 0x07
    &[],                                     // 0x08
    &[],                                     // 0x09
    &[asl_accumulator, read_pc_and_set_opc], // 0x0A
    &[],                                     // 0x0B
    &[],                                     // 0x0C
    &[],                                     // 0x0D
    abs_rmw!(asl),                           // 0x0E
    &[],                                     // 0x0F
    rel!(bpl),                               // 0x10
    &[],                                     // 0x11
    &[],                                     // 0x12
    &[],                                     // 0x13
    &[],                                     // 0x14
    &[],                                     // 0x15
    zpx_rmw!(asl),                           // 0x16
    &[],                                     // 0x17
    &[],                                     // 0x18
    &[],                                     // 0x19
    &[],                                     // 0x1A
    &[],                                     // 0x1B
    &[],                                     // 0x1C
    &[],                                     // 0x1D
    abx_rmw!(asl),                           // 0x1E
    &[],                                     // 0x1F
    &[],                                     // 0x20
    idx!(and),                               // 0x21
    &[],                                     // 0x22
    &[],                                     // 0x23
    zpg!(bit),                               // 0x24
    zpg!(and),                               // 0x25
    &[],                                     // 0x26
    &[],                                     // 0x27
    &[],                                     // 0x28
    &[and_imm, read_pc_and_set_opc],         // 0x29
    &[],                                     // 0x2A
    &[],                                     // 0x2B
    abs!(bit),                               // 0x2C
    abs!(and),                               // 0x2D
    &[],                                     // 0x2E
    &[],                                     // 0x2F
    rel!(bmi),                               // 0x30
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
    rel!(bcc),                               // 0x90
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
    rel!(bcs),                               // 0xB0
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
    rel!(bne),                               // 0xD0
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
