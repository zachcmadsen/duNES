use proc_bitfield::bitfield;

bitfield! {
    pub struct Mask(pub u8) {
        pub greyscale: bool [read_only] @ 0,
        pub show_bg_left: bool [read_only] @ 1,
        pub show_sprites_left: bool [read_only] @ 2,
        pub show_bg: bool [read_only] @ 3,
        pub show_sprites: bool [read_only] @ 4,
        pub emphasize_red: bool [read_only] @ 5,
        pub emphasize_green: bool [read_only] @ 6,
        pub emphasize_blue: bool [read_only] @ 7,
    }
}
