use crate::{
    bus::{self, Bus},
    cpu::{self, Cpu, CPU_ADDR_SPACE_SIZE},
    mapper::{self, Nrom},
    ppu::Ppu,
};

pub struct Emu {
    pub(crate) bus: Bus<CPU_ADDR_SPACE_SIZE>,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub(crate) mapper: Nrom,
}

impl Emu {
    pub fn new(rom: &[u8]) -> Emu {
        let mut bus = Bus::new();
        cpu::register(&mut bus);
        mapper::register(&mut bus);

        bus.register(Ppu::read_register, Ppu::write_register, 0x2000..=0x2007);

        Emu { bus, cpu: Cpu::new(), ppu: Ppu::new(), mapper: Nrom::new(rom) }
    }

    pub fn step(&mut self) {
        cpu::step(self);
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
        Ppu::tick(self);
        self.cpu.nmi = self.ppu.nmi;
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        bus::read_byte(self, addr)
    }
}
