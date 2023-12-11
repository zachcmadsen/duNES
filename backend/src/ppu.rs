mod address;
mod control;
mod mask;
mod status;

use common::Writer;

use crate::{
    mapper::Mirroring,
    ppu::{address::Address, control::Control, mask::Mask, status::Status},
    Emu,
};

/// The size of a nametable in bytes.
const NAMETABLE_SIZE: usize = 1024;
/// The size of the palette memory in bytes.
const PALETTES_SIZE: usize = 32;
/// The base address of the nametable memory.
const BASE_NAMETABLE_ADDRESS: u16 = 0x2000;
/// The base address of the attribute tables.
const BASE_ATTRIBUTE_ADDRESS: u16 = 0x23c0;
/// The base address of the palette memory.
const BASE_PALETTE_ADDRESS: u16 = 0x3f00;
/// The size of a tile in bytes.
const TILE_SIZE: usize = 16;
/// The size of one plane of a tile in bytes.
const TILE_PLANE_SIZE: usize = 8;
/// The size of OAM in bytes.
const OAM_SIZE: usize = 256;

pub struct Ppu {
    // Data
    nametables: Box<[u8; 2 * NAMETABLE_SIZE]>,
    palettes: Box<[u8; PALETTES_SIZE]>,
    read_buffer: u8,
    oam: Box<[u8; OAM_SIZE]>,

    // Registers
    control: Control,
    mask: Mask,
    status: Status,

    // Scrolling
    v: Address,
    t: Address,
    fine_x_scroll: u8,
    w: bool,

    // State
    cycle: u16,
    scanline: u16,
    pub nmi: bool,

    // Background latches
    nametable_latch: u8,
    attribute_bits: u8,
    attribute_latch: u8,
    tile_latch_low: u8,
    tile_latch_high: u8,

    // Background shifters
    tile_shifter_low: u16,
    tile_shifter_high: u16,
    attribute_shifter_low: u8,
    attribute_shifter_high: u8,

    oam_address: u8,

    palette: Box<[u32]>,
    writer: Writer,
}

impl Ppu {
    pub fn new(writer: Writer) -> Ppu {
        let palette = include_bytes!("../ntscpalette.pal")
            .chunks_exact(3)
            .map(|chunk| {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                u32::from_le_bytes([r, g, b, 0xFF])
            })
            .collect();

        Ppu {
            nametables: vec![0; 2 * NAMETABLE_SIZE].try_into().unwrap(),
            palettes: vec![0; PALETTES_SIZE].try_into().unwrap(),
            read_buffer: 0,
            oam: vec![0; OAM_SIZE].try_into().unwrap(),

            control: Control(0),
            mask: Mask(0),
            status: Status(0),

            v: Address(0),
            t: Address(0),
            fine_x_scroll: 0,
            w: false,

            cycle: 0,
            scanline: 0,
            nmi: false,

            nametable_latch: 0,
            attribute_bits: 0,
            attribute_latch: 0,
            tile_latch_low: 0,
            tile_latch_high: 0,

            tile_shifter_low: 0,
            tile_shifter_high: 0,
            attribute_shifter_low: 0,
            attribute_shifter_high: 0,

            oam_address: 0,

            writer,
            palette,
        }
    }

    fn read_data(emu: &Emu, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => emu.mapper.read_chr(address),
            0x2000..=0x3eff => {
                let mirrored_address =
                    Ppu::mirror_nametable_address(emu, address);
                emu.ppu.nametables
                    [(mirrored_address - BASE_NAMETABLE_ADDRESS) as usize]
            }
            0x3f00..=0x3fff => {
                let mirrored_address = emu.ppu.mirror_palette_address(address);
                emu.ppu.palettes
                    [(mirrored_address - BASE_PALETTE_ADDRESS) as usize]
            }
            _ => unreachable!(),
        }
    }

    fn write_data(emu: &mut Emu, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => unimplemented!(),
            0x2000..=0x3eff => {
                let mirrored_address =
                    Ppu::mirror_nametable_address(emu, address);
                emu.ppu.nametables
                    [(mirrored_address - BASE_NAMETABLE_ADDRESS) as usize] =
                    data;
            }
            0x3f00..=0x3fff => {
                let mirrored_address = emu.ppu.mirror_palette_address(address);
                emu.ppu.palettes
                    [(mirrored_address - BASE_PALETTE_ADDRESS) as usize] =
                    data;
            }
            _ => unreachable!(),
        }
    }

    fn mirror_nametable_address(emu: &Emu, address: u16) -> u16 {
        match emu.mapper.mirroring() {
            // The second and fourth nametables map to the first and second
            // nametables, respectively.
            Mirroring::Horizontal => match address {
                0x2400..=0x27ff => address - 0x0400,
                // We have to map the third and fourth logical nametables to
                // the second physical nametable.
                0x2800..=0x2bff => address - 0x0400,
                0x2c00..=0x2fff => address - 0x0800,
                _ => address,
            },
            // The third and fourth nametables map to the first and second
            // nametables, respectively.
            Mirroring::Vertical => match address {
                0x2800..=0x2bff | 0x2c00..=0x2fff => address - 0x0800,
                _ => address,
            },
        }
    }

    fn mirror_palette_address(&self, address: u16) -> u16 {
        // 0x3f20 - 0x3fff is a mirror of 0x3f00 - 0x3f1f.
        let address = address & 0x3f1f;
        match address {
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => address - 0x10,
            _ => address,
        }
    }

    fn fetch_nametable_byte(emu: &Emu) -> u8 {
        // The bottom 12 bits of v are the address of the next tile in
        // nametable memory.
        Ppu::read_data(emu, BASE_NAMETABLE_ADDRESS | (emu.ppu.v.0 & 0x0fff))
    }

    fn fetch_attribute_bits(emu: &mut Emu) -> u8 {
        // Divide the coarse offsets by four since each attribute byte applies
        // to a 4x4 group of tiles.
        let attribute_address = BASE_ATTRIBUTE_ADDRESS
            | ((emu.ppu.v.nametable() as u16) << 10)
            | (((emu.ppu.v.coarse_y_scroll() as u16) / 4) << 3)
            | ((emu.ppu.v.coarse_x_scroll() as u16) / 4);
        let mut attribute_byte = Ppu::read_data(emu, attribute_address);

        // The attribute byte is divided into four 2-bit areas that specify the
        // palettes for four 2x2 tile groups.
        //
        // The bottom row of a 2x2 group is covered by the four leftmost
        // bits.
        attribute_byte >>=
            ((emu.ppu.v.coarse_y_scroll() & 0x02) != 0) as u8 * 4;
        // The right column of a 2x2 group is covered by the two leftmost
        // bits.
        attribute_byte >>=
            ((emu.ppu.v.coarse_x_scroll() & 0x02) != 0) as u8 * 2;

        // Mask off the rest of the bits.
        attribute_byte & 0x03
    }

    fn background_pattern_table_address(&self) -> u16 {
        if self.control.background_pattern_table() {
            0x1000
        } else {
            0x0000
        }
    }

    fn fetch_tile_data(emu: &mut Emu) {
        match (emu.ppu.cycle) % 8 {
            1 => {
                emu.ppu.reload_background_shifters();
                emu.ppu.nametable_latch = Ppu::fetch_nametable_byte(emu);
            }
            3 => {
                emu.ppu.attribute_bits = Ppu::fetch_attribute_bits(emu);
            }
            5 => {
                emu.ppu.tile_latch_low = Ppu::read_data(
                    emu,
                    emu.ppu.background_pattern_table_address()
                        + (emu.ppu.nametable_latch as u16 * TILE_SIZE as u16)
                        + emu.ppu.v.fine_y_scroll() as u16,
                );
            }
            7 => {
                emu.ppu.tile_latch_high = Ppu::read_data(
                    emu,
                    emu.ppu.background_pattern_table_address()
                        + ((emu.ppu.nametable_latch as u16) << 4)
                        + emu.ppu.v.fine_y_scroll() as u16
                        + TILE_PLANE_SIZE as u16,
                )
            }
            0 => {
                if emu.ppu.rendering_enabled() {
                    emu.ppu.v.increment_coarse_x_scroll();
                }
            }
            _ => (),
        }
    }

    fn on_visible_scanline(&self) -> bool {
        self.scanline < 240
    }

    fn on_prerender_scanline(&self) -> bool {
        self.scanline == 261
    }

    fn on_screen(&self) -> bool {
        self.scanline < 240 && self.cycle < 256
    }

    fn start_of_vblank(&self) -> bool {
        self.scanline == 241 && self.cycle == 1
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.show_background() || self.mask.show_sprites()
    }

    fn fetch_pixel(&self) -> (u8, u8) {
        let pixel_offset = 0x8000 >> self.fine_x_scroll;
        let pixel_low = ((self.tile_shifter_low & pixel_offset) != 0) as u8;
        let pixel_high = ((self.tile_shifter_high & pixel_offset) != 0) as u8;
        let pixel = (pixel_high << 1) | pixel_low;

        let palette_offset = 0x80 >> self.fine_x_scroll;
        let palette_low =
            ((self.attribute_shifter_low & palette_offset) != 0) as u8;
        let palette_high =
            ((self.attribute_shifter_high & palette_offset) != 0) as u8;
        let palette = (palette_high << 1) | palette_low;

        (pixel, palette)
    }

    fn reload_background_shifters(&mut self) {
        self.tile_shifter_low =
            (self.tile_shifter_low & 0xff00) | self.tile_latch_low as u16;
        self.tile_shifter_high =
            (self.tile_shifter_high & 0xff00) | self.tile_latch_high as u16;
        self.attribute_latch = self.attribute_bits;
    }

    fn shift_background_shifters(&mut self) {
        self.tile_shifter_low <<= 1;
        self.tile_shifter_high <<= 1;
        self.attribute_shifter_low =
            (self.attribute_shifter_low << 1) | (self.attribute_latch & 1);
        self.attribute_shifter_high = (self.attribute_shifter_high << 1)
            | ((self.attribute_latch & 0x2) >> 1);
    }
}

pub fn tick(emu: &mut Emu) {
    if emu.ppu.on_prerender_scanline() {
        if emu.ppu.cycle == 1 {
            emu.ppu.status.set_sprite_overflow(false);
            emu.ppu.status.set_sprite_0_hit(false);
            emu.ppu.status.set_vblank(false);
            emu.cpu.nmi = false;
        }

        // The vertical scroll bits are reloaded during pixels 280 to 304,
        // if rendering is enabled.
        if emu.ppu.cycle >= 280
            && emu.ppu.cycle <= 304
            && emu.ppu.rendering_enabled()
        {
            emu.ppu.v.load_y_scroll(&emu.ppu.t);
        }
    }

    if emu.ppu.on_visible_scanline() || emu.ppu.on_prerender_scanline() {
        if (emu.ppu.cycle >= 1 && emu.ppu.cycle <= 256)
            || (emu.ppu.cycle >= 321 && emu.ppu.cycle <= 337)
        {
            emu.ppu.shift_background_shifters();
            Ppu::fetch_tile_data(emu);
        }

        if emu.ppu.cycle == 256 && emu.ppu.rendering_enabled() {
            emu.ppu.v.increment_y_scroll();
        }

        if emu.ppu.cycle == 257 && emu.ppu.rendering_enabled() {
            emu.ppu.v.load_x_scroll(&emu.ppu.t);
        }

        if emu.ppu.cycle == 339 {
            emu.ppu.nametable_latch = Ppu::fetch_nametable_byte(emu);
        }
    }

    if emu.ppu.start_of_vblank() {
        emu.ppu.status.set_vblank(true);
        emu.cpu.nmi = emu.ppu.control.nmi();
    }

    if emu.ppu.rendering_enabled() && emu.ppu.on_screen() {
        let (pixel_value, palette) = emu.ppu.fetch_pixel();
        let palette_index = Ppu::read_data(
            emu,
            0x3f00 + (palette * 4) as u16 + pixel_value as u16,
        );

        let buffer_index =
            emu.ppu.scanline as usize * 256 * 4 + (emu.ppu.cycle as usize) * 4;
        emu.ppu.writer.get_mut()[buffer_index..(buffer_index + 4)]
            .copy_from_slice(
                &emu.ppu.palette[palette_index as usize].to_le_bytes(),
            );
    }

    emu.ppu.cycle = (emu.ppu.cycle + 1) % 341;
    if emu.ppu.cycle == 0 {
        emu.ppu.scanline = (emu.ppu.scanline + 1) % 262;

        if emu.ppu.on_prerender_scanline() {
            emu.ppu.writer.swap();
        }
    }
}

pub fn read_register(emu: &mut Emu, address: u16) -> u8 {
    match address {
        0x2002 => {
            // TODO: Using the read buffer here isn't accurate. It should
            // be the data bus, which isn't emulated yet.
            let data = emu.ppu.status.0 | (emu.ppu.read_buffer & 0x1f);
            // TODO: Should emu.ppu.nmi be set to false? Also, reading the
            // status register within two cycles of vblank causes an NMI
            // not to occur?
            emu.ppu.status.set_vblank(false);
            emu.ppu.w = false;

            data
        }
        0x2004 => emu.ppu.oam[emu.ppu.oam_address as usize],
        0x2007 => {
            let data = if address < BASE_PALETTE_ADDRESS {
                emu.ppu.read_buffer
            } else {
                Ppu::read_data(emu, address)
            };

            // TODO: The data in the buffer should be different when
            // reading palette data.
            emu.ppu.read_buffer = Ppu::read_data(emu, address);

            emu.ppu.v.increment(emu.ppu.control.increment_mode());

            data
        }
        _ => unimplemented!(),
    }
}

pub fn write_register(emu: &mut Emu, address: u16, data: u8) {
    match address {
        0x2000 => {
            emu.ppu.control = Control(data);
            emu.ppu.t.set_nametable(emu.ppu.control.nametable());
        }
        0x2001 => emu.ppu.mask = Mask(data),
        0x2003 => emu.ppu.oam_address = data,
        0x2004 => {
            emu.ppu.oam[emu.ppu.oam_address as usize] = data;
            emu.ppu.oam_address = emu.ppu.oam_address.wrapping_add(1);
        }
        0x2005 => {
            if !emu.ppu.w {
                emu.ppu.fine_x_scroll = data & 0x7;
                emu.ppu.t.set_coarse_x_scroll(data >> 3);
            } else {
                emu.ppu.t.set_fine_y_scroll(data & 0x7);
                emu.ppu.t.set_coarse_y_scroll(data >> 3);
            }

            emu.ppu.w = !emu.ppu.w;
        }
        0x2006 => {
            if !emu.ppu.w {
                emu.ppu.t.set_high(data & 0x3f);
            } else {
                emu.ppu.t.set_low(data);
                emu.ppu.v = Address(emu.ppu.t.0);
            }

            emu.ppu.w = !emu.ppu.w;
        }
        0x2007 => {
            // TODO: Should emu.ppu.v.0 be mirrored down?
            Ppu::write_data(emu, emu.ppu.v.0, data);
            emu.ppu.v.increment(emu.ppu.control.increment_mode());
        }
        _ => (),
    }
}
