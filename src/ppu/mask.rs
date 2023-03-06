use proc_bitfield::bitfield;

bitfield! {
    pub struct Mask(pub u8) {
        pub greyscale: bool @ 0,
        pub show_background_left: bool @ 1,
        pub show_sprites_left: bool @ 2,
        pub show_background: bool @ 3,
        pub show_sprites: bool @ 4,
        pub emphasize_red: bool @ 5,
        pub emphasize_green: bool @ 6,
        pub emphasize_blue: bool @ 7,
    }
}
