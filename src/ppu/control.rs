use proc_bitfield::bitfield;

bitfield! {
    pub struct Control(pub u8) {
        pub nametable: u8 @ 0..2,
        pub vram_address_increment: bool @ 2,
        pub sprite_pattern_table: bool @ 3,
        pub background_pattern_table: bool @ 4,
        pub sprite_size: bool @ 5,
        pub nmi: bool @ 7,
    }
}

impl Control {
    pub fn background_pattern_table_address(&self) -> u16 {
        if self.background_pattern_table() {
            0x1000
        } else {
            0x0000
        }
    }
}
