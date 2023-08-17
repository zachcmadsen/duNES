use std::fs;

use backend::{Cpu, DuNesBus, NromCartridge, Status};

struct NestestLog {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cycles: u64,
}

#[test]
fn nestest() {
    let rom = fs::read("../roms/nestest/nestest.nes").unwrap();
    let logs: Vec<NestestLog> =
        fs::read_to_string("../roms/nestest/nestest.log")
            .unwrap()
            .lines()
            .map(|line| {
                let pc = u16::from_str_radix(&line[0..4], 16).unwrap();
                let a = u8::from_str_radix(&line[50..52], 16).unwrap();
                let x = u8::from_str_radix(&line[55..57], 16).unwrap();
                let y = u8::from_str_radix(&line[60..62], 16).unwrap();
                let p = u8::from_str_radix(&line[65..67], 16).unwrap();
                let sp = u8::from_str_radix(&line[71..73], 16).unwrap();
                let cycles = line[90..].parse().unwrap();

                NestestLog { pc, a, x, y, p, sp, cycles }
            })
            .collect();

    let cartridge = NromCartridge::new(&rom);
    let bus = DuNesBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    // The nestest log has different initial values for the program counter,
    // status register, and stack pointer.
    cpu.step();
    cpu.pc = 0xc000;
    cpu.p = Status::from(0x24);
    cpu.s = 0xfd;

    for log in logs {
        assert_eq!(cpu.pc, log.pc);
        assert_eq!(cpu.a, log.a);
        assert_eq!(cpu.x, log.x);
        assert_eq!(cpu.y, log.y);
        assert_eq!(u8::from(cpu.p), log.p);
        assert_eq!(cpu.s, log.sp);
        assert_eq!(cpu.cycles, log.cycles);

        cpu.step();
    }
}
