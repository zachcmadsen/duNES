#![cfg_attr(test, allow(dead_code))]

use crate::Emu;

const HEADER_SIZE: u8 = 16;
const PRG_ROM_BANK_SIZE: u16 = 16384;
/// The size of PRG RAM in bytes.
const PRG_RAM_SIZE: u16 = 8192;

pub struct Nrom {
    // These are pub(crate) since the CPU tests need to make an empty version
    // of Nrom.
    pub(crate) prg_ram: Box<[u8]>,
    pub(crate) prg_rom: Box<[u8]>,
}

impl Nrom {
    pub fn new(rom: &[u8]) -> Nrom {
        let (header, rom) = rom.split_at(HEADER_SIZE as usize);
        let prg_rom_size = header[4] as u16 * PRG_ROM_BANK_SIZE;
        Nrom {
            prg_ram: vec![0; PRG_RAM_SIZE as usize].into_boxed_slice(),
            prg_rom: rom[..prg_rom_size as usize].into(),
        }
    }
}

pub fn read_prg_ram(emu: &Emu, addr: u16) -> u8 {
    emu.nrom.prg_ram[(addr - 0x6000) as usize]
}

pub fn write_prg_ram(emu: &mut Emu, addr: u16, data: u8) {
    emu.nrom.prg_ram[(addr - 0x6000) as usize] = data;
}

pub fn read_prg_rom(emu: &Emu, addr: u16) -> u8 {
    emu.nrom.prg_rom[(addr - 0x8000) as usize % emu.nrom.prg_rom.len()]
}

pub fn write_prg_rom(emu: &mut Emu, addr: u16, data: u8) {
    emu.nrom.prg_rom[(addr - 0x8000) as usize % emu.nrom.prg_rom.len()] = data;
}
