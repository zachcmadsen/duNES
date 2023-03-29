use eframe::{
    egui::{
        CentralPanel, CollapsingHeader, Color32, ColorImage, Context, Key,
        RichText, TextureHandle, TextureOptions, Ui,
    },
    App, CreationContext,
};

use crate::{Cpu, DuNesBus, NromCartridge};

/// The width of the pattern table image in pixels.
const PATTERN_TABLE_WIDTH: usize = 128;
/// The height of the pattern table image in pixels.
const PATTERN_TABLE_HEIGHT: usize = 256;
/// The size of a tile in bytes.
const TILE_SIZE: usize = 16;
/// The size of one plane of a tile in bytes.
const TILE_PLANE_SIZE: usize = 8;
/// The width of a tile in pixels;
const TILE_WIDTH: usize = 8;
/// The height of a tile in pixels;
const TILE_HEIGHT: usize = 8;
/// The number of tiles per row in the pattern table image.
const TILES_PER_ROW: usize = 16;
/// The number of pixels per tile row in the pattern table image.
const PIXELS_PER_TILE_ROW: usize = 1024;
/// The total number of tiles in the pattern table image.
const TOTAL_TILES: usize = 512;

/// The width of the screen image in pixels.
const SCREEN_WIDTH: usize = 256;
/// The height of the screen image in pixels.
const SCREEN_HEIGHT: usize = 240;

// TODO: Do more research into palettes. Use a .pal file?
const PALLETE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];

pub struct DuNes {
    cpu: Cpu<DuNesBus>,

    pattern_table_texture: TextureHandle,
    screen_texture: TextureHandle,

    paused: bool,
    first_frame: bool,
}

impl App for DuNes {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let container = CentralPanel::default().show(ctx, |ui| {
            self.cpu.bus.controller = 0;

            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::J)) {
                0x80
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::K)) {
                0x40
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::U)) {
                0x20
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::Y)) {
                0x10
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::W)) {
                0x08
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::S)) {
                0x04
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::A)) {
                0x02
            } else {
                0
            };
            self.cpu.bus.controller |= if ui.input(|i| i.key_down(Key::D)) {
                0x01
            } else {
                0
            };

            if ui.input(|i| i.key_pressed(Key::P)) {
                self.paused = !self.paused;
            }

            if !self.paused {
                while !self.cpu.bus.ppu.is_frame_ready {
                    self.cpu.step();
                }
                self.cpu.bus.ppu.is_frame_ready = false
            } else if ui.input(|i| i.key_pressed(Key::Space)) {
                self.cpu.step();
            }

            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    self.draw_registers(ui);
                    self.draw_disassembly(ui);
                });
                self.draw_screen(ui);
                self.draw_pattern_table(ui);
            });
        });

        if self.first_frame {
            self.first_frame = false;
            frame.set_window_size(container.response.rect.size());
        }

        ctx.request_repaint();
    }
}

impl DuNes {
    pub fn new(rom: &[u8], cc: &CreationContext) -> DuNes {
        let cartridge = NromCartridge::new(rom);
        let bus = DuNesBus::new(cartridge);
        let cpu = Cpu::new(bus);

        let pattern_table_texture = cc.egui_ctx.load_texture(
            "pattern-table",
            ColorImage::new(
                [PATTERN_TABLE_WIDTH, PATTERN_TABLE_HEIGHT],
                Color32::BLACK,
            ),
            TextureOptions::NEAREST,
        );
        let screen_texture = cc.egui_ctx.load_texture(
            "screen",
            ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        DuNes {
            cpu,
            pattern_table_texture,
            screen_texture,
            paused: true,
            first_frame: true,
        }
    }

    fn draw_registers(&self, ui: &mut Ui) {
        CollapsingHeader::new("Registers")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "A: 0x{a:02X} ({a})",
                        a = self.cpu.a,
                    ))
                    .monospace(),
                );
                ui.label(
                    RichText::new(format!(
                        "X: 0x{x:02X} ({x})",
                        x = self.cpu.x
                    ))
                    .monospace(),
                );
                ui.label(
                    RichText::new(format!(
                        "Y: 0x{y:02X} ({y})",
                        y = self.cpu.y
                    ))
                    .monospace(),
                );
                ui.label(
                    RichText::new(format!("PC: 0x{:04X}", self.cpu.pc))
                        .monospace(),
                );
                ui.label(
                    RichText::new(format!("SP: 0x{:02X}", self.cpu.s))
                        .monospace(),
                );
                // TODO: Show which flags are on/off.
                ui.label(
                    RichText::new(format!(
                        "Status: 0x{:02X}",
                        u8::from(self.cpu.p),
                    ))
                    .monospace(),
                );
            });
    }

    fn draw_disassembly(&self, ui: &mut Ui) {
        CollapsingHeader::new("Disassembly")
            .default_open(true)
            .show(ui, |ui| {
                let disasm = self.cpu.disassemble();
                let mut disasm_iter = disasm.iter();

                // Use strong text for the current instruction.
                if let Some(instruction) = disasm_iter.next() {
                    ui.label(RichText::new(instruction).monospace().strong());
                }
                for instruction in disasm_iter {
                    ui.label(RichText::new(instruction).monospace());
                }
            });
    }

    fn draw_pattern_table(&mut self, ui: &mut Ui) {
        let image = self.generate_pattern_table_image();
        self.pattern_table_texture
            .set(image, TextureOptions::NEAREST);

        CollapsingHeader::new("Pattern table")
            .default_open(true)
            .show(ui, |ui| {
                ui.image(
                    self.pattern_table_texture.id(),
                    self.pattern_table_texture.size_vec2() * 2.0,
                );
            });
    }

    fn draw_screen(&mut self, ui: &mut Ui) {
        let image = self.generate_screen_image();
        self.screen_texture.set(image, TextureOptions::NEAREST);

        CollapsingHeader::new("Screen")
            .default_open(true)
            .show(ui, |ui| {
                ui.image(
                    self.screen_texture.id(),
                    self.screen_texture.size_vec2() * 2.0,
                );
            });
    }

    fn generate_pattern_table_image(&self) -> ColorImage {
        let mut pixels =
            vec![Color32::BLACK; PATTERN_TABLE_WIDTH * PATTERN_TABLE_HEIGHT];

        for tile in 0..TOTAL_TILES {
            let tile_row = tile / TILES_PER_ROW;
            let tile_col = tile % TILES_PER_ROW;
            let preceding_cols_pixel_width = TILE_WIDTH * tile_col;
            let base_pixel_index =
                tile_row * PIXELS_PER_TILE_ROW + preceding_cols_pixel_width;

            for row in 0..TILE_WIDTH {
                let mut low = self
                    .cpu
                    .bus
                    .ppu
                    .read_data((tile * TILE_SIZE + row) as u16);
                let mut high = self.cpu.bus.ppu.read_data(
                    (tile * TILE_SIZE + TILE_PLANE_SIZE + row) as u16,
                );

                // The LSB of a tile plane refers to the rightmost pixel of the
                // tile. Since we're drawing from the top-left of tile, we need
                // to invert the x-axis.
                for col in (0..TILE_HEIGHT).rev() {
                    let pixel_value = (1 & high) << 1 | (1 & low);
                    low >>= 1;
                    high >>= 1;

                    let offset = row * PATTERN_TABLE_WIDTH + col;
                    // TODO: Support choosing different palettes instead of defaulting to
                    // the first one.
                    let color = self.get_color(pixel_value, 0);
                    pixels[base_pixel_index + offset] = color
                }
            }
        }

        ColorImage {
            size: [PATTERN_TABLE_WIDTH, PATTERN_TABLE_HEIGHT],
            pixels,
        }
    }

    fn generate_screen_image(&self) -> ColorImage {
        let pixels = self
            .cpu
            .bus
            .ppu
            .frame
            .iter()
            .map(|(pixel_value, palette)| {
                self.get_color(*pixel_value, *palette)
            })
            .collect();

        ColorImage {
            size: [SCREEN_WIDTH, SCREEN_HEIGHT],
            pixels,
        }
    }

    fn get_color(&self, pixel_value: u8, palette: u8) -> Color32 {
        let palette_index = self
            .cpu
            .bus
            .ppu
            .read_data(0x3f00 + (palette * 4) as u16 + pixel_value as u16);
        let (r, g, b) = PALLETE[palette_index as usize];
        Color32::from_rgb(r, g, b)
    }
}
