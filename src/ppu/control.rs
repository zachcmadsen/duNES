use proc_bitfield::bitfield;

bitfield! {
    pub struct Control(pub u8) {
        pub nametable: u8 [read_only] @ 0..2,
        pub vram_address_increment: bool [read_only] @ 2,
        pub sprite_pattern_table: bool [read_only] @ 3,
        pub background_pattern_table: bool [read_only] @ 4,
        pub sprite_size: bool [read_only] @ 5,
        pub nmi: bool [read_only] @ 7,
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
