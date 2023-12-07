use std::ops::RangeInclusive;

use crate::Emu;

type ReadHandler = fn(&mut Emu, u16) -> u8;
type WriteHandler = fn(&mut Emu, u16, u8);

#[derive(Clone)]
struct Handler {
    read: ReadHandler,
    write: WriteHandler,
}

pub struct Bus<const N: usize> {
    handlers: Box<[Handler; N]>,

    // The last address and data values on the bus.
    addr: u16,
    data: u8,
}

impl<const N: usize> Bus<N> {
    pub fn new() -> Bus<N> {
        fn default_read_handler(emu: &mut Emu, _: u16) -> u8 {
            emu.bus.data
        }

        fn default_write_handler(_: &mut Emu, _: u16, _: u8) {}

        let default_handler = Handler {
            read: default_read_handler,
            write: default_write_handler,
        };
        let Ok(handlers) = vec![default_handler; N].try_into() else {
            panic!("the conversion to a boxed array failed")
        };

        Bus { handlers, addr: 0, data: 0 }
    }

    pub fn set(
        &mut self,
        range: RangeInclusive<u16>,
        read: Option<ReadHandler>,
        write: Option<WriteHandler>,
    ) {
        for handler in &mut self.handlers
            [(*range.start() as usize)..=(*range.end() as usize)]
        {
            if let Some(read) = read {
                handler.read = read;
            }
            if let Some(write) = write {
                handler.write = write;
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
    let data = (emu.bus.handlers[addr as usize].read)(emu, addr);
    emu.bus.addr = addr;
    emu.bus.data = data;
    data
}

pub fn write_byte(emu: &mut Emu, addr: u16, data: u8) {
    emu.bus.addr = addr;
    emu.bus.data = data;
    (emu.bus.handlers[addr as usize].write)(emu, addr, data);
}
