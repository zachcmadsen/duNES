use std::ops::RangeInclusive;

use crate::emu::Emu;

type Reader = fn(&mut Emu, u16) -> u8;
type Writer = fn(&mut Emu, u16, u8);

pub struct Bus<const N: usize> {
    readers: Box<[Reader; N]>,
    writers: Box<[Writer; N]>,
}

impl<const N: usize> Bus<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Bus<N> {
        fn read_default(_: &mut Emu, _: u16) -> u8 {
            u8::default()
        }

        fn write_default(_: &mut Emu, _: u16, _: u8) {}

        // TODO: Use the box keyword to avoid the array stack allocations?
        let readers =
            Box::<[fn(&mut Emu, u16) -> u8; N]>::new([read_default; N]);
        let writers =
            Box::<[fn(&mut Emu, u16, u8); N]>::new([write_default; N]);

        Bus { readers, writers }
    }

    pub fn set(
        &mut self,
        range: RangeInclusive<u16>,
        reader: Option<Reader>,
        writer: Option<Writer>,
    ) {
        for addr in range {
            if let Some(reader) = reader {
                self.readers[addr as usize] = reader;
            }
            if let Some(writer) = writer {
                self.writers[addr as usize] = writer;
            }
        }
    }

    pub fn reader(&self, addr: u16) -> Reader {
        self.readers[addr as usize]
    }

    pub fn writer(&self, addr: u16) -> Writer {
        self.writers[addr as usize]
    }
}
