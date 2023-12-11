use pixels_frontend::run;
use std::{env, fs};

fn main() {
    let mut args = env::args();
    // Skip the executable path.
    args.next();

    let Some(file_path) = args.next() else {
        eprintln!("duNES: error: expected a ROM file");
        return;
    };
    let rom = fs::read(&file_path).unwrap();

    run(rom);
}
