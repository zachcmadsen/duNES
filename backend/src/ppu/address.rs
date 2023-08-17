use proc_bitfield::bitfield;

/// The number of tile columns in a nametable.
const TILE_COLUMNS: u8 = 32;
/// The number of tile rows in a nametable.
const TILE_ROWS: u8 = 30;
/// The height of a tile in pixels.
const TILE_HEIGHT: u8 = 8;

bitfield! {
    pub struct Address(pub u16) {
        pub coarse_x_scroll: u8 @ 0..5,
        pub coarse_y_scroll: u8 @ 5..10,
        pub nametable: u8 @ 10..12,
        pub fine_y_scroll: u8 @ 12..15,

        nametable_x: bool @ 10,
        nametable_y: bool @ 11,

        pub low: u8 [write_only] @ 0..8,
        pub high: u8 [write_only] @ 8..14,
    }
}

impl Address {
    pub fn increment_coarse_x_scroll(&mut self) {
        if self.coarse_x_scroll() == (TILE_COLUMNS - 1) {
            self.set_coarse_x_scroll(0);
            self.set_nametable_x(!self.nametable_x());
        } else {
            self.set_coarse_x_scroll(self.coarse_x_scroll() + 1);
        }
    }

    pub fn increment_y_scroll(&mut self) {
        if self.fine_y_scroll() == (TILE_HEIGHT - 1) {
            self.set_fine_y_scroll(0);

            if self.coarse_y_scroll() == (TILE_ROWS - 1) {
                self.set_coarse_y_scroll(0);
                self.set_nametable_y(!self.nametable_y());
            } else if self.coarse_y_scroll() == 31 {
                self.set_coarse_y_scroll(0);
            } else {
                self.set_coarse_y_scroll(self.coarse_y_scroll() + 1);
            }
        } else {
            self.set_fine_y_scroll(self.fine_y_scroll() + 1);
        }
    }

    pub fn load_x_scroll(&mut self, other: &Address) {
        self.set_coarse_x_scroll(other.coarse_x_scroll());
        self.set_nametable_x(other.nametable_x());
    }

    pub fn load_y_scroll(&mut self, other: &Address) {
        self.set_coarse_y_scroll(other.coarse_y_scroll());
        self.set_nametable_y(other.nametable_y());
        self.set_fine_y_scroll(other.fine_y_scroll());
    }

    pub fn increment(&mut self, increment_mode: bool) {
        // TODO: Can this overflow or is it mirrored down past 0x3fff?
        self.0 += if increment_mode { 32 } else { 1 };
    }
}
