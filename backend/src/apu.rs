use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("backend/Nes_Snd_Emu/nes_apu/Nes_Apu.h");
        include!("backend/include/shim.h");

        #[cxx_name = "Nes_Apu"]
        type NesApu;

        fn new_nes_apu() -> UniquePtr<NesApu>;
        fn end_frame(self: Pin<&mut NesApu>, end_time: i64);
    }
}

pub struct Apu {
    nes_apu: UniquePtr<ffi::NesApu>,
}

impl Apu {
    pub fn new() -> Apu {
        Apu { nes_apu: ffi::new_nes_apu() }
    }

    pub fn end_frame(&mut self, end_time: i64) {
        self.nes_apu.pin_mut().end_frame(end_time);
    }
}
