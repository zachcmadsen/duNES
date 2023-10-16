mod instr;
mod lut;
mod mode;

use proc_bitfield::bitfield;

use crate::{bus, cpu::lut::OPC_LUT, Emu};

const IRQ_VECTOR: u16 = 0xFFFE;
const RESET_VECTOR: u16 = 0xFFFC;

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

    pub(crate) sync: bool,

    opc: u16,
    cyc: i8,
    addr: u16,
    carry: bool,
    data: u8,
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

            sync: false,

            // TODO(zach): Explain the initial values of `opc` and `cyc`.
            opc: 0x100,
            cyc: -1,
            addr: 0,
            carry: false,
            data: 0,
        }
    }
}

pub fn step(emu: &mut Emu) {
    // println!("opc: 0x{:02X}", emu.cpu.opc);
    emu.cpu.cyc += 1;
    emu.cpu.sync = false;
    if (OPC_LUT[emu.cpu.opc as usize].len() == 0) {
        println!("missing opc: {:02X}", emu.cpu.opc);
    }
    OPC_LUT[emu.cpu.opc as usize][emu.cpu.cyc as usize](emu);
}

fn next_byte(emu: &mut Emu) -> u8 {
    let byte = bus::read_byte(emu, emu.cpu.pc);
    emu.cpu.pc = emu.cpu.pc.wrapping_add(1);
    byte
}
