const PRG_RAM_BANK_SIZE: usize = 0x2000;
const PRG_ROM_BANK_SIZE: usize = 0x4000;
const CHR_ROM_BANK_SIZE: usize = 0x2000;

#[derive(Clone, Copy, Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct NromCartridge {
    prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    _mirroring: Mirroring,
}

impl NromCartridge {
    pub fn new(rom: &[u8]) -> NromCartridge {
        // TODO: Completely parse the iNES header. Many values are currently
        // ignored.
        let (header, data) = rom.split_at(16);

        let mapper_number = header[7] & 0xf0 | header[6] >> 4;
        if mapper_number != 0 {
            panic!("only NROM is supported")
        }

        let mirroring = if header[6] & 0x1 == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let num_prg_rom_banks = header[4];
        let num_prg_ram_banks = if header[8] == 0 { 1 } else { header[8] };
        let num_chr_rom_banks = header[5];

        let prg_ram_size = num_prg_ram_banks as usize * PRG_RAM_BANK_SIZE;
        let prg_rom_size = num_prg_rom_banks as usize * PRG_ROM_BANK_SIZE;
        let chr_rom_size = num_chr_rom_banks as usize * CHR_ROM_BANK_SIZE;

        NromCartridge {
            prg_ram: vec![0; prg_ram_size],
            prg_rom: data[..prg_rom_size].to_vec(),
            chr_rom: data[prg_rom_size..(prg_rom_size + chr_rom_size)]
                .to_vec(),
            _mirroring: mirroring,
        }
    }

    pub fn read_prg(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff => {
                self.prg_ram[(address - 0x6000) as usize % self.prg_ram.len()]
            }
            0x8000..=0xffff => {
                self.prg_rom[(address - 0x8000) as usize % self.prg_rom.len()]
            }
            _ => 0,
        }
    }

    pub fn write_prg(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7fff => {
                let address = (address - 0x6000) as usize % self.prg_ram.len();
                self.prg_ram[address] = data;
            }
            _ => unreachable!(),
        }
    }

    pub fn _read_chr(&self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    pub fn _mirroring(&self) -> Mirroring {
        self._mirroring
    }
}
