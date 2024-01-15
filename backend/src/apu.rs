#![cfg_attr(test, allow(dead_code))]

use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
    time::Duration,
};

use cxx::{type_id, UniquePtr};

use crate::Emu;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("backend/Nes_Snd_Emu/nes_apu/Blip_Buffer.h");
        include!("backend/Nes_Snd_Emu/nes_apu/Nes_Apu.h");
        include!("backend/include/shim.h");

        #[namespace = "std"]
        #[cxx_name = "error_condition"]
        type ErrorCondition = super::ErrorCondition;

        #[cxx_name = "Blip_Buffer"]
        type BlipBuffer;

        fn blip_buffer_new() -> UniquePtr<BlipBuffer>;
        fn set_sample_rate(
            self: Pin<&mut BlipBuffer>,
            sample_rate: i32,
            buffer_len: i32,
        ) -> ErrorCondition;
        fn clock_rate(self: Pin<&mut BlipBuffer>, clock_rate: i32);
        fn end_frame(self: Pin<&mut BlipBuffer>, time: i32);
        unsafe fn read_samples(
            self: Pin<&mut BlipBuffer>,
            dst: *mut i16,
            count: i32,
            stereo: bool,
        ) -> i32;

        #[cxx_name = "Nes_Apu"]
        type NesApu;

        fn nes_apu_new() -> UniquePtr<NesApu>;
        unsafe fn set_output(self: Pin<&mut NesApu>, buffer: *mut BlipBuffer);
        fn write_register(
            self: Pin<&mut NesApu>,
            time: i32,
            addr: u16,
            data: u8,
        );
        fn read_status(self: Pin<&mut NesApu>, time: i32) -> u8;
        fn end_frame(self: Pin<&mut NesApu>, time: i32);
    }
}

const SAMPLE_RATE: i32 = 44100;
const CLOCK_RATE: i32 = 1789773;

#[repr(C)]
struct ErrorCondition {
    _value: i32,
    _cat: *const c_void,
}
const _: () = assert!(mem::size_of::<ErrorCondition>() == 16);

unsafe impl cxx::ExternType for ErrorCondition {
    type Id = type_id!("std::error_condition");
    type Kind = cxx::kind::Trivial;
}

pub struct Apu {
    buffer: UniquePtr<ffi::BlipBuffer>,
    nes_apu: UniquePtr<ffi::NesApu>,
}

impl Apu {
    pub fn new() -> Apu {
        let mut buffer = ffi::blip_buffer_new();
        buffer.pin_mut().set_sample_rate(
            SAMPLE_RATE,
            Duration::from_millis(1000).as_millis() as i32,
        );
        buffer.pin_mut().clock_rate(CLOCK_RATE);

        let mut nes_apu = ffi::nes_apu_new();
        let buffer_ptr = buffer.into_raw();
        unsafe {
            nes_apu.pin_mut().set_output(buffer_ptr);
        }
        // TODO: Does Nes_APU do anything with the buffer pointer on
        // destruction? Is there a risk of a double free?
        let buffer = unsafe { UniquePtr::from_raw(buffer_ptr) };

        Apu { buffer, nes_apu }
    }
}

pub fn tick(emu: &mut Emu) {
    emu.apu.nes_apu.pin_mut().end_frame(1);
    emu.apu.buffer.pin_mut().end_frame(1);
}

pub fn read(emu: &mut Emu) -> u8 {
    emu.apu.nes_apu.pin_mut().read_status(0)
}

pub fn write(emu: &mut Emu, addr: u16, data: u8) {
    emu.apu.nes_apu.pin_mut().write_register(0, addr, data);
}

/// Fills `dst` with samples.
pub fn fill(emu: &mut Emu, dst: &mut [MaybeUninit<i16>]) {
    if dst.is_empty() {
        return;
    }

    unsafe {
        // TODO: Check that the number of samples read equals dst.len().
        emu.apu.buffer.pin_mut().read_samples(
            dst.as_mut_ptr() as *mut _,
            dst.len() as i32,
            false,
        );
    };
}
