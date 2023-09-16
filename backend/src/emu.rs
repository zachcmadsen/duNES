use crate::{bus::Bus, cpu::Cpu};

pub struct Emu {
    cpu: Cpu,
    ram: [u8; 0x800],
}

impl Emu {
    pub fn step(&mut self) {
        self.cpu.lda(&mut EmuView { _ram: &self.ram });
    }
}

struct EmuView<'a> {
    _ram: &'a [u8; 0x800],
}

impl<'a> Bus for EmuView<'a> {
    fn read_byte(&mut self, _addr: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, _addr: u16, _data: u8) {
        todo!()
    }
}
