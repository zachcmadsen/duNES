use std::fs;

use backend::{Bus, Cpu, Status};
use serde::Deserialize;

const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

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

struct TestBus {
    pub mem: Box<[u8; CPU_ADDR_SPACE_SIZE]>,
    pub cycles: Vec<(u16, u8, &'static str)>,
}

impl Bus for TestBus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        let data = self.mem[addr as usize];
        self.cycles.push((addr, data, "read"));
        data
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        self.cycles.push((addr, data, "write"));
        self.mem[addr as usize] = data;
    }
}

fn run(opcode: u8) {
    let bytes =
        fs::read(format!("../roms/processor_tests/{:02x}.json", opcode))
            .unwrap();
    let tests: Vec<TestCase> = serde_json::from_slice(&bytes).unwrap();

    let mut bus = TestBus {
        mem: vec![0; CPU_ADDR_SPACE_SIZE].try_into().unwrap(),
        cycles: Vec::new(),
    };
    let mut cpu = Cpu::new();

    for test in tests {
        cpu.a = test.initial.a;
        cpu.x = test.initial.x;
        cpu.y = test.initial.y;
        cpu.pc = test.initial.pc;
        cpu.s = test.initial.s;
        cpu.p = Status(test.initial.p);

        // Use `memset` since `fill` is too slow in debug builds.
        unsafe { libc::memset(bus.mem.as_mut_ptr() as _, 0, bus.mem.len()) };
        bus.cycles.clear();
        for (addr, data) in test.initial.ram {
            bus.mem[addr as usize] = data;
        }

        cpu.step(&mut bus);

        assert_eq!(cpu.a, test.r#final.a);
        assert_eq!(cpu.x, test.r#final.x);
        assert_eq!(cpu.y, test.r#final.y);
        assert_eq!(cpu.pc, test.r#final.pc);
        assert_eq!(cpu.s, test.r#final.s);
        assert_eq!(cpu.p.0, test.r#final.p);
        for (addr, data) in test.r#final.ram {
            assert_eq!(bus.mem[addr as usize], data);
        }
        assert_eq!(bus.cycles, test.cycles);
    }
}

#[test]
fn opc_4c() {
    run(0x4C);
}

#[test]
fn opc_6c() {
    run(0x6C);
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