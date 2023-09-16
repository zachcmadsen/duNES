use std::fs;

use backend::{Bus, Cpu, Status};
use serde::Deserialize;

#[derive(Deserialize)]
struct TestCase {
    initial: CpuState,
    r#final: CpuState,
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
    pub mem: Box<[u8; 0x10000]>,
}

impl Bus for TestBus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data;
    }
}

#[test]
fn lda_imm() {
    let bytes = fs::read("../roms/processor_tests/a9.json").unwrap();
    let tests: Vec<TestCase> = serde_json::from_slice(&bytes).unwrap();

    let mut bus = TestBus { mem: vec![0; 0x10000].try_into().unwrap() };
    let mut cpu = Cpu::new();

    for test in tests {
        cpu.a = test.initial.a;
        cpu.x = test.initial.x;
        cpu.y = test.initial.y;
        cpu.pc = test.initial.pc;
        cpu.s = test.initial.s;
        cpu.p = Status(test.initial.p);
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
    }
}
