use crate::{header::INesHeader, Emu};

/// The size of the iNES file header in bytes.
const INES_HEADER_SIZE: usize = 16;
/// The size of PRG RAM banks in bytes.
const PRG_RAM_BANK_SIZE: usize = 8192;
/// The size of PRG ROM banks in bytes.
const PRG_ROM_BANK_SIZE: usize = 16384;
/// The size of CHR ROM banks in bytes.
const CHR_ROM_BANK_SIZE: usize = 8192;

pub enum Mirroring {
    Horizontal,
    Vertical,
}

pub struct Nrom {
    pub(crate) prg_ram: Box<[u8]>,
    pub(crate) prg_rom: Box<[u8]>,
    pub(crate) chr_rom: Box<[u8]>,
    pub(crate) mirroring: Mirroring,
}

impl Nrom {
    pub fn new(rom: &[u8]) -> Nrom {
        let header = INesHeader::new(&rom[..INES_HEADER_SIZE]);

        let prg_rom_size = header.prg_rom_banks as usize * PRG_ROM_BANK_SIZE;
        let chr_rom_size = header.chr_rom_banks as usize * CHR_ROM_BANK_SIZE;

        Nrom {
            prg_ram: vec![
                0;
                header.prg_ram_banks as usize * PRG_RAM_BANK_SIZE
            ]
            .into_boxed_slice(),
            prg_rom: rom[INES_HEADER_SIZE..(INES_HEADER_SIZE + prg_rom_size)]
                .into(),
            chr_rom: rom[(INES_HEADER_SIZE + prg_rom_size)
                ..(INES_HEADER_SIZE + prg_rom_size + chr_rom_size)]
                .into(),
            mirroring: header.mirroring,
        }
    }

    pub(crate) fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }

    pub(crate) fn mirroring(&self) -> &Mirroring {
        &self.mirroring
    }
}

pub fn read_prg_ram(emu: &mut Emu, addr: u16) -> u8 {
    emu.mapper.prg_ram[(addr - 0x6000) as usize % emu.mapper.prg_ram.len()]
}

pub fn write_prg_ram(emu: &mut Emu, addr: u16, data: u8) {
    emu.mapper.prg_ram[(addr - 0x6000) as usize % emu.mapper.prg_ram.len()] =
        data;
}

pub fn read_prg_rom(emu: &mut Emu, addr: u16) -> u8 {
    emu.mapper.prg_rom[(addr - 0x8000) as usize % emu.mapper.prg_rom.len()]
}

pub fn write_prg_rom(_: &mut Emu, _: u16, _: u8) {}
