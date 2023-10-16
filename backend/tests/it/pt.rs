//! Tests for Tom Harte's ProcessorTests.
//!
//! The following opcodes aren't tested (because they're unstable or
//! unimplemented): ANE, JAM, LXA.

use std::fs;

use backend::{
    cpu::{self, Status},
    Emu,
};
use rkyv::Archive;

#[derive(Archive)]
struct Test {
    initial: CpuState,
    r#final: CpuState,
    cycles: Vec<(u16, u8, CycleKind)>,
}

#[derive(Archive)]
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

#[derive(Archive)]
#[allow(dead_code)]
enum CycleKind {
    Read,
    Write,
}

fn run(opcode: u8) {
    let bytes =
        fs::read(format!("../roms/processor_tests/{:02x}.rkyv", opcode))
            .unwrap();
    let tests = unsafe { rkyv::archived_root::<Vec<Test>>(&bytes) };

    let mut emu = Emu::default();

    // Get through the reset sequence.
    for _ in 0..6 {
        cpu::step(&mut emu);
    }

    for test in tests.iter() {
        emu.cpu.a = test.initial.a;
        emu.cpu.x = test.initial.x;
        emu.cpu.y = test.initial.y;
        emu.cpu.pc = test.initial.pc;
        emu.cpu.s = test.initial.s;
        emu.cpu.p = Status(test.initial.p);

        // Use `memset` since `fill` is too slow in debug builds.
        unsafe {
            libc::memset(emu.bus.mem.as_mut_ptr() as _, 0, emu.bus.mem.len());
        };
        for &(addr, data) in test.initial.ram.iter() {
            emu.bus.mem[addr as usize] = data;
        }

        // TODO(zach): Assert read/write cycles.
        for &(addr, data, _) in test.cycles.iter() {
            cpu::step(&mut emu);
            assert_eq!(emu.bus.addr, addr);
            assert_eq!(emu.bus.data, data);
        }

        assert_eq!(emu.cpu.a, test.r#final.a);
        assert_eq!(emu.cpu.x, test.r#final.x);
        assert_eq!(emu.cpu.y, test.r#final.y);
        assert_eq!(emu.cpu.pc, test.r#final.pc);
        assert_eq!(emu.cpu.s, test.r#final.s);
        assert_eq!(emu.cpu.p.0, test.r#final.p);
        for &(addr, data) in test.r#final.ram.iter() {
            assert_eq!(emu.bus.mem[addr as usize], data);
        }
    }
}

#[test]
fn opc_00() {
    run(0x00);
}

#[test]
fn opc_01() {
    run(0x01);
}

#[test]
fn opc_03() {
    run(0x03);
}

#[test]
fn opc_04() {
    run(0x04);
}

#[test]
fn opc_05() {
    run(0x05);
}

#[test]
fn opc_06() {
    run(0x06);
}

#[test]
fn opc_07() {
    run(0x07);
}

#[test]
fn opc_08() {
    run(0x08);
}

#[test]
fn opc_09() {
    run(0x09);
}

#[test]
fn opc_0a() {
    run(0x0A);
}

#[test]
fn opc_0b() {
    run(0x0B);
}

#[test]
fn opc_0c() {
    run(0x0C);
}

#[test]
fn opc_0d() {
    run(0x0D);
}

#[test]
fn opc_0e() {
    run(0x0E);
}

#[test]
fn opc_0f() {
    run(0x0F);
}

#[test]
fn opc_10() {
    run(0x10);
}

#[test]
fn opc_11() {
    run(0x11);
}

#[test]
fn opc_13() {
    run(0x13);
}

#[test]
fn opc_14() {
    run(0x14);
}

#[test]
fn opc_15() {
    run(0x15);
}

#[test]
fn opc_16() {
    run(0x16);
}

#[test]
fn opc_17() {
    run(0x17);
}

#[test]
fn opc_18() {
    run(0x18);
}

#[test]
fn opc_19() {
    run(0x19);
}

#[test]
fn opc_1a() {
    run(0x1A);
}

#[test]
fn opc_1b() {
    run(0x1B);
}

#[test]
fn opc_1c() {
    run(0x1C);
}

#[test]
fn opc_1d() {
    run(0x1D);
}

#[test]
fn opc_1e() {
    run(0x1E);
}

#[test]
fn opc_1f() {
    run(0x1F);
}

#[test]
fn opc_20() {
    run(0x20);
}

#[test]
fn opc_21() {
    run(0x21);
}

#[test]
fn opc_23() {
    run(0x23);
}

#[test]
fn opc_24() {
    run(0x24);
}

#[test]
fn opc_25() {
    run(0x25);
}

#[test]
fn opc_26() {
    run(0x26);
}

#[test]
fn opc_27() {
    run(0x27);
}

#[test]
fn opc_28() {
    run(0x28);
}

#[test]
fn opc_29() {
    run(0x29);
}

#[test]
fn opc_2a() {
    run(0x2A);
}

#[test]
fn opc_2b() {
    run(0x2B);
}

#[test]
fn opc_2c() {
    run(0x2C);
}

#[test]
fn opc_2d() {
    run(0x2D);
}

#[test]
fn opc_2e() {
    run(0x2E);
}

#[test]
fn opc_2f() {
    run(0x2F);
}

#[test]
fn opc_30() {
    run(0x30);
}

#[test]
fn opc_31() {
    run(0x31);
}

#[test]
fn opc_33() {
    run(0x33);
}

#[test]
fn opc_34() {
    run(0x34);
}

#[test]
fn opc_35() {
    run(0x35);
}

#[test]
fn opc_36() {
    run(0x36);
}

#[test]
fn opc_37() {
    run(0x37);
}

#[test]
fn opc_38() {
    run(0x38);
}

#[test]
fn opc_39() {
    run(0x39);
}

#[test]
fn opc_3a() {
    run(0x3A);
}

#[test]
fn opc_3b() {
    run(0x3B);
}

#[test]
fn opc_3c() {
    run(0x3C);
}

#[test]
fn opc_3d() {
    run(0x3D);
}

#[test]
fn opc_3e() {
    run(0x3E);
}

#[test]
fn opc_3f() {
    run(0x3F);
}

#[test]
fn opc_40() {
    run(0x40);
}

#[test]
fn opc_41() {
    run(0x41);
}

#[test]
fn opc_43() {
    run(0x43);
}

#[test]
fn opc_44() {
    run(0x44);
}

#[test]
fn opc_45() {
    run(0x45);
}

#[test]
fn opc_46() {
    run(0x46);
}

#[test]
fn opc_47() {
    run(0x47);
}

#[test]
fn opc_48() {
    run(0x48);
}

#[test]
fn opc_49() {
    run(0x49);
}

#[test]
fn opc_4a() {
    run(0x4A);
}

#[test]
fn opc_4b() {
    run(0x4B);
}

#[test]
fn opc_4c() {
    run(0x4C);
}

#[test]
fn opc_4d() {
    run(0x4D);
}

#[test]
fn opc_4e() {
    run(0x4E);
}

#[test]
fn opc_4f() {
    run(0x4F);
}

#[test]
fn opc_50() {
    run(0x50);
}

#[test]
fn opc_51() {
    run(0x51);
}

#[test]
fn opc_53() {
    run(0x53);
}

#[test]
fn opc_54() {
    run(0x54);
}

#[test]
fn opc_55() {
    run(0x55);
}

#[test]
fn opc_56() {
    run(0x56);
}

#[test]
fn opc_57() {
    run(0x57);
}

#[test]
fn opc_58() {
    run(0x58);
}

#[test]
fn opc_59() {
    run(0x59);
}

#[test]
fn opc_5a() {
    run(0x5A);
}

#[test]
fn opc_5b() {
    run(0x5B);
}

#[test]
fn opc_5c() {
    run(0x5C);
}

#[test]
fn opc_5d() {
    run(0x5D);
}

#[test]
fn opc_5e() {
    run(0x5E);
}

#[test]
fn opc_5f() {
    run(0x5F);
}

#[test]
fn opc_60() {
    run(0x60);
}

#[test]
fn opc_61() {
    run(0x61);
}

#[test]
fn opc_63() {
    run(0x63);
}

#[test]
fn opc_64() {
    run(0x64);
}

#[test]
fn opc_65() {
    run(0x65);
}

#[test]
fn opc_66() {
    run(0x66);
}

#[test]
fn opc_67() {
    run(0x67);
}

#[test]
fn opc_68() {
    run(0x68);
}

#[test]
fn opc_69() {
    run(0x69);
}

#[test]
fn opc_6a() {
    run(0x6A);
}

#[test]
fn opc_6b() {
    run(0x6B);
}

#[test]
fn opc_6c() {
    run(0x6C);
}

#[test]
fn opc_6d() {
    run(0x6D);
}

#[test]
fn opc_6e() {
    run(0x6E);
}

#[test]
fn opc_6f() {
    run(0x6F);
}

#[test]
fn opc_70() {
    run(0x70);
}

#[test]
fn opc_71() {
    run(0x71);
}

#[test]
fn opc_73() {
    run(0x73);
}

#[test]
fn opc_74() {
    run(0x74);
}

#[test]
fn opc_75() {
    run(0x75);
}

#[test]
fn opc_76() {
    run(0x76);
}

#[test]
fn opc_77() {
    run(0x77);
}

#[test]
fn opc_78() {
    run(0x78);
}

#[test]
fn opc_79() {
    run(0x79);
}

#[test]
fn opc_7a() {
    run(0x7A);
}

#[test]
fn opc_7b() {
    run(0x7B);
}

#[test]
fn opc_7c() {
    run(0x7C);
}

#[test]
fn opc_7d() {
    run(0x7D);
}

#[test]
fn opc_7e() {
    run(0x7E);
}

#[test]
fn opc_7f() {
    run(0x7F);
}

#[test]
fn opc_80() {
    run(0x80);
}

#[test]
fn opc_81() {
    run(0x81);
}

#[test]
fn opc_82() {
    run(0x82);
}

#[test]
fn opc_83() {
    run(0x83);
}

#[test]
fn opc_84() {
    run(0x84);
}

#[test]
fn opc_85() {
    run(0x85);
}

#[test]
fn opc_86() {
    run(0x86);
}

#[test]
fn opc_87() {
    run(0x87);
}

#[test]
fn opc_88() {
    run(0x88);
}

#[test]
fn opc_89() {
    run(0x89);
}

#[test]
fn opc_8a() {
    run(0x8A);
}

#[test]
fn opc_8c() {
    run(0x8C);
}

#[test]
fn opc_8d() {
    run(0x8D);
}

#[test]
fn opc_8e() {
    run(0x8E);
}

#[test]
fn opc_8f() {
    run(0x8F);
}

#[test]
fn opc_90() {
    run(0x90);
}

#[test]
fn opc_91() {
    run(0x91);
}

#[test]
fn opc_93() {
    run(0x93);
}

#[test]
fn opc_94() {
    run(0x94);
}

#[test]
fn opc_95() {
    run(0x95);
}

#[test]
fn opc_96() {
    run(0x96);
}

#[test]
fn opc_97() {
    run(0x97);
}

#[test]
fn opc_98() {
    run(0x98);
}

#[test]
fn opc_99() {
    run(0x99);
}

#[test]
fn opc_9a() {
    run(0x9A);
}

#[test]
fn opc_9b() {
    run(0x9B);
}

#[test]
fn opc_9c() {
    run(0x9C);
}

#[test]
fn opc_9d() {
    run(0x9D);
}

#[test]
fn opc_9e() {
    run(0x9E);
}

#[test]
fn opc_9f() {
    run(0x9F);
}

#[test]
fn opc_a0() {
    run(0xA0);
}

#[test]
fn opc_a1() {
    run(0xA1);
}

#[test]
fn opc_a2() {
    run(0xA2);
}

#[test]
fn opc_a3() {
    run(0xA3);
}

#[test]
fn opc_a4() {
    run(0xA4);
}

#[test]
fn opc_a5() {
    run(0xA5);
}

#[test]
fn opc_a6() {
    run(0xA6);
}

#[test]
fn opc_a7() {
    run(0xA7);
}

#[test]
fn opc_a8() {
    run(0xA8);
}

#[test]
fn opc_a9() {
    run(0xA9);
}

#[test]
fn opc_aa() {
    run(0xAA);
}

#[test]
fn opc_ac() {
    run(0xAC);
}

#[test]
fn opc_ad() {
    run(0xAD);
}

#[test]
fn opc_ae() {
    run(0xAE);
}

#[test]
fn opc_af() {
    run(0xAF);
}

#[test]
fn opc_b0() {
    run(0xB0);
}

#[test]
fn opc_b1() {
    run(0xB1);
}

#[test]
fn opc_b3() {
    run(0xB3);
}

#[test]
fn opc_b4() {
    run(0xB4);
}

#[test]
fn opc_b5() {
    run(0xB5);
}

#[test]
fn opc_b6() {
    run(0xB6);
}

#[test]
fn opc_b7() {
    run(0xB7);
}

#[test]
fn opc_b8() {
    run(0xB8);
}

#[test]
fn opc_b9() {
    run(0xB9);
}

#[test]
fn opc_ba() {
    run(0xBA);
}

#[test]
fn opc_bb() {
    run(0xBB);
}

#[test]
fn opc_bc() {
    run(0xBC);
}

#[test]
fn opc_bd() {
    run(0xBD);
}

#[test]
fn opc_be() {
    run(0xBE);
}

#[test]
fn opc_bf() {
    run(0xBF);
}

#[test]
fn opc_c0() {
    run(0xC0);
}

#[test]
fn opc_c1() {
    run(0xC1);
}

#[test]
fn opc_c2() {
    run(0xC2);
}

#[test]
fn opc_c3() {
    run(0xC3);
}

#[test]
fn opc_c4() {
    run(0xC4);
}

#[test]
fn opc_c5() {
    run(0xC5);
}

#[test]
fn opc_c6() {
    run(0xC6);
}

#[test]
fn opc_c7() {
    run(0xC7);
}

#[test]
fn opc_c8() {
    run(0xC8);
}

#[test]
fn opc_c9() {
    run(0xC9);
}

#[test]
fn opc_ca() {
    run(0xCA);
}

#[test]
fn opc_cb() {
    run(0xCB);
}

#[test]
fn opc_cc() {
    run(0xCC);
}

#[test]
fn opc_cd() {
    run(0xCD);
}

#[test]
fn opc_ce() {
    run(0xCE);
}

#[test]
fn opc_cf() {
    run(0xCF);
}

#[test]
fn opc_d0() {
    run(0xD0);
}

#[test]
fn opc_d1() {
    run(0xD1);
}

#[test]
fn opc_d3() {
    run(0xD3);
}

#[test]
fn opc_d4() {
    run(0xD4);
}

#[test]
fn opc_d5() {
    run(0xD5);
}

#[test]
fn opc_d6() {
    run(0xD6);
}

#[test]
fn opc_d7() {
    run(0xD7);
}

#[test]
fn opc_d8() {
    run(0xD8);
}

#[test]
fn opc_d9() {
    run(0xD9);
}

#[test]
fn opc_da() {
    run(0xDA);
}

#[test]
fn opc_db() {
    run(0xDB);
}

#[test]
fn opc_dc() {
    run(0xDC);
}

#[test]
fn opc_dd() {
    run(0xDD);
}

#[test]
fn opc_de() {
    run(0xDE);
}

#[test]
fn opc_df() {
    run(0xDF);
}

#[test]
fn opc_e0() {
    run(0xE0);
}

#[test]
fn opc_e1() {
    run(0xE1);
}

#[test]
fn opc_e2() {
    run(0xE2);
}

#[test]
fn opc_e3() {
    run(0xE3);
}

#[test]
fn opc_e4() {
    run(0xE4);
}

#[test]
fn opc_e5() {
    run(0xE5);
}

#[test]
fn opc_e6() {
    run(0xE6);
}

#[test]
fn opc_e7() {
    run(0xE7);
}

#[test]
fn opc_e8() {
    run(0xE8);
}

#[test]
fn opc_e9() {
    run(0xE9);
}

#[test]
fn opc_ea() {
    run(0xEA);
}

#[test]
fn opc_eb() {
    run(0xEB);
}

#[test]
fn opc_ec() {
    run(0xEC);
}

#[test]
fn opc_ed() {
    run(0xED);
}

#[test]
fn opc_ee() {
    run(0xEE);
}

#[test]
fn opc_ef() {
    run(0xEF);
}

#[test]
fn opc_f0() {
    run(0xF0);
}

#[test]
fn opc_f1() {
    run(0xF1);
}

#[test]
fn opc_f3() {
    run(0xF3);
}

#[test]
fn opc_f4() {
    run(0xF4);
}

#[test]
fn opc_f5() {
    run(0xF5);
}

#[test]
fn opc_f6() {
    run(0xF6);
}

#[test]
fn opc_f7() {
    run(0xF7);
}

#[test]
fn opc_f8() {
    run(0xF8);
}

#[test]
fn opc_f9() {
    run(0xF9);
}

#[test]
fn opc_fa() {
    run(0xFA);
}

#[test]
fn opc_fb() {
    run(0xFB);
}

#[test]
fn opc_fc() {
    run(0xFC);
}

#[test]
fn opc_fd() {
    run(0xFD);
}

#[test]
fn opc_fe() {
    run(0xFE);
}

#[test]
fn opc_ff() {
    run(0xFF);
}
