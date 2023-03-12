use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use dunes::{Bus, Cpu};

const ZERO_PAGE_START: usize = 0xa;
const CODE_SEGMENT_START: u16 = 0x400;
const INTERRUPT_FEEDBACK_REGISTER: u16 = 0xbffc;
const IRQ_MASK: u8 = 0x1;
const NMI_MASK: u8 = 0x2;

const KLAUS_FUNC_ROM: &[u8] =
    include_bytes!("../tests/roms/6502_functional_test.bin");

#[derive(Clone)]
struct KlausBus {
    memory: Box<[u8; 0x10000]>,
}

impl KlausBus {
    fn new() -> KlausBus {
        let mut memory: Box<[u8; 0x10000]> =
            vec![0; 0x10000].try_into().unwrap();
        memory[ZERO_PAGE_START..].copy_from_slice(KLAUS_FUNC_ROM);

        KlausBus { memory }
    }
}

impl Bus for KlausBus {
    fn read(&mut self, pins: &mut dunes::Pins) {
        pins.data = self.memory[pins.address as usize];
    }

    fn write(&mut self, pins: &mut dunes::Pins) {
        if pins.address == INTERRUPT_FEEDBACK_REGISTER {
            let old_data = self.memory[pins.address as usize];
            let prev_nmi = old_data & NMI_MASK != 0;
            let new_nmi = pins.data & NMI_MASK != 0;

            pins.irq = pins.data & IRQ_MASK != 0;
            pins.nmi = !prev_nmi && new_nmi;
        }

        self.memory[pins.address as usize] = pins.data;
    }
}

fn run(cpu: &mut Cpu<KlausBus>) {
    cpu.step();
    cpu.pc = CODE_SEGMENT_START;

    for _ in 0..26765881 {
        cpu.step();
    }
}

fn functional(c: &mut Criterion) {
    let bus = KlausBus::new();

    c.bench_function("functional", |b| {
        b.iter_batched(
            || Cpu::new(bus.clone()),
            |mut cpu| run(&mut cpu),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(15);
    targets = functional
);
criterion_main!(benches);
