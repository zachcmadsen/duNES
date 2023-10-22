use crate::{header::INesHeader, Bus, Emu};

/// The size of the iNES file header in bytes.
const INES_HEADER_SIZE: usize = 16;
/// The size of PRG RAM banks in bytes.
const PRG_RAM_BANK_SIZE: usize = 8192;
/// The size of PRG ROM banks in bytes.
const PRG_ROM_BANK_SIZE: usize = 16384;

pub struct Nrom {
    pub prg_ram: Box<[u8]>,
    pub prg_rom: Box<[u8]>,
}

impl Nrom {
    pub fn new(rom: &[u8]) -> Nrom {
        let header = INesHeader::new(&rom[..INES_HEADER_SIZE]);

        Nrom {
            prg_ram: vec![
                0;
                header.prg_ram_banks as usize * PRG_RAM_BANK_SIZE
            ]
            .into_boxed_slice(),
            prg_rom: rom[INES_HEADER_SIZE
                ..(INES_HEADER_SIZE
                    + (header.prg_rom_banks as usize * PRG_ROM_BANK_SIZE))]
                .into(),
        }
    }

    pub fn register<const N: usize>(&self, bus: &mut Bus<N>) {
        bus.register(
            prg_ram_read_handler,
            prg_ram_write_handler,
            0x6000..=0x7FFF,
        );
        bus.register(
            prg_rom_read_handler,
            prg_rom_write_handler,
            0x8000..=0xFFFF,
        );
    }
}

fn prg_ram_read_handler(emu: &mut Emu, addr: u16) -> u8 {
    emu.mapper.prg_ram[(addr - 0x6000) as usize % emu.mapper.prg_ram.len()]
}

fn prg_ram_write_handler(emu: &mut Emu, addr: u16, data: u8) {
    emu.mapper.prg_ram[(addr - 0x6000) as usize % emu.mapper.prg_ram.len()] =
        data;
}

fn prg_rom_read_handler(emu: &mut Emu, addr: u16) -> u8 {
    emu.mapper.prg_rom[(addr - 0x8000) as usize % emu.mapper.prg_rom.len()]
}

fn prg_rom_write_handler(_: &mut Emu, _: u16, _: u8) {}
