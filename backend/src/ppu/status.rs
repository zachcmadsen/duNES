use proc_bitfield::bitfield;

bitfield! {
    pub struct Status(pub u8) {
        pub sprite_overflow: bool @ 5,
        pub sprite_0_hit: bool @ 6,
        pub vblank: bool @ 7
    }
}
