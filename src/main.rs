use std::{env, fs};

use dunes::DuNes;

fn main() {
    let mut args = env::args();
    // Skip the executable path.
    args.next();
    let Some(filepath) = args.next() else {
        eprintln!("duNES: error: a ROM filepath was not provided");
        return;
    };

    let Ok(rom) = fs::read(&filepath) else {
        eprintln!("duNES: error: could not read from {}", &filepath);
        return;
    };

    let dunes = DuNes::new(&rom);
    eframe::run_native(
        "duNES",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(dunes)),
    )
    .unwrap();
}
