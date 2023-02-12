use std::f32::consts::TAU;

use cpal::traits::*;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no default output device");
    let output_config = device
        .default_output_config()
        .expect("no default output config");
    let sample_rate = output_config.sample_rate().0;
    let tone_frequency = 440.0;
    let period = sample_rate as f32 / tone_frequency;
    let phase_increment = period.recip();
    let mut phase = 0.0;

    let output_stream = device
        .build_output_stream::<f32, _, _>(
            &output_config.into(),
            move |buffer, _info| {
                for slot in buffer {
                    *slot = (phase * TAU).sin();
                    phase = (phase + phase_increment) % 1.0;
                }
            },
            |err| {
                eprintln!("output stream error: {}", err);
            },
            None,
        )
        .expect("cannot build output stream");

    output_stream.play().unwrap();

    loop {
        std::thread::yield_now();
    }
}
