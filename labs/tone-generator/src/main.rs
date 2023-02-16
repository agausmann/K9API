use k9api_dsp::amplify;
use k9api_dsp::math::Real;
use k9api_dsp::wave::Sine;

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
    let mut sine = Sine::new(sample_rate as Real / tone_frequency, 0.0);

    println!("sample rate {}", sample_rate);

    let output_stream = device
        .build_output_stream::<Real, _, _>(
            &output_config.into(),
            move |buffer, _info| {
                sine.fill(buffer);
                amplify(0.5, buffer);
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
