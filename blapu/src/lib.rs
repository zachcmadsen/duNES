use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("blapu/Nes_Snd_Emu-0.1.7/nes_apu/Nes_Apu.h");
        include!("blapu/include/shim.h");

        #[cxx_name = "Nes_Apu"]
        type NesApu;

        fn new_nes_apu() -> UniquePtr<NesApu>;
        fn end_frame(self: Pin<&mut NesApu>, end_time: i64);
    }
}

pub struct NesApu(UniquePtr<ffi::NesApu>);

impl NesApu {
    pub fn new() -> NesApu {
        NesApu(ffi::new_nes_apu())
    }

    pub fn end_frame(&mut self, end_time: i64) {
        self.0.pin_mut().end_frame(end_time);
    }
}
