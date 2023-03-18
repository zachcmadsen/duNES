use std::fs;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use dunes::{Bus, Cpu};

const ZERO_PAGE_START_ADDRESS: usize = 0xa;
const CODE_SEGMENT_START_ADDRESS: u16 = 0x400;
const SUCCESS_ADDRESS: u16 = 0x336d;

#[derive(Clone)]
struct KlausBus {
    memory: Box<[u8; 0x10000]>,
}

impl KlausBus {
    fn new(rom: &[u8]) -> KlausBus {
        let mut memory: Box<[u8; 0x10000]> =
            vec![0; 0x10000].try_into().unwrap();
        memory[ZERO_PAGE_START_ADDRESS..].copy_from_slice(rom);

        KlausBus { memory }
    }
}

impl Bus for KlausBus {
    fn read(&mut self, pins: &mut dunes::Pins) {
        pins.data = self.memory[pins.address as usize];
    }

    fn write(&mut self, pins: &mut dunes::Pins) {
        self.memory[pins.address as usize] = pins.data;
    }
}

fn functional(c: &mut Criterion) {
    let rom = fs::read("tests/roms/6502_functional_test.bin")
        .expect("tests/roms/6502_functional_test.bin should exist");
    let bus = KlausBus::new(&rom);

    c.bench_function("functional", |b| {
        b.iter_batched(
            || Cpu::new(bus.clone()),
            |mut cpu| {
                cpu.step();
                cpu.pc = CODE_SEGMENT_START_ADDRESS;

                while cpu.pc != SUCCESS_ADDRESS {
                    cpu.step();
                }
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = functional
);
criterion_main!(benches);
