use proc_bitfield::bitfield;

bitfield! {
    pub struct Address(pub u16) {
        pub coarse_x_scroll: u8 @ 0..5,
        pub coarse_y_scroll: u8 @ 5..10,
        pub nametable: u8 @ 10..12,
        pub fine_y_scroll: u8 @ 12..15,

        pub nametable_x: bool @ 10,
        pub nametable_y: bool @ 11,

        pub low: u8 [write_only] @ 0..8,
        pub high: u8 [write_only] @ 8..14,
    }
}

impl Address {
    pub fn increment_coarse_x_scroll(&mut self) {
        // 31 means we've reached the last tile in the nametable (the last col)
        // so we switch to the other horizontal nametable
        if self.coarse_x_scroll() == 31 {
            self.set_coarse_x_scroll(0);
            self.set_nametable_x(!self.nametable_x());
        } else {
            self.set_coarse_x_scroll(self.coarse_x_scroll() + 1);
        }
    }

    pub fn increment_y_scroll(&mut self) {
        if self.fine_y_scroll() < 7 {
            // Haven't reached the last row of the tile yet so increment
            self.set_fine_y_scroll(self.fine_y_scroll() + 1);
        } else {
            // it wraps back to 0
            self.set_fine_y_scroll(0);

            if self.coarse_y_scroll() == 29 {
                // We've reached the last row of tiles so wrap to the next
                // nametable
                self.set_coarse_y_scroll(0);
                self.set_nametable_y(!self.nametable_y());
            } else if self.coarse_y_scroll() == 31 {
                // If coarse y is out of bounds then the PPU will read the
                // attribute data as tile data. When it reaches 31 (the last
                // row of the nametable), it will wrap 0 and not switch
                // nametables.
                self.set_coarse_y_scroll(0);
            } else {
                self.set_coarse_y_scroll(self.coarse_y_scroll() + 1);
            }
        }
    }
}
