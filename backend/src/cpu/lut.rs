use crate::{
    cpu::{
        instr::{
            adc, adc_imm, and, and_imm, asl, asl_a, bcc, bcs, beq, bit, bmi,
            bne, bpl, bvc, bvs, clc, cld, cli, clv, cmp, cmp_imm, cpx,
            cpx_imm, cpy, cpy_imm, dec, dex, dey, eor, eor_imm, inc, inx, iny,
            lda, lda_imm, ldx, ldx_imm, ldy, ldy_imm, lsr, lsr_a, nop,
            nop_imm, nop_imp, ora, ora_imm, peek, pha, php, pla, plp, pull_p,
            pull_pch, pull_pcl, push_p, push_pch, push_pcl, read_pc,
            read_pc_and_inc_pc, rol, rol_a, ror, ror_a, sbc, sbc_imm, sec,
            sed, sei, set_pch, set_pcl, slo, sta, stx, sty, tax, tay, tsx,
            txa, txs, tya,
        },
        mode::read_pc_and_set_opc,
        IRQ_VECTOR,
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

macro_rules! aby_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_pc_and_add_index_to_low_and_set_high::<
                false,
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

macro_rules! idx_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_add_index::<true>,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::read_addr_and_set_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
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

macro_rules! idy_rmw {
    ($f:ident) => {
        &[
            $crate::cpu::mode::read_pc_and_set_low,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::read_addr_and_add_y_to_low_and_set_high::<false>,
            $crate::cpu::mode::read_addr_and_opt_fix_high,
            $crate::cpu::mode::read_addr_and_set_data,
            $crate::cpu::mode::write_data_to_addr,
            $f,
            $crate::cpu::mode::read_pc_and_set_opc,
        ]
    };
}

macro_rules! imp {
    ($f:ident) => {
        &[$f, $crate::cpu::mode::read_pc_and_set_opc]
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
    &[
        read_pc_and_inc_pc,
        push_pch,
        push_pcl,
        push_p,
        set_pcl::<IRQ_VECTOR>,
        set_pch::<IRQ_VECTOR>,
        read_pc_and_set_opc,
    ], // 0x00
    idx!(ora),                                  // 0x01
    &[],                                        // 0x02
    idx_rmw!(slo),                              // 0x03
    zpg!(nop),                                  // 0x04
    zpg!(ora),                                  // 0x05
    zpg_rmw!(asl),                              // 0x06
    zpg_rmw!(slo),                              // 0x07
    &[read_pc, php, read_pc_and_set_opc],       // 0x08
    imp!(ora_imm),                              // 0x09
    imp!(asl_a),                                // 0x0A
    &[],                                        // 0x0B
    abs!(nop),                                  // 0x0C
    abs!(ora),                                  // 0x0D
    abs_rmw!(asl),                              // 0x0E
    abs_rmw!(slo),                              // 0x0F
    rel!(bpl),                                  // 0x10
    idy_r!(ora),                                // 0x11
    &[],                                        // 0x12
    idy_rmw!(slo),                              // 0x13
    zpx!(nop),                                  // 0x14
    zpx!(ora),                                  // 0x15
    zpx_rmw!(asl),                              // 0x16
    zpx_rmw!(slo),                              // 0x17
    imp!(clc),                                  // 0x18
    aby_r!(ora),                                // 0x19
    imp!(nop_imp),                              // 0x1A
    aby_rmw!(slo),                              // 0x1B
    abx_r!(nop),                                // 0x1C
    abx_r!(ora),                                // 0x1D
    abx_rmw!(asl),                              // 0x1E
    abx_rmw!(slo),                              // 0x1F
    &[],                                        // 0x20
    idx!(and),                                  // 0x21
    &[],                                        // 0x22
    &[],                                        // 0x23
    zpg!(bit),                                  // 0x24
    zpg!(and),                                  // 0x25
    zpg_rmw!(rol),                              // 0x26
    &[],                                        // 0x27
    &[read_pc, peek, plp, read_pc_and_set_opc], // 0x28
    imp!(and_imm),                              // 0x29
    imp!(rol_a),                                // 0x2A
    &[],                                        // 0x2B
    abs!(bit),                                  // 0x2C
    abs!(and),                                  // 0x2D
    abs_rmw!(rol),                              // 0x2E
    &[],                                        // 0x2F
    rel!(bmi),                                  // 0x30
    idy_r!(and),                                // 0x31
    &[],                                        // 0x32
    &[],                                        // 0x33
    zpx!(nop),                                  // 0x34
    zpx!(and),                                  // 0x35
    zpx_rmw!(rol),                              // 0x36
    &[],                                        // 0x37
    imp!(sec),                                  // 0x38
    aby_r!(and),                                // 0x39
    imp!(nop_imp),                              // 0x3A
    &[],                                        // 0x3B
    abx_r!(nop),                                // 0x3C
    abx_r!(and),                                // 0x3D
    abx_rmw!(rol),                              // 0x3E
    &[],                                        // 0x3F
    &[read_pc, peek, pull_p, pull_pcl, pull_pch, read_pc_and_set_opc], // 0x40
    idx!(eor),                                  // 0x41
    &[],                                        // 0x42
    &[],                                        // 0x43
    zpg!(nop),                                  // 0x44
    zpg!(eor),                                  // 0x45
    zpg_rmw!(lsr),                              // 0x46
    &[],                                        // 0x47
    &[read_pc, pha, read_pc_and_set_opc],       // 0x48
    imp!(eor_imm),                              // 0x49
    imp!(lsr_a),                                // 0x4A
    &[],                                        // 0x4B
    &[],                                        // 0x4C
    abs!(eor),                                  // 0x4D
    abs_rmw!(lsr),                              // 0x4E
    &[],                                        // 0x4F
    rel!(bvc),                                  // 0x50
    idy_r!(eor),                                // 0x51
    &[],                                        // 0x52
    &[],                                        // 0x53
    zpx!(nop),                                  // 0x54
    zpx!(eor),                                  // 0x55
    zpx_rmw!(lsr),                              // 0x56
    &[],                                        // 0x57
    imp!(cli),                                  // 0x58
    aby_r!(eor),                                // 0x59
    imp!(nop_imp),                              // 0x5A
    &[],                                        // 0x5B
    abx_r!(nop),                                // 0x5C
    abx_r!(eor),                                // 0x5D
    abx_rmw!(lsr),                              // 0x5E
    &[],                                        // 0x5F
    &[
        read_pc,
        peek,
        pull_pcl,
        pull_pch,
        read_pc_and_inc_pc,
        read_pc_and_set_opc,
    ], // 0x60
    idx!(adc),                                  // 0x61
    &[],                                        // 0x62
    &[],                                        // 0x63
    zpg!(nop),                                  // 0x64
    zpg!(adc),                                  // 0x65
    zpg_rmw!(ror),                              // 0x66
    &[],                                        // 0x67
    &[read_pc, peek, pla, read_pc_and_set_opc], // 0x68
    imp!(adc_imm),                              // 0x69
    imp!(ror_a),                                // 0x6A
    &[],                                        // 0x6B
    &[],                                        // 0x6C
    abs!(adc),                                  // 0x6D
    abs_rmw!(ror),                              // 0x6E
    &[],                                        // 0x6F
    rel!(bvs),                                  // 0x70
    idy_r!(adc),                                // 0x71
    &[],                                        // 0x72
    &[],                                        // 0x73
    zpx!(nop),                                  // 0x74
    zpx!(adc),                                  // 0x75
    zpx_rmw!(ror),                              // 0x76
    &[],                                        // 0x77
    imp!(sei),                                  // 0x78
    aby_r!(adc),                                // 0x79
    imp!(nop_imp),                              // 0x7A
    &[],                                        // 0x7B
    abx_r!(nop),                                // 0x7C
    abx_r!(adc),                                // 0x7D
    abx_rmw!(ror),                              // 0x7E
    &[],                                        // 0x7F
    imp!(nop_imm),                              // 0x80
    idx!(sta),                                  // 0x81
    imp!(nop_imm),                              // 0x82
    &[],                                        // 0x83
    zpg!(sty),                                  // 0x84
    zpg!(sta),                                  // 0x85
    zpg!(stx),                                  // 0x86
    &[],                                        // 0x87
    imp!(dey),                                  // 0x88
    imp!(nop_imm),                              // 0x89
    imp!(txa),                                  // 0x8A
    &[],                                        // 0x8B
    abs!(sty),                                  // 0x8C
    abs!(sta),                                  // 0x8D
    abs!(stx),                                  // 0x8E
    &[],                                        // 0x8F
    rel!(bcc),                                  // 0x90
    idy_w!(sta),                                // 0x91
    &[],                                        // 0x92
    &[],                                        // 0x93
    zpx!(sty),                                  // 0x94
    zpx!(sta),                                  // 0x95
    zpy!(stx),                                  // 0x96
    &[],                                        // 0x97
    imp!(tya),                                  // 0x98
    aby_w!(sta),                                // 0x99
    imp!(txs),                                  // 0x9A
    &[],                                        // 0x9B
    &[],                                        // 0x9C
    abx_w!(sta),                                // 0x9D
    &[],                                        // 0x9E
    &[],                                        // 0x9F
    imp!(ldy_imm),                              // 0xA0
    idx!(lda),                                  // 0xA1
    imp!(ldx_imm),                              // 0xA2
    &[],                                        // 0xA3
    zpg!(ldy),                                  // 0xA4
    zpg!(lda),                                  // 0xA5
    zpg!(ldx),                                  // 0xA6
    &[],                                        // 0xA7
    imp!(tay),                                  // 0xA8
    imp!(lda_imm),                              // 0xA9
    imp!(tax),                                  // 0xAA
    &[],                                        // 0xAB
    abs!(ldy),                                  // 0xAC
    abs!(lda),                                  // 0xAD
    abs!(ldx),                                  // 0xAE
    &[],                                        // 0xAF
    rel!(bcs),                                  // 0xB0
    idy_r!(lda),                                // 0xB1
    &[],                                        // 0xB2
    &[],                                        // 0xB3
    zpx!(ldy),                                  // 0xB4
    zpx!(lda),                                  // 0xB5
    zpy!(ldx),                                  // 0xB6
    &[],                                        // 0xB7
    imp!(clv),                                  // 0xB8
    aby_r!(lda),                                // 0xB9
    imp!(tsx),                                  // 0xBA
    &[],                                        // 0xBB
    abx_r!(ldy),                                // 0xBC
    abx_r!(lda),                                // 0xBD
    aby_r!(ldx),                                // 0xBE
    &[],                                        // 0xBF
    imp!(cpy_imm),                              // 0xC0
    idx!(cmp),                                  // 0xC1
    imp!(nop_imm),                              // 0xC2
    &[],                                        // 0xC3
    zpg!(cpy),                                  // 0xC4
    zpg!(cmp),                                  // 0xC5
    zpg_rmw!(dec),                              // 0xC6
    &[],                                        // 0xC7
    imp!(iny),                                  // 0xC8
    imp!(cmp_imm),                              // 0xC9
    imp!(dex),                                  // 0xCA
    &[],                                        // 0xCB
    abs!(cpy),                                  // 0xCC
    abs!(cmp),                                  // 0xCD
    abs_rmw!(dec),                              // 0xCE
    &[],                                        // 0xCF
    rel!(bne),                                  // 0xD0
    idy_r!(cmp),                                // 0xD1
    &[],                                        // 0xD2
    &[],                                        // 0xD3
    zpx!(nop),                                  // 0xD4
    zpx!(cmp),                                  // 0xD5
    zpx_rmw!(dec),                              // 0xD6
    &[],                                        // 0xD7
    imp!(cld),                                  // 0xD8
    aby_r!(cmp),                                // 0xD9
    imp!(nop_imp),                              // 0xDA
    &[],                                        // 0xDB
    abx_r!(nop),                                // 0xDC
    abx_r!(cmp),                                // 0xDD
    abx_rmw!(dec),                              // 0xDE
    &[],                                        // 0xDF
    imp!(cpx_imm),                              // 0xE0
    idx!(sbc),                                  // 0xE1
    imp!(nop_imm),                              // 0xE2
    &[],                                        // 0xE3
    zpg!(cpx),                                  // 0xE4
    zpg!(sbc),                                  // 0xE5
    zpg_rmw!(inc),                              // 0xE6
    &[],                                        // 0xE7
    imp!(inx),                                  // 0xE8
    imp!(sbc_imm),                              // 0xE9
    imp!(nop_imp),                              // 0xEA
    &[],                                        // 0xEB
    abs!(cpx),                                  // 0xEC
    abs!(sbc),                                  // 0xED
    abs_rmw!(inc),                              // 0xEE
    &[],                                        // 0xEF
    rel!(beq),                                  // 0xF0
    idy_r!(sbc),                                // 0xF1
    &[],                                        // 0xF2
    &[],                                        // 0xF3
    zpx!(nop),                                  // 0xF4
    zpx!(sbc),                                  // 0xF5
    zpx_rmw!(inc),                              // 0xF6
    &[],                                        // 0xF7
    imp!(sed),                                  // 0xF8
    aby_r!(sbc),                                // 0xF9
    imp!(nop_imp),                              // 0xFA
    &[],                                        // 0xFB
    abx_r!(nop),                                // 0xFC
    abx_r!(sbc),                                // 0xFD
    abx_rmw!(inc),                              // 0xFE
    &[],                                        // 0xFF
];
