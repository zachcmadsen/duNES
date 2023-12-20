use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Ctrl(pub u8) {
        pub base_nt_addr: u8 [read_only] @ 0..2,
        pub vram_addr_incr: bool [read_only] @ 2,
        pub sprite_pt_addr: bool [read_only] @ 3,
        pub bg_pt_addr: bool [read_only] @ 4,
        pub sprite_size: bool [read_only] @ 5,
        pub nmi: bool [read_only] @ 7,
    }
}
