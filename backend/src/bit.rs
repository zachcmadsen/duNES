pub trait BitPos {
    fn lsb(self) -> bool;
    fn msb(self) -> bool;
}

impl BitPos for u8 {
    fn lsb(self) -> bool {
        const LSB_MASK: u8 = 0x01;
        (self & LSB_MASK) == LSB_MASK
    }

    fn msb(self) -> bool {
        const MSB_MASK: u8 = 0x80;
        (self & MSB_MASK) == MSB_MASK
    }
}
