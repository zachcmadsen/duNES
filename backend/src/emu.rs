use crate::{
    bus::Bus,
    cartridge::NromCartridge,
    cpu::{self, Cpu},
};

/// The size of the CPU address space in bytes;
const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

pub struct Emu {
    pub bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub cpu: Cpu,
    pub(crate) cart: Option<NromCartridge>,
}

fn ram_read_handler(emu: &mut Emu, addr: u16) -> u8 {
    emu.bus.mem[(addr & 0x07FF) as usize]
}

fn ram_write_handler(emu: &mut Emu, addr: u16, data: u8) {
    emu.bus.mem[(addr & 0x07FF) as usize] = data;
}

fn cartridge_read_handler(emu: &mut Emu, addr: u16) -> u8 {
    emu.cart.as_ref().unwrap().read_prg(addr)
}

fn cartridge_write_handler(emu: &mut Emu, addr: u16, data: u8) {
    emu.cart.as_mut().unwrap().write_prg(addr, data);
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut bus = Bus::new();
        bus.register(ram_read_handler, ram_write_handler, 0x0000..=0x1FFF);
        bus.register(
            cartridge_read_handler,
            cartridge_write_handler,
            0x4020..=0xFFFF,
        );

        Emu { bus, cpu: Cpu::new(), cart: Some(NromCartridge::new(rom)) }
    }

    pub fn step(&mut self) {
        cpu::step(self);
    }
}

impl Default for Emu {
    fn default() -> Self {
        Self { bus: Bus::new(), cpu: Cpu::new(), cart: None }
    }
}
