use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("backend/Nes_Snd_Emu/nes_apu/Nes_Apu.h");
        include!("backend/include/shim.h");

        #[cxx_name = "Nes_Apu"]
        type NesApu;

        fn nes_apu_new() -> UniquePtr<NesApu>;
        // fn write_register(
        //     self: Pin<&mut NesApu>,
        //     time: i32,
        //     addr: u16,
        //     data: u8,
        // );
        // fn read_status(self: Pin<&mut NesApu>, time: i32) -> u8;
        fn end_frame(self: Pin<&mut NesApu>, time: i32);
    }
}

pub struct Apu {
    nes_apu: UniquePtr<ffi::NesApu>,
}

impl Apu {
    pub fn new() -> Apu {
        let nes_apu = ffi::nes_apu_new();

        Apu { nes_apu }
    }

    pub fn end_frame(&mut self, time: i32) {
        self.nes_apu.pin_mut().end_frame(time);
    }
}
