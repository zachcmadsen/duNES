use std::fs;

use neobackend::Emu;

macro_rules! blargg_test {
    ($name:ident, $path:expr) => {
        #[test]
        fn $name() {
            run($path);
        }
    };
}

fn run(path: &str) {
    const STATUS_ADDR: u16 = 0x6000;
    const OUTPUT_ADDR: u16 = 0x6004;
    const RUNNING_STATUS: u8 = 0x80;

    let rom = fs::read(format!("roms/{path}")).unwrap();
    let mut emu = Emu::new(&rom);

    // Run the reset sequence.
    emu.step();

    let mut status = emu.peek(STATUS_ADDR);
    while status != RUNNING_STATUS {
        emu.step();
        status = emu.peek(STATUS_ADDR);
    }

    while status == RUNNING_STATUS {
        emu.step();
        status = emu.peek(STATUS_ADDR);
    }

    let mut output = Vec::new();
    let mut addr = OUTPUT_ADDR;
    let mut byte = emu.peek(addr);
    while byte != b'\0' {
        output.push(byte);
        addr += 1;
        byte = emu.peek(addr);
    }

    let output = String::from_utf8_lossy(&output);
    assert!(output.contains("Passed"), "{}", output);
}
