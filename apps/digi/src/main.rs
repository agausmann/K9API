use std::time::{Duration, Instant};

use anyhow::Context;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate,
};
use rustfft::{num_complex::Complex32, num_traits::Zero, FftPlanner};
use slint::{Color, Image, Rgb8Pixel, SharedPixelBuffer, Timer, TimerMode};

fn main() -> anyhow::Result<()> {
    let main_window = MainWindow::new()?;

    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .context("no input devices found")?;

    let config = input_device
        .supported_input_configs()?
        .find(|cfg| {
            cfg.channels() == 1
                && cfg.min_sample_rate() <= SampleRate(8000)
                && cfg.max_sample_rate() >= SampleRate(8000)
                && cfg.sample_format() == SampleFormat::F32
        })
        .map(|cfg| cfg.with_sample_rate(SampleRate(8000)).config())
        .context("cannot find a suitable config")?;

    let fft = FftPlanner::<f32>::new().plan_fft_forward(256);
    let audio_main_window = main_window.as_weak();
    let mut buffer = SharedPixelBuffer::new(256, 256);
    let mut pixels =
        (0..256).flat_map(|r| (0..256).map(move |g| Rgb8Pixel::new(r as u8, g as u8, 0)));
    buffer.make_mut_slice().fill_with(|| pixels.next().unwrap());

    let mut fft_buf = vec![Complex32::zero(); 256];
    let gradient = [
        Color::from_rgb_u8(0x00, 0x00, 0x20),
        Color::from_rgb_u8(0x00, 0x00, 0x30),
        Color::from_rgb_u8(0x00, 0x00, 0x50),
        Color::from_rgb_u8(0x00, 0x00, 0x91),
        Color::from_rgb_u8(0x1e, 0x90, 0xff),
        Color::from_rgb_u8(0xff, 0xff, 0xff),
        Color::from_rgb_u8(0xff, 0xff, 0x00),
        Color::from_rgb_u8(0xfe, 0x6d, 0x16),
        Color::from_rgb_u8(0xff, 0x00, 0x00),
        Color::from_rgb_u8(0xc6, 0x00, 0x00),
        Color::from_rgb_u8(0x9f, 0x00, 0x00),
        Color::from_rgb_u8(0x75, 0x00, 0x00),
        Color::from_rgb_u8(0x4a, 0x00, 0x00),
    ];

    let input_stream = input_device.build_input_stream(
        &config,
        move |data: &[f32], info| {
            let start = Instant::now();
            fft_buf.fill(Complex32::zero());
            for (a, b) in fft_buf.iter_mut().zip(data) {
                *a = Complex32::new(*b, 0.0);
            }
            fft.process(&mut fft_buf);
            buffer.make_mut_slice().copy_within(0..255 * 256, 256);
            for (a, b) in buffer.make_mut_slice()[..256].iter_mut().zip(&fft_buf) {
                // TODO runtime controls for adjusting range
                let x = (b.norm() / 20.0 + 0.1).clamp(0.0, 0.99999) * (gradient.len() - 1) as f32;
                let bin = x as usize;
                let factor = x.fract();

                let color = gradient[bin + 1].mix(&gradient[bin], factor);
                *a = Rgb8Pixel::new(color.red(), color.green(), color.blue());
            }
            let abuf = buffer.clone();
            audio_main_window
                .upgrade_in_event_loop(move |handle| {
                    let image = Image::from_rgb8(abuf);
                    handle.set_map(image);
                })
                .ok();

            eprintln!(
                "{} {}",
                start.elapsed().as_micros(),
                fft_buf
                    .iter()
                    .map(|z| z.norm())
                    .max_by(f32::total_cmp)
                    .unwrap()
            );
        },
        |err| {
            eprintln!("{:?}", err);
        },
        None,
    )?;

    input_stream.play()?;
    main_window.run()?;

    Ok(())
}

slint::slint! {
    export component MainWindow inherits Window {
        in property <image> map;

        Image {
            width: 100%;
            height: 100%;
            image-fit: ImageFit.fill;
            image-rendering: ImageRendering.smooth;
            source: map;
        }
    }
}
