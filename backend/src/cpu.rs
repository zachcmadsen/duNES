use proc_bitfield::bitfield;

use crate::bus::Bus;

macro_rules! update_flags {
    ($cpu:expr, $reg:expr) => {
        $cpu.p.set_z($reg == 0);
        $cpu.p.set_n(($reg & 0x80) != 0);
    };
}

bitfield! {
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

    /// The effective address.
    addr: u16,
}

impl Cpu {
    /// Constructs a new `Cpu` in a power up state.
    pub const fn new() -> Cpu {
        Cpu { a: 0, x: 0, y: 0, pc: 0, s: 0xFD, p: Status(0x34), addr: 0 }
    }

    /// Executes the next instruction.
    pub fn step(&mut self, bus: &mut impl Bus) {
        let _opcode = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

        self.imm(bus);
        self.lda(bus);
    }

    fn imm(&mut self, _: &mut impl Bus) {
        self.addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
    }

    /// Loads the accumulator into memory.
    fn lda(&mut self, bus: &mut impl Bus) {
        self.a = bus.read_byte(self.addr);
        update_flags!(self, self.a);
    }
}
