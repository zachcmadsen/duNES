mod tb;

use std::sync::Arc;

use backend::Emu;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use pixels::{Pixels, SurfaceTexture};
use rtrb::{chunks::ChunkError, RingBuffer};
use tracing::warn;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::tb::triple_buffer;

pub fn run(rom: Vec<u8>) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("duNES")
            .with_inner_size(LogicalSize::new(0, 0))
            .build(&event_loop)
            .unwrap(),
    );

    let size = window.inner_size();
    let surface_texture =
        SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels =
        Pixels::new(size.width, size.height, surface_texture).unwrap();

    let buffer = vec![0; pixels.frame().len()].into_boxed_slice();
    let (mut writer, reader) = triple_buffer(buffer);
    let (mut producer, mut consumer) = RingBuffer::new(2048);

    let emu_thread = std::thread::spawn({
        let window = window.clone();
        move || {
            let mut emu = Emu::new(&rom);
            // emu.ppu.on_frame(move |buffer| {
            //     writer.get_mut().copy_from_slice(buffer);
            //     writer.swap();
            //     window.request_redraw();
            // });

            loop {
                let slots = producer.slots();
                if slots > 0 {
                    // while emu.apu.samples() < slots as u64 {
                    //     emu.tick();
                    // }

                    let mut chunk =
                        producer.write_chunk_uninit(slots).unwrap();
                    let (first, second) = chunk.as_mut_slices();
                    // emu.apu.fill(first);
                    // emu.apu.fill(second);
                    unsafe { chunk.commit_all() };
                }

                std::thread::park();
            }
        }
    });

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Fixed(512),
    };
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [i16], _| {
                let chunk = match consumer.read_chunk(data.len()) {
                    Ok(chunk) => chunk,
                    Err(ChunkError::TooFewSlots(n)) => {
                        warn!("not enough samples to fill the buffer");
                        let chunk = consumer.read_chunk(n).unwrap();
                        chunk
                    }
                };

                let (first, second) = chunk.as_slices();
                let mid = first.len();
                let end = chunk.len();
                data[..mid].copy_from_slice(first);
                data[mid..end].copy_from_slice(second);
                chunk.commit_all();

                emu_thread.thread().unpark();
            },
            move |_| {},
            None,
        )
        .unwrap();
    stream.play().unwrap();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested, ..
            } => {
                pixels.frame_mut().copy_from_slice(reader.get());
                window.pre_present_notify();
                pixels.render().unwrap();
            }
            _ => (),
        })
        .unwrap();
}
