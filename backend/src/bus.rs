use std::ops::RangeInclusive;

use crate::Emu;

#[derive(Clone)]
struct Handler {
    read: fn(&mut Emu, u16) -> u8,
    write: fn(&mut Emu, u16, u8),
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

    pub fn register(
        &mut self,
        read_handler: fn(&mut Emu, u16) -> u8,
        write_handler: fn(&mut Emu, u16, u8),
        range: RangeInclusive<u16>,
    ) {
        for handler in &mut self.handlers
            [(*range.start() as usize)..=(*range.end() as usize)]
        {
            handler.read = read_handler;
            handler.write = write_handler;
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
