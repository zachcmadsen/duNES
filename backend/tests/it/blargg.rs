use std::fs;

use backend::Emu;

fn run(dir: &str) {
    const STATUS_ADDR: u16 = 0x6000;
    const OUTPUT_ADDR: u16 = 0x6004;
    const RUNNING_STATUS: u8 = 0x80;

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let rom = fs::read(entry.path()).unwrap();

        // TODO: Reuse the same emulator instance across the ROMs to test
        // resets and loading new ROMs.
        let mut emu = Emu::new(&rom);

        let mut status = emu.read(STATUS_ADDR);
        while status != RUNNING_STATUS {
            emu.step();
            status = emu.read(STATUS_ADDR);
        }

        while status == RUNNING_STATUS {
            emu.step();
            status = emu.read(STATUS_ADDR);
        }

        let mut output = Vec::new();
        let mut addr = OUTPUT_ADDR;
        let mut byte = emu.read(addr);
        while byte != b'\0' {
            output.push(byte);
            addr += 1;
            byte = emu.read(addr);
        }

        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("Passed"), "{}", output);
    }
}

#[test]
fn instr() {
    run("../roms/instr_test-v5/")
}

#[test]
fn apu() {
    // TODO: Add the rest of the apu_test ROMs once APU IRQs are handled.
    run("../roms/apu_test/")
}
