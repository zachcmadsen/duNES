use std::process::Command;

fn main() {
    Command::new("git")
        .args(["submodule", "update", "--init"])
        .status()
        .unwrap();

    cxx_build::bridge("src/apu.rs")
        .cpp(true)
        .include("Nes_Snd_Emu")
        .include("Nes_Snd_Emu/nes_apu")
        .include("include")
        // TODO: Add the cpp files programmatically?
        .file("Nes_Snd_Emu/nes_apu/apu_snapshot.cpp")
        .file("Nes_Snd_Emu/nes_apu/Blip_Buffer.cpp")
        .file("Nes_Snd_Emu/nes_apu/Multi_Buffer.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Apu.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Namco.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Oscs.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nes_Vrc6.cpp")
        .file("Nes_Snd_Emu/nes_apu/Nonlinear_Buffer.cpp")
        .file("src/shim.cpp")
        .flag("-std=c++14")
        .flag_if_supported("-Wno-deprecated")
        .flag_if_supported("-Wno-multichar")
        .flag_if_supported("-Wno-overflow")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-value")
        .compile("nes_snd_emu");

    println!("cargo:rerun-if-changed=Nes_Snd_Emu");
    println!("cargo:rerun-if-changed=include/shim.h");
    println!("cargo:rerun-if-changed=src/shim.cpp");
    println!("cargo:rerun-if-changed=src/apu.rs");
}
