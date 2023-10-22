use crate::{
    bus::Bus,
    cpu::{self, Cpu},
    mapper::Nrom,
};

/// The size of the CPU address space in bytes;
const CPU_ADDR_SPACE_SIZE: usize = 0x10000;

pub struct Emu {
    pub bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub cpu: Cpu,
    pub(crate) mapper: Nrom,
}

fn ram_read_handler(emu: &mut Emu, addr: u16) -> u8 {
    emu.bus.mem[(addr & 0x07FF) as usize]
}

fn ram_write_handler(emu: &mut Emu, addr: u16, data: u8) {
    emu.bus.mem[(addr & 0x07FF) as usize] = data;
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut bus = Bus::new();
        bus.register(ram_read_handler, ram_write_handler, 0x0000..=0x1FFF);
        let mapper = Nrom::new(rom);
        mapper.register(&mut bus);

        Emu { bus, cpu: Cpu::new(), mapper }
    }

    pub fn step(&mut self) {
        cpu::step(self);
    }
}

impl Default for Emu {
    fn default() -> Self {
        Self {
            bus: Bus::new(),
            cpu: Cpu::new(),
            mapper: Nrom {
                prg_rom: vec![].into_boxed_slice(),
                prg_ram: vec![].into_boxed_slice(),
            },
        }
    }
}
