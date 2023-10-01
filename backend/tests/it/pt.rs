use std::fs;

use backend::{Emu, Status};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
struct TestCase<'a> {
    initial: CpuState,
    r#final: CpuState,
    cycles: Vec<(u16, u8, &'a str)>,
}

#[derive(Deserialize)]
struct CpuState {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

fn run(opcode: u8) {
    let bytes =
        fs::read(format!("../roms/processor_tests/{:02x}.json", opcode))
            .unwrap();
    let tests: Vec<TestCase> = serde_json::from_slice(&bytes).unwrap();

    let mut emu = Emu::new();

    for test in tests {
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
        for (addr, data) in test.initial.ram {
            emu.bus.mem[addr as usize] = data;
        }

        for (addr, data, _) in test.cycles {
            emu.step();
            assert_eq!(emu.bus.addr, addr);
            assert_eq!(emu.bus.data, data);
        }

        assert_eq!(emu.cpu.a, test.r#final.a);
        assert_eq!(emu.cpu.x, test.r#final.x);
        assert_eq!(emu.cpu.y, test.r#final.y);
        assert_eq!(emu.cpu.pc, test.r#final.pc);
        assert_eq!(emu.cpu.s, test.r#final.s);
        assert_eq!(emu.cpu.p.0, test.r#final.p);
        for (addr, data) in test.r#final.ram {
            assert_eq!(emu.bus.mem[addr as usize], data);
        }
    }
}

#[test]
fn opc_00() {
    run(0x00);
}

#[test]
fn opc_06() {
    run(0x06);
}

#[test]
fn opc_0a() {
    run(0x0A);
}

#[test]
fn opc_0e() {
    run(0x0E);
}

#[test]
fn opc_10() {
    run(0x10);
}

#[test]
fn opc_16() {
    run(0x16);
}

#[test]
fn opc_18() {
    run(0x18);
}

#[test]
fn opc_1e() {
    run(0x1E);
}

#[test]
fn opc_21() {
    run(0x21);
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
fn opc_29() {
    run(0x29);
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
fn opc_30() {
    run(0x30);
}

#[test]
fn opc_31() {
    run(0x31);
}

#[test]
fn opc_35() {
    run(0x35);
}

#[test]
fn opc_39() {
    run(0x39);
}

#[test]
fn opc_3d() {
    run(0x3D);
}

#[test]
fn opc_41() {
    run(0x41);
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
fn opc_49() {
    run(0x49);
}

#[test]
fn opc_4a() {
    run(0x4A);
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
fn opc_50() {
    run(0x50);
}

#[test]
fn opc_51() {
    run(0x51);
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
fn opc_58() {
    run(0x58);
}

#[test]
fn opc_59() {
    run(0x59);
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
fn opc_61() {
    run(0x61);
}

#[test]
fn opc_65() {
    run(0x65);
}

#[test]
fn opc_69() {
    run(0x69);
}

#[test]
fn opc_6d() {
    run(0x6D);
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
fn opc_75() {
    run(0x75);
}

#[test]
fn opc_79() {
    run(0x79);
}

#[test]
fn opc_7d() {
    run(0x7D);
}

#[test]
fn opc_81() {
    run(0x81);
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
fn opc_88() {
    run(0x88);
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
fn opc_90() {
    run(0x90);
}

#[test]
fn opc_91() {
    run(0x91);
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
fn opc_99() {
    run(0x99);
}

#[test]
fn opc_9d() {
    run(0x9D);
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
fn opc_a9() {
    run(0xA9);
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
fn opc_b0() {
    run(0xB0);
}

#[test]
fn opc_b1() {
    run(0xB1);
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
fn opc_b8() {
    run(0xB8);
}

#[test]
fn opc_b9() {
    run(0xB9);
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
fn opc_c0() {
    run(0xC0);
}

#[test]
fn opc_c1() {
    run(0xC1);
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
fn opc_d0() {
    run(0xD0);
}

#[test]
fn opc_d1() {
    run(0xD1);
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
fn opc_d8() {
    run(0xD8);
}

#[test]
fn opc_d9() {
    run(0xD9);
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
fn opc_e0() {
    run(0xE0);
}

#[test]
fn opc_e4() {
    run(0xE4);
}

#[test]
fn opc_e6() {
    run(0xE6);
}

#[test]
fn opc_e8() {
    run(0xE8);
}

#[test]
fn opc_ec() {
    run(0xEC);
}

#[test]
fn opc_ee() {
    run(0xEE);
}

#[test]
fn opc_f0() {
    run(0xF0);
}

#[test]
fn opc_f6() {
    run(0xF6);
}

#[test]
fn opc_fe() {
    run(0xFE);
}
