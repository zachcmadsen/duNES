mod address;
mod ctrl;
mod mask;
mod status;

use crate::{
    mapper::Mirroring,
    ppu::{address::Address, ctrl::Ctrl, mask::Mask, status::Status},
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

/// The width of the screen in pixels.
pub const WIDTH: u32 = 256;
/// The height of the screen in pixels.
pub const HEIGHT: u32 = 240;
/// The size of the framebuffer in bytes.
const FRAMEBUFFER_SIZE: usize = 4 * WIDTH as usize * HEIGHT as usize;

pub struct Ppu {
    // Data
    nametables: Box<[u8; 2 * NAMETABLE_SIZE]>,
    palettes: Box<[u8; PALETTES_SIZE]>,
    read_buffer: u8,

    primary_oam: Box<[u8; OAM_SIZE]>,

    /// The I/O bus.
    io_bus: u8,

    // TODO: Set/unset this in the right places.
    vblank: bool,

    /// The control register.
    ctrl: Ctrl,
    /// The mask register.
    mask: Mask,
    /// The status register.
    status: Status,

    /// VRAM address.
    v: Address,
    /// Temporary VRAM address.
    t: Address,
    fine_x_scroll: u8,
    /// A write toggle shared by the scroll and address registers.
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

    oam_addr: u8,

    palette: Box<[u32]>,
    buffer: Box<[u8; FRAMEBUFFER_SIZE]>,

    #[allow(clippy::type_complexity)]
    on_frame: Option<Box<dyn FnMut(&[u8])>>,
}

impl Ppu {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Ppu {
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
            primary_oam: vec![0; OAM_SIZE].try_into().unwrap(),

            io_bus: 0,
            vblank: false,

            ctrl: Ctrl(0),
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

            oam_addr: 0,

            buffer: vec![0; FRAMEBUFFER_SIZE].try_into().unwrap(),
            palette,

            on_frame: None,
        }
    }

    pub fn on_frame(&mut self, f: impl FnMut(&[u8]) + 'static) {
        self.on_frame = Some(Box::new(f));
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
                0x2800..=0x2fff => address - 0x0800,
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
            | ((emu.ppu.v.nt() as u16) << 10)
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
        if self.ctrl.bg_pt_addr() {
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
        self.mask.show_bg() || self.mask.show_sprites()
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
        emu.cpu.nmi = emu.ppu.ctrl.nmi();
    }

    if emu.ppu.rendering_enabled() && emu.ppu.on_screen() {
        let (pixel_value, palette) = emu.ppu.fetch_pixel();
        let palette_index = Ppu::read_data(
            emu,
            0x3f00 + (palette * 4) as u16 + pixel_value as u16,
        );

        let buffer_index =
            emu.ppu.scanline as usize * 256 * 4 + (emu.ppu.cycle as usize) * 4;
        emu.ppu.buffer[buffer_index..(buffer_index + 4)].copy_from_slice(
            &emu.ppu.palette[palette_index as usize].to_le_bytes(),
        );
    }

    emu.ppu.cycle = (emu.ppu.cycle + 1) % 341;
    if emu.ppu.cycle == 0 {
        emu.ppu.scanline = (emu.ppu.scanline + 1) % 262;

        if emu.ppu.on_prerender_scanline() {
            if let Some(on_frame) = &mut emu.ppu.on_frame {
                on_frame(emu.ppu.buffer.as_ref());
            }
        }
    }
}

pub fn read_register(emu: &mut Emu, address: u16) -> u8 {
    match address {
        0x2007 => {
            let data = if address < BASE_PALETTE_ADDRESS {
                emu.ppu.read_buffer
            } else {
                Ppu::read_data(emu, address)
            };

            // TODO: The data in the buffer should be different when
            // reading palette data.
            emu.ppu.read_buffer = Ppu::read_data(emu, address);

            emu.ppu.v.increment(emu.ppu.ctrl.vram_addr_incr());

            data
        }
        _ => unimplemented!(),
    }
}

pub fn write_register(emu: &mut Emu, address: u16, data: u8) {
    match address {
        0x2007 => {
            // TODO: Should emu.ppu.v.0 be mirrored down?
            Ppu::write_data(emu, emu.ppu.v.0, data);
            emu.ppu.v.increment(emu.ppu.ctrl.vram_addr_incr());
        }
        _ => unimplemented!(),
    }
}

pub fn read_bus(emu: &mut Emu, _: u16) -> u8 {
    emu.ppu.io_bus
}

pub fn write_bus(emu: &mut Emu, _: u16, data: u8) {
    emu.ppu.io_bus = data;
}

pub fn write_ctrl(emu: &mut Emu, _: u16, data: u8) {
    let prev_ctrl = emu.ppu.ctrl;
    emu.ppu.ctrl = Ctrl(data);
    emu.ppu.t.set_nt(emu.ppu.ctrl.base_nt_addr());

    // Flipping the NMI flag to true when the PPU is in vblank and the status
    // register's vblank flag is set raises an NMI.
    if emu.ppu.vblank
        && emu.ppu.status.vblank()
        && (!prev_ctrl.nmi() && emu.ppu.ctrl.nmi())
    {
        // TODO: Trigger an NMI.
    }

    emu.ppu.io_bus = data;
}

pub fn write_mask(emu: &mut Emu, _: u16, data: u8) {
    // TODO: Use the mask register to control rendering and colors.
    emu.ppu.mask = Mask(data);
    emu.ppu.io_bus = data;
}

pub fn read_status(emu: &mut Emu, _: u16) -> u8 {
    emu.ppu.io_bus = (emu.ppu.io_bus & 0x1F) | emu.ppu.status.0;
    // TODO: Reading the status register within two cycles of the start of
    // vblank has special behavior.
    emu.ppu.status.set_vblank(false);

    emu.ppu.w = false;

    emu.ppu.io_bus
}

pub fn write_oam_addr(emu: &mut Emu, _: u16, data: u8) {
    // TODO: Do I need to set oam_addr during rendering? Does it matter?
    emu.ppu.oam_addr = data;
    emu.ppu.io_bus = data;
}

pub fn read_oam_data(emu: &mut Emu, _: u16) -> u8 {
    emu.ppu.io_bus = emu.ppu.primary_oam[emu.ppu.oam_addr as usize];
    emu.ppu.io_bus
}

pub fn write_oam_data(emu: &mut Emu, _: u16, data: u8) {
    emu.ppu.primary_oam[emu.ppu.oam_addr as usize] = data;
    emu.ppu.oam_addr = emu.ppu.oam_addr.wrapping_add(1);
}

pub fn write_scroll(emu: &mut Emu, _: u16, data: u8) {
    const COARSE_SCROLL_MASK: u8 = 0xF8;
    const FINE_SCROLL_MASK: u8 = 0x07;
    if !emu.ppu.w {
        emu.ppu.t.set_coarse_x_scroll((data & COARSE_SCROLL_MASK) >> 3);
        emu.ppu.fine_x_scroll = data & FINE_SCROLL_MASK;
        emu.ppu.w = true;
    } else {
        emu.ppu.t.set_coarse_y_scroll((data & COARSE_SCROLL_MASK) >> 3);
        emu.ppu.t.set_fine_y_scroll(data & FINE_SCROLL_MASK);
        emu.ppu.w = false;
    }

    emu.ppu.io_bus = data;
}

pub fn write_addr(emu: &mut Emu, _: u16, data: u8) {
    if !emu.ppu.w {
        emu.ppu.t.set_high(data & 0x3F);
        emu.ppu.w = true;
    } else {
        emu.ppu.t.set_low(data);
        emu.ppu.v = Address(emu.ppu.t.0);
        emu.ppu.w = false;
    }

    emu.ppu.io_bus = data;
}
