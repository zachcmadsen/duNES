mod instr;
mod lut;
mod mode;

use proc_bitfield::bitfield;

use crate::{
    bus::{self, Bus},
    cpu::lut::OPC_LUT,
    emu::Emu,
};

/// The size of the CPU address space in bytes.
pub const ADDR_SPACE_SIZE: usize = 0x10000;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Status(pub u8) {
        c: bool @ 0,
        z: bool @ 1,
        i: bool @ 2,
        d: bool @ 3,
        b: bool @ 4,
        u: bool @ 5,
        v: bool @ 6,
        n: bool @ 7,
    }
}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: Status,

    opc: u16,
    cyc: i8,
    addr: u16,
    carry: bool,
    data: u8,

    pub(crate) nmi: bool,
    irq: bool,

    prev_nmi: bool,
    pending_nmi: bool,
    pending_irq: bool,

    pub(crate) bus: Bus,

    // Some CPU tests assume 64 KB of RAM.
    #[cfg(test)]
    ram: Box<[u8; ADDR_SPACE_SIZE]>,
}

impl Cpu {
    /// Constructs a new `Cpu` in a power up state.
    pub fn new(bus: Bus) -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            p: Status(0x34),

            // TODO(zach): Explain the initial values of `opc` and `cyc`.
            opc: 0x100,
            cyc: -1,
            addr: 0,
            carry: false,
            data: 0,

            nmi: false,
            irq: false,

            prev_nmi: false,
            pending_nmi: false,
            pending_irq: false,

            bus,

            #[cfg(test)]
            ram: vec![0; ADDR_SPACE_SIZE].try_into().unwrap(),
        }
    }
}

pub fn tick(emu: &mut Emu) {
    emu.cpu.cyc += 1;
    OPC_LUT[emu.cpu.opc as usize][emu.cpu.cyc as usize](emu);

    emu.cpu.pending_nmi |= !emu.cpu.prev_nmi && emu.cpu.nmi;
    emu.cpu.prev_nmi = emu.cpu.nmi;
    emu.cpu.pending_irq = !emu.cpu.p.i() && emu.cpu.irq;
}

fn next_byte(emu: &mut Emu) -> u8 {
    let byte = bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
    byte
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn make_test_emu() -> Emu {
        use crate::{
            apu::Apu,
            emu::RAM_SIZE,
            mapper::{Mirroring, Nrom},
            ppu::Ppu,
        };

        fn read_ram(emu: &mut Emu, addr: u16) -> u8 {
            emu.cpu.ram[addr as usize]
        }

        fn write_ram(emu: &mut Emu, addr: u16, data: u8) {
            emu.cpu.ram[addr as usize] = data;
        }

        let mut bus = Bus::new();
        bus.set(0x0000..=0xFFFF, Some(read_ram), Some(write_ram));

        Emu {
            apu: Apu::new(),
            cpu: Cpu::new(bus),
            ppu: Ppu::new(),
            mapper: Nrom {
                prg_rom: vec![].into_boxed_slice(),
                prg_ram: vec![].into_boxed_slice(),
                chr_rom: vec![].into_boxed_slice(),
                mirroring: Mirroring::Horizontal,
            },
            ram: vec![0; RAM_SIZE].try_into().unwrap(),
        }
    }

    #[test]
    fn processor_tests() {
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

        let mut emu = make_test_emu();

        for _ in 0..6 {
            tick(&mut emu);
        }

        // Skip ANE, JAM, and LXA opcodes.
        const OPC_TO_SKIP: &[u8] = &[
            0x02, 0x12, 0x22, 0x32, 0x42, 0x52, 0x62, 0x72, 0x8B, 0x92, 0xAB,
            0xB2, 0xD2, 0xF2,
        ];
        for opc in 0x00..=0xFF {
            if OPC_TO_SKIP.contains(&opc) {
                continue;
            }

            let bytes =
                fs::read(format!("../roms/processor_tests/{:02x}.rkyv", opc))
                    .unwrap();
            let tests = unsafe { rkyv::archived_root::<Vec<Test>>(&bytes) };

            for test in tests.iter() {
                emu.cpu.a = test.initial.a;
                emu.cpu.x = test.initial.x;
                emu.cpu.y = test.initial.y;
                emu.cpu.pc = test.initial.pc;
                emu.cpu.s = test.initial.s;
                emu.cpu.p = Status(test.initial.p);

                for &(addr, data) in test.initial.ram.iter() {
                    emu.cpu.ram[addr as usize] = data;
                }

                // TODO: Assert read/write cycles.
                for &(addr, data, _) in test.cycles.iter() {
                    tick(&mut emu);
                    assert_eq!(emu.cpu.bus.addr(), addr);
                    assert_eq!(emu.cpu.bus.data(), data);
                }

                assert_eq!(emu.cpu.a, test.r#final.a);
                assert_eq!(emu.cpu.x, test.r#final.x);
                assert_eq!(emu.cpu.y, test.r#final.y);
                assert_eq!(emu.cpu.pc, test.r#final.pc);
                assert_eq!(emu.cpu.s, test.r#final.s);
                assert_eq!(emu.cpu.p.0, test.r#final.p);
                for &(addr, data) in test.r#final.ram.iter() {
                    assert_eq!(emu.cpu.ram[addr as usize], data);
                }
            }
        }
    }

    mod klaus {
        use super::*;

        fn run(emu: &mut Emu, success_addr: u16) {
            for _ in 0..6 {
                tick(emu);
            }

            emu.cpu.pc = 0x0400;
            let mut prev_pc = emu.cpu.pc;

            loop {
                tick(emu);

                let is_start_of_instr = emu.cpu.cyc as usize
                    == OPC_LUT[emu.cpu.opc as usize].len() - 2;
                if is_start_of_instr {
                    if prev_pc == emu.cpu.pc {
                        if emu.cpu.pc == success_addr {
                            break;
                        }

                        panic!("trapped at 0x{:04X}", emu.cpu.pc);
                    }

                    prev_pc = emu.cpu.pc;
                }
            }
        }

        #[test]
        fn functional() {
            let rom =
                fs::read("../roms/klaus/6502_functional_test.bin").unwrap();

            let mut emu = make_test_emu();
            emu.cpu.ram[0x000A..].copy_from_slice(&rom);

            run(&mut emu, 0x336D);
        }

        #[test]
        fn interrupt() {
            fn write_ram(emu: &mut Emu, addr: u16, data: u8) {
                const IRQ_MASK: u8 = 0b00000001;
                const NMI_MASK: u8 = 0b00000010;

                if addr == 0xBFFC {
                    let prev_data = emu.cpu.ram[addr as usize];
                    let prev_nmi = prev_data & NMI_MASK != 0;
                    let new_nmi = data & NMI_MASK != 0;

                    emu.cpu.irq = data & IRQ_MASK != 0;
                    emu.cpu.nmi = !prev_nmi && new_nmi;
                }

                emu.cpu.ram[addr as usize] = data;
            }

            let rom =
                fs::read("../roms/klaus/6502_interrupt_test.bin").unwrap();

            let mut emu = make_test_emu();
            emu.cpu.bus.set(0x0000..=0xFFFF, None, Some(write_ram));
            emu.cpu.ram[0x000A..].copy_from_slice(&rom);

            run(&mut emu, 0x06F5);
        }
    }
}
