use std::{fs, ptr};

use serde::Deserialize;

use crate::{
    cpu::{self, Cpu, Status},
    emu::Emu,
    nrom::Nrom,
};

macro_rules! processor_test {
    ($name:ident, $opc:expr) => {
        #[test]
        fn $name() {
            run($opc);
        }
    };
}

#[derive(Deserialize)]
struct Test<'a> {
    initial: CpuState,
    #[serde(rename = "final")]
    end: CpuState,
    #[serde(borrow)]
    cycles: Vec<(u16, u8, &'a str)>,
}

#[derive(Deserialize)]
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

fn run(opc: u8) {
    let contents =
        fs::read(format!("../roms/processor_tests/{:02x}.json", opc)).unwrap();
    let tests: Vec<Test> = serde_json::from_slice(&contents).unwrap();

    let cpu = Cpu::new();
    let mut emu = Emu {
        cpu,
        nrom: Nrom { prg_ram: Box::new([]), prg_rom: Box::new([]) },
    };

    for test in tests {
        emu.cpu.a = test.initial.a;
        emu.cpu.x = test.initial.x;
        emu.cpu.y = test.initial.y;
        emu.cpu.pc = test.initial.pc;
        emu.cpu.s = test.initial.s;
        emu.cpu.p = Status(test.initial.p);

        // Use `write_bytes` since `fill` is too slow in debug builds.
        unsafe {
            ptr::write_bytes(
                emu.cpu.bus.ram.as_mut_ptr(),
                0,
                emu.cpu.bus.ram.len(),
            );
        }
        for (addr, data) in test.initial.ram {
            emu.cpu.bus.ram[addr as usize] = data;
        }
        emu.cpu.bus.cycles.clear();

        cpu::step(&mut emu);

        assert_eq!(emu.cpu.a, test.end.a);
        assert_eq!(emu.cpu.x, test.end.x);
        assert_eq!(emu.cpu.y, test.end.y);
        assert_eq!(emu.cpu.pc, test.end.pc);
        assert_eq!(emu.cpu.s, test.end.s);
        assert_eq!(emu.cpu.p.0, test.end.p);
        for (addr, data) in test.end.ram {
            assert_eq!(emu.cpu.bus.ram[addr as usize], data);
        }
        assert_eq!(emu.cpu.bus.cycles, test.cycles);
    }
}

// JAM, ANE (8B), and LXA (AB) are excluded.
processor_test!(opc_00, 0x00);
processor_test!(opc_01, 0x01);
processor_test!(opc_03, 0x03);
processor_test!(opc_04, 0x04);
processor_test!(opc_05, 0x05);
processor_test!(opc_06, 0x06);
processor_test!(opc_07, 0x07);
processor_test!(opc_08, 0x08);
processor_test!(opc_09, 0x09);
processor_test!(opc_0a, 0x0A);
processor_test!(opc_0b, 0x0B);
processor_test!(opc_0c, 0x0C);
processor_test!(opc_0d, 0x0D);
processor_test!(opc_0e, 0x0E);
processor_test!(opc_0f, 0x0F);
processor_test!(opc_10, 0x10);
processor_test!(opc_11, 0x11);
processor_test!(opc_13, 0x13);
processor_test!(opc_14, 0x14);
processor_test!(opc_15, 0x15);
processor_test!(opc_16, 0x16);
processor_test!(opc_17, 0x17);
processor_test!(opc_18, 0x18);
processor_test!(opc_19, 0x19);
processor_test!(opc_1a, 0x1A);
processor_test!(opc_1b, 0x1B);
processor_test!(opc_1c, 0x1C);
processor_test!(opc_1d, 0x1D);
processor_test!(opc_1e, 0x1E);
processor_test!(opc_1f, 0x1F);
processor_test!(opc_20, 0x20);
processor_test!(opc_21, 0x21);
processor_test!(opc_23, 0x23);
processor_test!(opc_24, 0x24);
processor_test!(opc_25, 0x25);
processor_test!(opc_26, 0x26);
processor_test!(opc_27, 0x27);
processor_test!(opc_28, 0x28);
processor_test!(opc_29, 0x29);
processor_test!(opc_2a, 0x2A);
processor_test!(opc_2b, 0x2B);
processor_test!(opc_2c, 0x2C);
processor_test!(opc_2d, 0x2D);
processor_test!(opc_2e, 0x2E);
processor_test!(opc_2f, 0x2F);
processor_test!(opc_30, 0x30);
processor_test!(opc_31, 0x31);
processor_test!(opc_33, 0x33);
processor_test!(opc_34, 0x34);
processor_test!(opc_35, 0x35);
processor_test!(opc_36, 0x36);
processor_test!(opc_37, 0x37);
processor_test!(opc_38, 0x38);
processor_test!(opc_39, 0x39);
processor_test!(opc_3a, 0x3A);
processor_test!(opc_3b, 0x3B);
processor_test!(opc_3c, 0x3C);
processor_test!(opc_3d, 0x3D);
processor_test!(opc_3e, 0x3E);
processor_test!(opc_3f, 0x3F);
processor_test!(opc_40, 0x40);
processor_test!(opc_41, 0x41);
processor_test!(opc_43, 0x43);
processor_test!(opc_44, 0x44);
processor_test!(opc_45, 0x45);
processor_test!(opc_46, 0x46);
processor_test!(opc_47, 0x47);
processor_test!(opc_48, 0x48);
processor_test!(opc_49, 0x49);
processor_test!(opc_4a, 0x4A);
processor_test!(opc_4b, 0x4B);
processor_test!(opc_4c, 0x4C);
processor_test!(opc_4d, 0x4D);
processor_test!(opc_4e, 0x4E);
processor_test!(opc_4f, 0x4F);
processor_test!(opc_50, 0x50);
processor_test!(opc_51, 0x51);
processor_test!(opc_53, 0x53);
processor_test!(opc_54, 0x54);
processor_test!(opc_55, 0x55);
processor_test!(opc_56, 0x56);
processor_test!(opc_57, 0x57);
processor_test!(opc_58, 0x58);
processor_test!(opc_59, 0x59);
processor_test!(opc_5a, 0x5A);
processor_test!(opc_5b, 0x5B);
processor_test!(opc_5c, 0x5C);
processor_test!(opc_5d, 0x5D);
processor_test!(opc_5e, 0x5E);
processor_test!(opc_5f, 0x5F);
processor_test!(opc_60, 0x60);
processor_test!(opc_61, 0x61);
processor_test!(opc_63, 0x63);
processor_test!(opc_64, 0x64);
processor_test!(opc_65, 0x65);
processor_test!(opc_66, 0x66);
processor_test!(opc_67, 0x67);
processor_test!(opc_68, 0x68);
processor_test!(opc_69, 0x69);
processor_test!(opc_6a, 0x6A);
processor_test!(opc_6b, 0x6B);
processor_test!(opc_6c, 0x6C);
processor_test!(opc_6d, 0x6D);
processor_test!(opc_6e, 0x6E);
processor_test!(opc_6f, 0x6F);
processor_test!(opc_70, 0x70);
processor_test!(opc_71, 0x71);
processor_test!(opc_73, 0x73);
processor_test!(opc_74, 0x74);
processor_test!(opc_75, 0x75);
processor_test!(opc_76, 0x76);
processor_test!(opc_77, 0x77);
processor_test!(opc_78, 0x78);
processor_test!(opc_79, 0x79);
processor_test!(opc_7a, 0x7A);
processor_test!(opc_7b, 0x7B);
processor_test!(opc_7c, 0x7C);
processor_test!(opc_7d, 0x7D);
processor_test!(opc_7e, 0x7E);
processor_test!(opc_7f, 0x7F);
processor_test!(opc_80, 0x80);
processor_test!(opc_81, 0x81);
processor_test!(opc_82, 0x82);
processor_test!(opc_83, 0x83);
processor_test!(opc_84, 0x84);
processor_test!(opc_85, 0x85);
processor_test!(opc_86, 0x86);
processor_test!(opc_87, 0x87);
processor_test!(opc_88, 0x88);
processor_test!(opc_89, 0x89);
processor_test!(opc_8a, 0x8A);
processor_test!(opc_8c, 0x8C);
processor_test!(opc_8d, 0x8D);
processor_test!(opc_8e, 0x8E);
processor_test!(opc_8f, 0x8F);
processor_test!(opc_90, 0x90);
processor_test!(opc_91, 0x91);
processor_test!(opc_93, 0x93);
processor_test!(opc_94, 0x94);
processor_test!(opc_95, 0x95);
processor_test!(opc_96, 0x96);
processor_test!(opc_97, 0x97);
processor_test!(opc_98, 0x98);
processor_test!(opc_99, 0x99);
processor_test!(opc_9a, 0x9A);
processor_test!(opc_9b, 0x9B);
processor_test!(opc_9c, 0x9C);
processor_test!(opc_9d, 0x9D);
processor_test!(opc_9e, 0x9E);
processor_test!(opc_9f, 0x9F);
processor_test!(opc_a0, 0xA0);
processor_test!(opc_a1, 0xA1);
processor_test!(opc_a2, 0xA2);
processor_test!(opc_a3, 0xA3);
processor_test!(opc_a4, 0xA4);
processor_test!(opc_a5, 0xA5);
processor_test!(opc_a6, 0xA6);
processor_test!(opc_a7, 0xA7);
processor_test!(opc_a8, 0xA8);
processor_test!(opc_a9, 0xA9);
processor_test!(opc_aa, 0xAA);
processor_test!(opc_ac, 0xAC);
processor_test!(opc_ad, 0xAD);
processor_test!(opc_ae, 0xAE);
processor_test!(opc_af, 0xAF);
processor_test!(opc_b0, 0xB0);
processor_test!(opc_b1, 0xB1);
processor_test!(opc_b3, 0xB3);
processor_test!(opc_b4, 0xB4);
processor_test!(opc_b5, 0xB5);
processor_test!(opc_b6, 0xB6);
processor_test!(opc_b7, 0xB7);
processor_test!(opc_b8, 0xB8);
processor_test!(opc_b9, 0xB9);
processor_test!(opc_ba, 0xBA);
processor_test!(opc_bb, 0xBB);
processor_test!(opc_bc, 0xBC);
processor_test!(opc_bd, 0xBD);
processor_test!(opc_be, 0xBE);
processor_test!(opc_bf, 0xBF);
processor_test!(opc_c0, 0xC0);
processor_test!(opc_c1, 0xC1);
processor_test!(opc_c2, 0xC2);
processor_test!(opc_c3, 0xC3);
processor_test!(opc_c4, 0xC4);
processor_test!(opc_c5, 0xC5);
processor_test!(opc_c6, 0xC6);
processor_test!(opc_c7, 0xC7);
processor_test!(opc_c8, 0xC8);
processor_test!(opc_c9, 0xC9);
processor_test!(opc_ca, 0xCA);
processor_test!(opc_cb, 0xCB);
processor_test!(opc_cc, 0xCC);
processor_test!(opc_cd, 0xCD);
processor_test!(opc_ce, 0xCE);
processor_test!(opc_cf, 0xCF);
processor_test!(opc_d0, 0xD0);
processor_test!(opc_d1, 0xD1);
processor_test!(opc_d3, 0xD3);
processor_test!(opc_d4, 0xD4);
processor_test!(opc_d5, 0xD5);
processor_test!(opc_d6, 0xD6);
processor_test!(opc_d7, 0xD7);
processor_test!(opc_d8, 0xD8);
processor_test!(opc_d9, 0xD9);
processor_test!(opc_da, 0xDA);
processor_test!(opc_db, 0xDB);
processor_test!(opc_dc, 0xDC);
processor_test!(opc_dd, 0xDD);
processor_test!(opc_de, 0xDE);
processor_test!(opc_df, 0xDF);
processor_test!(opc_e0, 0xE0);
processor_test!(opc_e1, 0xE1);
processor_test!(opc_e2, 0xE2);
processor_test!(opc_e3, 0xE3);
processor_test!(opc_e4, 0xE4);
processor_test!(opc_e5, 0xE5);
processor_test!(opc_e6, 0xE6);
processor_test!(opc_e7, 0xE7);
processor_test!(opc_e8, 0xE8);
processor_test!(opc_e9, 0xE9);
processor_test!(opc_ea, 0xEA);
processor_test!(opc_eb, 0xEB);
processor_test!(opc_ec, 0xEC);
processor_test!(opc_ed, 0xED);
processor_test!(opc_ee, 0xEE);
processor_test!(opc_ef, 0xEF);
processor_test!(opc_f0, 0xF0);
processor_test!(opc_f1, 0xF1);
processor_test!(opc_f3, 0xF3);
processor_test!(opc_f4, 0xF4);
processor_test!(opc_f5, 0xF5);
processor_test!(opc_f6, 0xF6);
processor_test!(opc_f7, 0xF7);
processor_test!(opc_f8, 0xF8);
processor_test!(opc_f9, 0xF9);
processor_test!(opc_fa, 0xFA);
processor_test!(opc_fb, 0xFB);
processor_test!(opc_fc, 0xFC);
processor_test!(opc_fd, 0xFD);
processor_test!(opc_fe, 0xFE);
processor_test!(opc_ff, 0xFF);
