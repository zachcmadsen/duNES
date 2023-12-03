fn main() {
    cxx_build::bridge("src/lib.rs")
        .cpp(true)
        .include("Nes_Snd_Emu-0.1.7")
        .include("Nes_Snd_Emu-0.1.7/nes_apu")
        .include("include")
        // TODO: Add the cpp files programmatically?
        .file("Nes_Snd_Emu-0.1.7/nes_apu/apu_snapshot.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Blip_Buffer.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Multi_Buffer.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Nes_Apu.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Nes_Namco.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Nes_Oscs.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Nes_Vrc6.cpp")
        .file("Nes_Snd_Emu-0.1.7/nes_apu/Nonlinear_Buffer.cpp")
        .file("src/shim.cpp")
        .flag("-std=c++14")
        .flag_if_supported("-Wno-deprecated")
        .flag_if_supported("-Wno-multichar")
        .flag_if_supported("-Wno-overflow")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-value")
        .compile("blapu");
}
