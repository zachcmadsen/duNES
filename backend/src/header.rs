pub struct INesHeader {
    pub prg_rom_banks: u8,
    pub _chr_rom_banks: u8,
    _mapper: u8,
    pub prg_ram_banks: u8,
}

impl INesHeader {
    pub fn new(header: &[u8]) -> INesHeader {
        assert!(header[0..4] == [b'N', b'E', b'S', 0x1A]);

        let prg_rom_banks = header[4];
        // TODO(zach): If header[5] == 0, then the cart uses CHR RAM. Do I need
        // to handle that?
        let chr_rom_banks = header[5];
        let mapper = header[6] >> 4 | header[7] & 0xF0;
        let prg_ram_banks = if header[8] == 0 { 1 } else { header[8] };

        INesHeader {
            prg_rom_banks,
            _chr_rom_banks: chr_rom_banks,
            _mapper: mapper,
            prg_ram_banks,
        }
    }
}
