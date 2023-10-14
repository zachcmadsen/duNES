use crate::Emu;

#[derive(Clone, Debug)]
struct Handler {
    read: fn(&mut Emu, u16) -> u8,
    write: fn(&mut Emu, u16, u8),
}

pub struct Bus<const N: usize> {
    handlers: Box<[Handler; N]>,
    pub mem: Box<[u8; N]>,

    // The last address and data values on the bus.
    pub addr: u16,
    pub data: u8,
}

impl<const N: usize> Bus<N> {
    pub fn new() -> Bus<N> {
        fn read_default(emu: &mut Emu, addr: u16) -> u8 {
            emu.bus.mem[addr as usize]
        }

        fn write_default(emu: &mut Emu, addr: u16, data: u8) {
            emu.bus.mem[addr as usize] = data;
        }

        let default_handler =
            Handler { read: read_default, write: write_default };

        Bus {
            handlers: vec![default_handler; N].try_into().unwrap(),
            mem: vec![0; N].try_into().unwrap(),

            addr: 0,
            data: 0,
        }
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
