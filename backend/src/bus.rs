use std::ops::RangeInclusive;

use crate::{cpu::ADDR_SPACE_SIZE, emu::Emu};

pub struct Bus {
    readers: Box<[fn(&mut Emu, u16) -> u8; ADDR_SPACE_SIZE]>,
    writers: Box<[fn(&mut Emu, u16, u8); ADDR_SPACE_SIZE]>,

    /// The last address value on the bus.
    addr: u16,
    /// The last data value on the bus.
    data: u8,
}

impl Bus {
    pub fn new() -> Bus {
        fn read_default(emu: &mut Emu, _: u16) -> u8 {
            emu.cpu.bus.data
        }

        fn write_default(_: &mut Emu, _: u16, _: u8) {}

        // TODO: Use the box keyword to avoid the array stack allocations?
        let readers = Box::<[fn(&mut Emu, u16) -> u8; ADDR_SPACE_SIZE]>::new(
            [read_default; ADDR_SPACE_SIZE],
        );
        let writers = Box::<[fn(&mut Emu, u16, u8); ADDR_SPACE_SIZE]>::new(
            [write_default; ADDR_SPACE_SIZE],
        );

        Bus { readers, writers, addr: 0, data: 0 }
    }

    pub fn set(
        &mut self,
        range: RangeInclusive<u16>,
        reader: Option<fn(&mut Emu, u16) -> u8>,
        writer: Option<fn(&mut Emu, u16, u8)>,
    ) {
        for addr in range {
            if let Some(reader) = reader {
                self.readers[addr as usize] = reader;
            }
            if let Some(writer) = writer {
                self.writers[addr as usize] = writer;
            }
        }
    }

    #[cfg(test)]
    pub fn addr(&self) -> u16 {
        self.addr
    }

    #[cfg(test)]
    pub fn data(&self) -> u8 {
        self.data
    }
}

pub fn read_byte(emu: &mut Emu, addr: u16) -> u8 {
    let data = (emu.cpu.bus.readers[addr as usize])(emu, addr);
    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    data
}

pub fn write_byte(emu: &mut Emu, addr: u16, data: u8) {
    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    (emu.cpu.bus.writers[addr as usize])(emu, addr, data);
}
