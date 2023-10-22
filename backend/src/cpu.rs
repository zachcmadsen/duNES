mod instr;
mod lut;
mod mode;

use proc_bitfield::bitfield;

use crate::{
    bus::{self, Bus},
    cpu::lut::OPC_LUT,
    Emu,
};

/// The size of the CPU address space in bytes;
pub const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

/// The size of the CPU RAM in bytes.
#[allow(dead_code)]
const CPU_RAM_SIZE: usize = 2048;

/// The address of the NMI vector.
const _NMI_VECTOR: u16 = 0xFFFA;
/// The address of the reset vector.
const RESET_VECTOR: u16 = 0xFFFC;
/// The address of the IRQ vector.
const IRQ_VECTOR: u16 = 0xFFFE;

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
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    p: Status,

    opc: u16,
    cyc: i8,
    addr: u16,
    carry: bool,
    data: u8,

    #[cfg(not(test))]
    ram: Box<[u8; CPU_RAM_SIZE]>,

    // Some CPU tests assume 64 KB of RAM.
    #[cfg(test)]
    ram: Box<[u8; CPU_ADDR_SPACE_SIZE]>,
}

impl Cpu {
    /// Constructs a new `Cpu` in a power up state.
    pub fn new() -> Cpu {
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

            #[cfg(not(test))]
            ram: vec![0; CPU_RAM_SIZE].try_into().unwrap(),

            #[cfg(test)]
            ram: vec![0; CPU_ADDR_SPACE_SIZE].try_into().unwrap(),
        }
    }
}

pub fn register<const N: usize>(bus: &mut Bus<N>) {
    fn ram_read_handler(emu: &mut Emu, addr: u16) -> u8 {
        emu.cpu.ram[(addr & 0x07FF) as usize]
    }

    fn ram_write_handler(emu: &mut Emu, addr: u16, data: u8) {
        emu.cpu.ram[(addr & 0x07FF) as usize] = data;
    }

    bus.register(ram_read_handler, ram_write_handler, 0x0000..=0x1FFF);
}

pub fn step(emu: &mut Emu) {
    emu.cpu.cyc += 1;
    OPC_LUT[emu.cpu.opc as usize][emu.cpu.cyc as usize](emu);
}

fn next_byte(emu: &mut Emu) -> u8 {
    let byte = bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
    byte
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rkyv::Archive;

    use super::{super::mapper::Nrom, *};

    #[test]
    fn processor_tests() {
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

        fn ram_read_handler(emu: &mut Emu, addr: u16) -> u8 {
            emu.cpu.ram[addr as usize]
        }

        fn ram_write_handler(emu: &mut Emu, addr: u16, data: u8) {
            emu.cpu.ram[addr as usize] = data;
        }

        let mut bus = Bus::new();
        bus.register(ram_read_handler, ram_write_handler, 0x0000..=0xFFFF);

        let mut emu = Emu {
            bus,
            cpu: Cpu::new(),
            // TODO(zach): Use a dummy mapper if we go with trait objects for
            // mappers?
            mapper: Nrom {
                prg_rom: vec![].into_boxed_slice(),
                prg_ram: vec![].into_boxed_slice(),
            },
        };

        // Get through the reset sequence.
        for _ in 0..6 {
            step(&mut emu);
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

                // Use `memset` since `fill` is too slow in debug builds.
                unsafe {
                    libc::memset(
                        emu.cpu.ram.as_mut_ptr() as _,
                        0,
                        emu.cpu.ram.len(),
                    );
                };
                for &(addr, data) in test.initial.ram.iter() {
                    emu.cpu.ram[addr as usize] = data;
                }

                // TODO(zach): Assert read/write cycles.
                for &(addr, data, _) in test.cycles.iter() {
                    step(&mut emu);
                    assert_eq!(emu.bus.addr(), addr);
                    assert_eq!(emu.bus.data(), data);
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
}
