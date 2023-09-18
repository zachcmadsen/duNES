use crate::util::word;

pub trait Bus {
    fn read_byte(&mut self, addr: u16) -> u8;

    fn write_byte(&mut self, addr: u16, data: u8);

    fn read_word(&mut self, addr: u16) -> u16 {
        let low = self.read_byte(addr);
        let high = self.read_byte(addr.wrapping_add(1));
        word!(low, high)
    }
}
