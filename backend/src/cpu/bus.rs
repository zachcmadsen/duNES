use crate::{apu, emu::Emu, nrom, scheduler};

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
    apu::tick(emu);

    let data = match addr {
        // 0x0800-0x1FFF are mirrors of 0x0000-0x07FF.
        0x0000..=0x1FFF => emu.cpu.bus.ram[(addr & 0x07FF) as usize],
        0x2000..=0x4014 => 0,
        0x4015 => apu::read(emu),
        0x4016..=0x401F => 0,
        0x6000..=0x7FFF => nrom::read_prg_ram(emu, addr),
        0x8000..=0xFFFF => nrom::read_prg_rom(emu, addr),
        _ => todo!(),
    };
    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    data
}

/// Writes `data` to address `addr`.
pub fn write(emu: &mut Emu, addr: u16, data: u8) {
    scheduler::tick(emu);
    apu::tick(emu);

    emu.cpu.bus.addr = addr;
    emu.cpu.bus.data = data;
    match addr {
        // 0x0800-0x1FFF are mirrors of 0x0000-0x07FF.
        0x0000..=0x1FFF => emu.cpu.bus.ram[(addr & 0x07FF) as usize] = data,
        0x2000..=0x3FFF => (),
        0x4000..=0x4017 => apu::write(emu, addr, data),
        0x4018..=0x401F => (),
        0x6000..=0x7FFF => nrom::write_prg_ram(emu, addr, data),
        0x8000..=0xFFFF => nrom::write_prg_rom(emu, addr, data),
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
        0x6000..=0x7FFF => Some(nrom::read_prg_ram(emu, addr)),
        0x8000..=0xFFFF => Some(nrom::read_prg_rom(emu, addr)),
        _ => None,
    }
}
