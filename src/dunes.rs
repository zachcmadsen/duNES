use std::collections::BTreeMap;

use eframe::{
    egui::{CentralPanel, CollapsingHeader, Context, Key, RichText, Ui},
    App,
};

use crate::{bus::Bus, cartridge::NromCartridge, cpu::Cpu};

/// The number of preceding instructions to show in the assembly panel.
const INSTRUCTIONS_BEFORE: usize = 10;
/// The total number of instructions to show in the assembly panel.
const TOTAL_INSTRUCTIONS: usize = 21;

pub struct DuNes {
    cpu: Cpu,
    assembly: BTreeMap<u16, String>,
}

impl App for DuNes {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(Key::Space)) {
                self.cpu.step();
            }

            self.draw_registers(ui);
            self.draw_assembly(ui);
        });
    }
}

impl DuNes {
    pub fn new(rom: &[u8]) -> DuNes {
        let cartridge = NromCartridge::new(rom);
        let bus = Bus::new(cartridge);
        let cpu = Cpu::new(bus);
        // TODO: React to memory changes to have accurate assembly (or
        // disassemble on-the-fly?). Disassembling the entire CPU memory at
        // startup is good enough for now, but it will become more innacurate
        // as other mappers are added.
        let assembly = cpu.disassemble();

        DuNes { cpu, assembly }
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
                )
            });
    }

    fn draw_assembly(&self, ui: &mut Ui) {
        CollapsingHeader::new("Assembly")
            .default_open(true)
            .show(ui, |ui| {
                // TODO: Reuse this Vec between updates to save an allocation.
                let mut instructions_before =
                    Vec::with_capacity(INSTRUCTIONS_BEFORE);
                for thing in self
                    .assembly
                    .range(..self.cpu.pc)
                    .rev()
                    .take(INSTRUCTIONS_BEFORE)
                {
                    instructions_before.push(thing);
                }
                let mut instructions_after = self
                    .assembly
                    .range((self.cpu.pc)..)
                    // There may not be INSTRUCTIONS_BEFORE instructions before
                    // the current instruction, i.e., PC < INSTRUCTIONS_BEFORE.
                    // We subtract the actual amount so that we always show
                    // TOTAL_INSTRUCTIONS instructions.
                    .take(TOTAL_INSTRUCTIONS - instructions_before.len());

                for (address, instruction) in instructions_before.iter().rev()
                {
                    ui.label(
                        RichText::new(format!("{address:04X}: {instruction}"))
                            .monospace(),
                    );
                }
                if let Some((address, instruction)) = instructions_after.next()
                {
                    ui.label(
                        RichText::new(format!("{address:04X}: {instruction}"))
                            .monospace()
                            .strong(),
                    );
                }
                for (address, instruction) in instructions_after {
                    ui.label(
                        RichText::new(format!("{address:04X}: {instruction}"))
                            .monospace(),
                    );
                }
            });
    }
}
