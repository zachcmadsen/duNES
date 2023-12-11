use std::{env, fs};
use tracing::Level;

use frontend::run;
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut args = env::args();
    // Skip the executable path.
    args.next();

    let Some(file_path) = args.next() else {
        eprintln!("duNES: error: expected a ROM file");
        return;
    };
    let rom = fs::read(file_path).unwrap();

    run(rom);
}
