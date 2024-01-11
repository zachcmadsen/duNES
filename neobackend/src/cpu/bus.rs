use crate::{emu::Emu, scheduler};

/// The size of the CPU's internal ram in bytes.
const RAM_SIZE: u16 = 0x0800;

pub struct Bus {
    /// The CPU's internal RAM.
    ram: Box<[u8; RAM_SIZE as usize]>,
    /// The last address on the bus.
    addr: u16,
    /// The last data on the bus.
    data: u8,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: vec![0; RAM_SIZE as usize].try_into().unwrap(),
            addr: 0,
            data: 0,
        }
    }
}

/// Reads the byte at address `addr`.
pub fn read(emu: &mut Emu, addr: u16) -> u8 {
    scheduler::tick(emu);

    let data = match addr {
        // 0x0800-0x1FFF are mirrors of 0x0000-0x07FF.
        0x0000..=0x1FFF => emu.cpu.bus.ram[(addr & 0x07FF) as usize],
        0x2000..=0x401F => 0,
        0x6000..=0x7FFF => emu.nrom.read_prg_ram(addr),
        0x8000..=0xFFFF => emu.nrom.read_prg_rom(addr),
        _ => todo!(),
    };
    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    data
}

/// Writes `data` to address `addr`.
pub fn write(emu: &mut Emu, addr: u16, data: u8) {
    scheduler::tick(emu);

    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    match addr {
        // 0x0800-0x1FFF are mirrors of 0x0000-0x07FF.
        0x0000..=0x1FFF => emu.cpu.bus.ram[(addr & 0x07FF) as usize] = data,
        0x2000..=0x401F => (),
        0x6000..=0x7FFF => emu.nrom.write_prg_ram(addr, data),
        0x8000..=0xFFFF => emu.nrom.write_prg_rom(addr, data),
        _ => todo!(),
    };
}

pub fn peek(emu: &mut Emu, addr: u16) -> Option<u8> {
    match addr {
        // 0x0800-0x1FFF are mirrors of 0x0000-0x07FF.
        0x0000..=0x1FFF => Some(emu.cpu.bus.ram[(addr & 0x07FF) as usize]),
        // TODO: Do any mappers have side effects on reads? If they do, we need
        // to call out to a peek method on the mappers that disables side
        // effects.
        0x6000..=0x7FFF => Some(emu.nrom.read_prg_ram(addr)),
        0x8000..=0xFFFF => Some(emu.nrom.read_prg_rom(addr)),
        _ => None,
    }
}
