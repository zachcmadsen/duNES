use proc_bitfield::bitfield;

use crate::{bus::Bus, util::word};

macro_rules! update_flags {
    ($cpu:expr, $reg:expr) => {
        $cpu.p.set_z($reg == 0);
        $cpu.p.set_n(($reg & 0x80) != 0);
    };
}

trait BugRead: Bus {
    fn read_word_bugged(&mut self, addr: u16) -> u16;
}

impl<B: Bus> BugRead for B {
    fn read_word_bugged(&mut self, addr: u16) -> u16 {
        let low = self.read_byte(addr);
        // Indirect addressing modes are affected by a hardware bug where reads
        // wrap instead of crossing pages.
        let high = self
            .read_byte((addr as u8).wrapping_add(1) as u16 | (addr & 0xFF00));
        word!(low, high)
    }
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
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        const READ: bool = false;

        let opcode = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

        #[rustfmt::skip]
        match opcode {
            // 0x00 => self.brk(),
            // 0x01 => { self.indexed_indirect(); self.ora(); }
            // 0x02 => self.jam(),
            // 0x03 => { self.indexed_indirect(); self.slo(); }
            // 0x04 => { self.zpg(bus); self.nop(); }
            // 0x05 => { self.zpg(bus); self.ora(); }
            // 0x06 => { self.zpg(bus); self.asl(); }
            // 0x07 => { self.zpg(bus); self.slo(); }
            // 0x08 => self.php(),
            // 0x09 => { self.imm(); self.ora(); }
            // 0x0a => self.asl_accumulator(),
            // 0x0b => { self.imm(); self.anc(); }
            // 0x0c => { self.abs(bus); self.nop(); }
            // 0x0d => { self.abs(bus); self.ora(); }
            // 0x0e => { self.abs(bus); self.asl(); }
            // 0x0f => { self.abs(bus); self.slo(); }
            // 0x10 => self.bpl(),
            // 0x11 => { self.indirect_indexed::<READ>(); self.ora(); }
            // 0x12 => self.jam(),
            // 0x13 => { self.indirect_indexed::<WRITE>(); self.slo(); }
            // 0x14 => { self.zpx(bus); self.nop(); }
            // 0x15 => { self.zpx(bus); self.ora(); }
            // 0x16 => { self.zpx(bus); self.asl(); }
            // 0x17 => { self.zpx(bus); self.slo(); }
            // 0x18 => self.clc(),
            // 0x19 => { self.aby::<READ>(bus); self.ora(); }
            // 0x1a => { self.addr = self.pc; self.nop(); },
            // 0x1b => { self.absolute_indexed::<Y, WRITE>(); self.slo(); }
            // 0x1c => { self.abx::<READ>(bus); self.nop(); }
            // 0x1d => { self.abx::<READ>(bus); self.ora(); }
            // 0x1e => { self.absolute_indexed::<X, WRITE>(); self.asl(); }
            // 0x1f => { self.absolute_indexed::<X, WRITE>(); self.slo(); }
            // 0x20 => self.jsr(),
            // 0x21 => { self.indexed_indirect(); self.and(); }
            // 0x22 => self.jam(),
            // 0x23 => { self.indexed_indirect(); self.rla(); }
            // 0x24 => { self.zpg(bus); self.bit(); }
            // 0x25 => { self.zpg(bus); self.and(); }
            // 0x26 => { self.zpg(bus); self.rol(); }
            // 0x27 => { self.zpg(bus); self.rla(); }
            // 0x28 => self.plp(),
            // 0x29 => { self.imm(); self.and(); }
            // 0x2a => self.rol_accumulator(),
            // 0x2b => { self.imm(); self.anc(); }
            // 0x2c => { self.abs(bus); self.bit(); }
            // 0x2d => { self.abs(bus); self.and(); }
            // 0x2e => { self.abs(bus); self.rol(); }
            // 0x2f => { self.abs(bus); self.rla(); }
            // 0x30 => self.bmi(),
            // 0x31 => { self.indirect_indexed::<READ>(); self.and(); }
            // 0x32 => self.jam(),
            // 0x33 => { self.indirect_indexed::<WRITE>(); self.rla(); }
            // 0x34 => { self.zpx(bus); self.nop(); }
            // 0x35 => { self.zpx(bus); self.and(); }
            // 0x36 => { self.zpx(bus); self.rol(); }
            // 0x37 => { self.zpx(bus); self.rla(); }
            // 0x38 => self.sec(),
            // 0x39 => { self.aby::<READ>(bus); self.and(); }
            // 0x3a => { self.addr = self.pc; self.nop(); },
            // 0x3b => { self.absolute_indexed::<Y, WRITE>(); self.rla(); }
            // 0x3c => { self.abx::<READ>(bus); self.nop(); }
            // 0x3d => { self.abx::<READ>(bus); self.and(); }
            // 0x3e => { self.absolute_indexed::<X, WRITE>(); self.rol(); }
            // 0x3f => { self.absolute_indexed::<X, WRITE>(); self.rla(); }
            // 0x40 => self.rti(),
            // 0x41 => { self.indexed_indirect(); self.eor(); }
            // 0x42 => self.jam(),
            // 0x43 => { self.indexed_indirect(); self.sre(); }
            // 0x44 => { self.zpg(bus); self.nop(); }
            // 0x45 => { self.zpg(bus); self.eor(); }
            // 0x46 => { self.zpg(bus); self.lsr(); }
            // 0x47 => { self.zpg(bus); self.sre(); }
            // 0x48 => self.pha(),
            // 0x49 => { self.imm(); self.eor(); }
            // 0x4a => self.lsr_accumulator(),
            // 0x4b => { self.imm(); self.alr(); }
            0x4C => { self.abs(bus); self.jmp(); }
            // 0x4d => { self.abs(bus); self.eor(); }
            // 0x4e => { self.abs(bus); self.lsr(); }
            // 0x4f => { self.abs(bus); self.sre(); }
            // 0x50 => self.bvc(),
            // 0x51 => { self.indirect_indexed::<READ>(); self.eor(); }
            // 0x52 => self.jam(),
            // 0x53 => { self.indirect_indexed::<WRITE>(); self.sre(); }
            // 0x54 => { self.zpx(bus); self.nop(); }
            // 0x55 => { self.zpx(bus); self.eor(); }
            // 0x56 => { self.zpx(bus); self.lsr(); }
            // 0x57 => { self.zpx(bus); self.sre(); }
            // 0x58 => self.cli(),
            // 0x59 => { self.aby::<READ>(bus); self.eor(); }
            // 0x5a => { self.addr = self.pc; self.nop(); },
            // 0x5b => { self.absolute_indexed::<Y, WRITE>(); self.sre(); }
            // 0x5c => { self.abx::<READ>(bus); self.nop(); }
            // 0x5d => { self.abx::<READ>(bus); self.eor(); }
            // 0x5e => { self.absolute_indexed::<X, WRITE>(); self.lsr(); }
            // 0x5f => { self.absolute_indexed::<X, WRITE>(); self.sre(); }
            // 0x60 => self.rts(),
            // 0x61 => { self.indexed_indirect(); self.adc(); }
            // 0x62 => self.jam(),
            // 0x63 => { self.indexed_indirect(); self.rra(); }
            // 0x64 => { self.zpg(bus); self.nop(); }
            // 0x65 => { self.zpg(bus); self.adc(); }
            // 0x66 => { self.zpg(bus); self.ror(); }
            // 0x67 => { self.zpg(bus); self.rra(); }
            // 0x68 => self.pla(),
            // 0x69 => { self.imm(); self.adc(); }
            // 0x6a => self.ror_accumulator(),
            // 0x6b => { self.imm(); self.arr(); }
            0x6C => { self.ind(bus); self.jmp(); }
            // 0x6d => { self.abs(bus); self.adc(); }
            // 0x6e => { self.abs(bus); self.ror(); }
            // 0x6f => { self.abs(bus); self.rra(); }
            // 0x70 => self.bvs(),
            // 0x71 => { self.indirect_indexed::<READ>(); self.adc(); }
            // 0x72 => self.jam(),
            // 0x73 => { self.indirect_indexed::<WRITE>(); self.rra(); }
            // 0x74 => { self.zpx(bus); self.nop(); }
            // 0x75 => { self.zpx(bus); self.adc(); }
            // 0x76 => { self.zpx(bus); self.ror(); }
            // 0x77 => { self.zpx(bus); self.rra(); }
            // 0x78 => self.sei(),
            // 0x79 => { self.aby::<READ>(bus); self.adc(); }
            // 0x7a => { self.addr = self.pc; self.nop(); },
            // 0x7b => { self.absolute_indexed::<Y, WRITE>(); self.rra(); }
            // 0x7c => { self.abx::<READ>(bus); self.nop(); }
            // 0x7d => { self.abx::<READ>(bus); self.adc(); }
            // 0x7e => { self.absolute_indexed::<X, WRITE>(); self.ror(); }
            // 0x7f => { self.absolute_indexed::<X, WRITE>(); self.rra(); }
            // 0x80 => { self.imm(); self.nop(); }
            // 0x81 => { self.indexed_indirect(); self.sta(); }
            // 0x82 => { self.imm(); self.nop(); }
            // 0x83 => { self.indexed_indirect(); self.sax(); }
            // 0x84 => { self.zpg(bus); self.sty(); }
            // 0x85 => { self.zpg(bus); self.sta(); }
            // 0x86 => { self.zpg(bus); self.stx(); }
            // 0x87 => { self.zpg(bus); self.sax(); }
            // 0x88 => self.dey(),
            // 0x89 => { self.imm(); self.nop(); }
            // 0x8a => self.txa(),
            // 0x8b => { self.imm(); self.ane(); }
            // 0x8c => { self.abs(bus); self.sty(); }
            // 0x8d => { self.abs(bus); self.sta(); }
            // 0x8e => { self.abs(bus); self.stx(); }
            // 0x8f => { self.abs(bus); self.sax(); }
            // 0x90 => self.bcc(),
            // 0x91 => { self.indirect_indexed::<WRITE>(); self.sta(); }
            // 0x92 => self.jam(),
            // 0x93 => { self.absolute_indexed::<Y, WRITE>(); self.sha(); }
            // 0x94 => { self.zpx(bus); self.sty(); }
            // 0x95 => { self.zpx(bus); self.sta(); }
            // 0x96 => { self.zpy(bus); self.stx(); }
            // 0x97 => { self.zpy(bus); self.sax(); }
            // 0x98 => self.tya(),
            // 0x99 => { self.absolute_indexed::<Y, WRITE>(); self.sta(); }
            // 0x9a => self.txs(),
            // 0x9b => { self.absolute_indexed::<Y, WRITE>(); self.tas(); }
            // 0x9c => { self.absolute_indexed::<X, WRITE>(); self.shy(); }
            // 0x9d => { self.absolute_indexed::<X, WRITE>(); self.sta(); }
            // 0x9e => { self.absolute_indexed::<Y, WRITE>(); self.shx(); }
            // 0x9f => { self.indirect_indexed::<WRITE>(); self.sha(); }
            0xA0 => { self.imm(); self.ldy(bus); }
            0xA1 => { self.idx(bus); self.lda(bus); }
            0xA2 => { self.imm(); self.ldx(bus); }
            // 0xA3 => { self.indexed_indirect(); self.lax(); }
            0xA4 => { self.zpg(bus); self.ldy(bus); }
            0xA5 => { self.zpg(bus); self.lda(bus); }
            0xA6 => { self.zpg(bus); self.ldx(bus); }
            // 0xA7 => { self.zpg(bus); self.lax(); }
            // 0xA8 => self.tay(),
            0xA9 => { self.imm(); self.lda(bus); }
            // 0xAA => self.tax(),
            // 0xAB => { self.imm(); self.lxa() },
            0xAC => { self.abs(bus); self.ldy(bus); }
            0xAD => { self.abs(bus); self.lda(bus); }
            0xAE => { self.abs(bus); self.ldx(bus); }
            // 0xAF => { self.abs(bus); self.lax(); }
            // 0xB0 => self.bcs(),
            0xB1 => { self.idy::<READ>(bus); self.lda(bus); }
            // 0xB2 => self.jam(),
            // 0xB3 => { self.indirect_indexed::<READ>(); self.lax(); }
            0xB4 => { self.zpx(bus); self.ldy(bus); }
            0xB5 => { self.zpx(bus); self.lda(bus); }
            0xB6 => { self.zpy(bus); self.ldx(bus); }
            // 0xB7 => { self.zpy(bus); self.lax(); }
            // 0xB8 => self.clv(),
            0xB9 => { self.aby::<READ>(bus); self.lda(bus); }
            // 0xBA => self.tsx(),
            // 0xBB => { self.aby::<READ>(bus); self.las() },
            0xBC => { self.abx::<READ>(bus); self.ldy(bus); }
            0xBD => { self.abx::<READ>(bus); self.lda(bus); }
            0xBE => { self.aby::<READ>(bus); self.ldx(bus); }
            // 0xBF => { self.aby::<READ>(bus); self.lax(); }
            // 0xc0 => { self.imm(); self.cpy(); }
            // 0xc1 => { self.indexed_indirect(); self.cmp(); }
            // 0xc2 => { self.imm(); self.nop(); }
            // 0xc3 => { self.indexed_indirect(); self.dcp(); }
            // 0xc4 => { self.zpg(bus); self.cpy(); }
            // 0xc5 => { self.zpg(bus); self.cmp(); }
            // 0xc6 => { self.zpg(bus); self.dec(); }
            // 0xc7 => { self.zpg(bus); self.dcp(); }
            // 0xc8 => self.iny(),
            // 0xc9 => { self.imm(); self.cmp(); }
            // 0xca => self.dex(),
            // 0xcb => { self.imm(); self.sbx() },
            // 0xcc => { self.abs(bus); self.cpy(); }
            // 0xcd => { self.abs(bus); self.cmp(); }
            // 0xce => { self.abs(bus); self.dec(); }
            // 0xcf => { self.abs(bus); self.dcp(); }
            // 0xd0 => self.bne(),
            // 0xd1 => { self.indirect_indexed::<READ>(); self.cmp(); }
            // 0xd2 => self.jam(),
            // 0xd3 => { self.indirect_indexed::<WRITE>(); self.dcp(); }
            // 0xd4 => { self.zpx(bus); self.nop(); }
            // 0xd5 => { self.zpx(bus); self.cmp(); }
            // 0xd6 => { self.zpx(bus); self.dec(); }
            // 0xd7 => { self.zpx(bus); self.dcp(); }
            // 0xd8 => self.cld(),
            // 0xd9 => { self.aby::<READ>(bus); self.cmp(); }
            // 0xda => { self.addr = self.pc; self.nop(); },
            // 0xdb => { self.absolute_indexed::<Y, WRITE>(); self.dcp(); }
            // 0xdc => { self.abx::<READ>(bus); self.nop(); }
            // 0xdd => { self.abx::<READ>(bus); self.cmp(); }
            // 0xde => { self.absolute_indexed::<X, WRITE>(); self.dec(); }
            // 0xdf => { self.absolute_indexed::<X, WRITE>(); self.dcp(); }
            // 0xe0 => { self.imm(); self.cpx(); }
            // 0xe1 => { self.indexed_indirect(); self.sbc(); }
            // 0xe2 => { self.imm(); self.nop(); }
            // 0xe3 => { self.indexed_indirect(); self.isb(); }
            // 0xe4 => { self.zpg(bus); self.cpx(); }
            // 0xe5 => { self.zpg(bus); self.sbc(); }
            // 0xe6 => { self.zpg(bus); self.inc(); }
            // 0xe7 => { self.zpg(bus); self.isb(); }
            // 0xe8 => self.inx(),
            // 0xe9 => { self.imm(); self.sbc(); }
            // 0xea => { self.addr = self.pc; self.nop(); },
            // 0xeb => { self.imm(); self.sbc(); }
            // 0xec => { self.abs(bus); self.cpx(); }
            // 0xed => { self.abs(bus); self.sbc(); }
            // 0xee => { self.abs(bus); self.inc(); }
            // 0xef => { self.abs(bus); self.isb(); }
            // 0xf0 => self.beq(),
            // 0xf1 => { self.indirect_indexed::<READ>(); self.sbc(); }
            // 0xf2 => self.jam(),
            // 0xf3 => { self.indirect_indexed::<WRITE>(); self.isb(); }
            // 0xf4 => { self.zpx(bus); self.nop(); }
            // 0xf5 => { self.zpx(bus); self.sbc(); }
            // 0xf6 => { self.zpx(bus); self.inc(); }
            // 0xf7 => { self.zpx(bus); self.isb(); }
            // 0xf8 => self.sed(),
            // 0xf9 => { self.aby::<READ>(bus); self.sbc(); }
            // 0xfa => { self.addr = self.pc; self.nop(); },
            // 0xfb => { self.absolute_indexed::<Y, WRITE>(); self.isb(); }
            // 0xfc => { self.abx::<READ>(bus); self.nop(); }
            // 0xfd => { self.abx::<READ>(bus); self.sbc(); }
            // 0xfe => { self.absolute_indexed::<X, WRITE>(); self.inc(); }
            // 0xff => { self.absolute_indexed::<X, WRITE>(); self.isb(); }
            _ => todo!()
        };
    }

    fn next_byte(&mut self, bus: &mut impl Bus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn next_word(&mut self, bus: &mut impl Bus) -> u16 {
        let word = bus.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    fn abs(&mut self, bus: &mut impl Bus) {
        self.addr = self.next_word(bus);
    }

    fn abx<const W: bool>(&mut self, bus: &mut impl Bus) {
        let (low, carry) = self.next_byte(bus).overflowing_add(self.x);
        let high = self.next_byte(bus);
        self.addr = word!(low, high.wrapping_add(carry as u8));

        // The effective address is invalid if it crosses a page. It takes an
        // extra read cycle to fix it.
        if W || carry {
            bus.read_byte(word!(low, high));
        }
    }

    fn aby<const W: bool>(&mut self, bus: &mut impl Bus) {
        let (low, carry) = self.next_byte(bus).overflowing_add(self.y);
        let high = self.next_byte(bus);
        self.addr = word!(low, high.wrapping_add(carry as u8));

        // The effective address is invalid if it crosses a page. It takes an
        // extra read cycle to fix it.
        if W || carry {
            bus.read_byte(word!(low, high));
        }
    }

    fn imm(&mut self) {
        self.addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
    }

    fn idx(&mut self, bus: &mut impl Bus) {
        let ptr = self.next_byte(bus);
        bus.read_byte(ptr as u16);
        self.addr = bus.read_word_bugged(ptr.wrapping_add(self.x) as u16);
    }

    fn idy<const W: bool>(&mut self, bus: &mut impl Bus) {
        let ptr = self.next_byte(bus);
        let (low, carry) = bus.read_byte(ptr as u16).overflowing_add(self.y);
        let high = bus.read_byte(ptr.wrapping_add(1) as u16);
        self.addr = word!(low, high.wrapping_add(carry as u8));

        // The effective address is invalid if it crosses a page. It takes an
        // extra read cycle to fix it.
        if W || carry {
            bus.read_byte(word!(low, high));
        }
    }

    fn ind(&mut self, bus: &mut impl Bus) {
        let ptr = self.next_word(bus);
        self.addr = bus.read_word_bugged(ptr);
    }

    fn zpg(&mut self, bus: &mut impl Bus) {
        self.addr = self.next_byte(bus) as u16;
    }

    fn zpx(&mut self, bus: &mut impl Bus) {
        let addr = self.next_byte(bus);
        bus.read_byte(addr as u16);
        self.addr = addr.wrapping_add(self.x) as u16;
    }

    fn zpy(&mut self, bus: &mut impl Bus) {
        let addr = self.next_byte(bus);
        bus.read_byte(addr as u16);
        self.addr = addr.wrapping_add(self.y) as u16;
    }

    fn jmp(&mut self) {
        self.pc = self.addr;
    }

    fn lda(&mut self, bus: &mut impl Bus) {
        self.a = bus.read_byte(self.addr);
        update_flags!(self, self.a);
    }

    fn ldx(&mut self, bus: &mut impl Bus) {
        self.x = bus.read_byte(self.addr);
        update_flags!(self, self.x);
    }

    fn ldy(&mut self, bus: &mut impl Bus) {
        self.y = bus.read_byte(self.addr);
        update_flags!(self, self.y);
    }
}
