use crate::bus::Bus;

pub struct Cpu {
    a: u8,
    pc: u16,
}

impl Cpu {
    pub fn lda(&mut self, bus: &mut impl Bus) {
        self.a = bus.read_byte(self.pc);

        // TODO: Update flags.
    }
}
