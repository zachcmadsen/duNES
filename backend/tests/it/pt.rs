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
            libc::memset(emu.bus.mem.as_mut_ptr() as _, 0, emu.bus.mem.len())
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
fn opc_ac() {
    run(0xAC);
}

#[test]
fn opc_ad() {
    run(0xAD);
}

#[test]
fn opc_a9() {
    run(0xA9);
}

#[test]
fn opc_ae() {
    run(0xAE);
}

#[test]
fn opc_b1() {
    run(0xB1);
}

#[test]
fn opc_b5() {
    run(0xB5);
}

#[test]
fn opc_b4() {
    run(0xB4);
}

#[test]
fn opc_b6() {
    run(0xB6);
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
