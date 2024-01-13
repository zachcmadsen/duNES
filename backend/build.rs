use std::process::Command;

fn main() {
    Command::new("git")
        .args(["submodule", "update", "--init"])
        .status()
        .unwrap();

    cxx_build::bridge("src/apu.rs")
        .cpp(true)
        .std("c++14")
        .include("include")
        .include("Nes_Snd_Emu/nes_apu")
        .file("src/apu/shim.cpp")
        .file("Nes_Snd_Emu/nes_apu/Blip_Buffer.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Apu.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Oscs.cpp")
        .define("NDEBUG", None)
        .compile("nes_snd_emu");

    println!("cargo:rerun-if-changed=Nes_Snd_Emu/nes_apu");
    println!("cargo:rerun-if-changed=include/shim.h");
    println!("cargo:rerun-if-changed=src/shim.cpp");
    println!("cargo:rerun-if-changed=src/apu.rs");
}
