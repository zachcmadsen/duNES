use std::{mem::MaybeUninit, time::Duration};

use cxx::UniquePtr;

use crate::emu::Emu;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("backend/Nes_Snd_Emu/nes_apu/Blip_Buffer.h");
        include!("backend/Nes_Snd_Emu/nes_apu/Nes_Apu.h");
        include!("backend/include/shim.h");

        #[cxx_name = "Blip_Buffer"]
        type BlipBuffer;

        fn new_blip_buffer() -> UniquePtr<BlipBuffer>;
        fn sample_rate(
            self: Pin<&mut BlipBuffer>,
            sample_rate: i64,
            buffer_len: i32,
        ) -> *const c_char;
        fn clock_rate(self: Pin<&mut BlipBuffer>, clock_rate: i64);
        fn end_frame(self: Pin<&mut BlipBuffer>, cycles: i64);
        fn samples_avail(self: &BlipBuffer) -> i64;
        unsafe fn read_samples(
            self: Pin<&mut BlipBuffer>,
            dst: *mut i16,
            count: i64,
            stereo: bool,
        ) -> i64;

        #[cxx_name = "Nes_Apu"]
        type NesApu;

        fn new_nes_apu() -> UniquePtr<NesApu>;
        unsafe fn output(self: Pin<&mut NesApu>, blip_buffer: *mut BlipBuffer);
        fn write_register(
            self: Pin<&mut NesApu>,
            cycles: i64,
            addr: u32,
            data: i32,
        );
        fn read_status(self: Pin<&mut NesApu>, cycles: i64) -> i32;
        fn end_frame(self: Pin<&mut NesApu>, cycles: i64);
    }
}

const CLOCK_RATE: u32 = 1789773;
pub const SAMPLE_RATE: u32 = 44100;

pub struct Apu {
    blip_buffer: UniquePtr<ffi::BlipBuffer>,
    nes_apu: UniquePtr<ffi::NesApu>,
}

impl Apu {
    pub fn new() -> Apu {
        let mut blip_buffer = ffi::new_blip_buffer();
        blip_buffer.pin_mut().sample_rate(
            SAMPLE_RATE as i64,
            Duration::from_secs(1).as_millis() as i32,
        );
        blip_buffer.pin_mut().clock_rate(CLOCK_RATE as i64);

        let mut nes_apu = ffi::new_nes_apu();
        let blip_buffer_ptr = blip_buffer.into_raw();
        unsafe { nes_apu.pin_mut().output(blip_buffer_ptr) };
        let blip_buffer = unsafe { UniquePtr::from_raw(blip_buffer_ptr) };

        Apu { blip_buffer, nes_apu }
    }

    pub fn samples(&self) -> u64 {
        self.blip_buffer.samples_avail() as u64
    }

    pub fn tick(&mut self) {
        self.nes_apu.pin_mut().end_frame(1);
        self.blip_buffer.pin_mut().end_frame(1);
    }

    /// Fills `dst` with samples.
    pub fn fill(&mut self, dst: &mut [MaybeUninit<i16>]) {
        if dst.is_empty() {
            return;
        }

        unsafe {
            // TODO: Check that the number of samples read equals dst.len().
            self.blip_buffer.pin_mut().read_samples(
                dst.as_mut_ptr() as *mut _,
                dst.len() as i64,
                false,
            );
        };
    }
}

pub fn read(emu: &mut Emu, _: u16) -> u8 {
    emu.apu.nes_apu.pin_mut().read_status(1) as u8
}

pub fn write(emu: &mut Emu, addr: u16, data: u8) {
    emu.apu.nes_apu.pin_mut().write_register(1, addr as u32, data as i32);
}
