use k9api_dsp::filter::{Passband, Window, WindowMethod};
use k9api_dsp::math::{sin, Real, TAU};

fn main() {
    let sample_rate = 8000; // Hz

    let mut frequency = 100.0; // Hz
    let mut phase = 0.0;
    let slew_rate = 200.0; // Hz / s
    let cutoff = 1000.0;
    let transition_width = 100.0;

    let low_pass = WindowMethod {
        gain: 1.0,
        sample_rate: sample_rate as Real,
        passband: Passband::LowPass { cutoff },
        transition_width: Some(transition_width),
        num_taps: None,
        window: Window::HAMMING,
    };
    let high_pass = WindowMethod {
        passband: Passband::HighPass { cutoff },
        ..low_pass
    };
    let band_pass = WindowMethod {
        passband: Passband::centered_band_pass(cutoff, 500.0),
        ..low_pass
    };
    let band_reject = WindowMethod {
        passband: Passband::centered_band_reject(cutoff, 500.0),
        ..low_pass
    };

    let mut low_pass = low_pass.build();
    let mut high_pass = high_pass.build();
    let mut band_pass = band_pass.build();
    let mut band_reject = band_reject.build();

    let generate_samples = move |buffer: &mut [Real]| {
        for slot in buffer.chunks_mut(5) {
            let wave = 0.5 * sin(TAU * phase);
            slot[0] = wave;
            slot[1] = low_pass.process_sample(wave);
            slot[2] = high_pass.process_sample(wave);
            slot[3] = band_pass.process_sample(wave);
            slot[4] = band_reject.process_sample(wave);
            phase = (phase + frequency / sample_rate as Real) % 1.0;
            frequency = frequency + slew_rate / sample_rate as Real;
        }
    };

    //to_audio_device(sample_rate as u32, generate_samples);
    to_wav_file(sample_rate as u32, generate_samples);
}

fn to_wav_file(sample_rate: u32, mut generator: impl FnMut(&mut [Real])) {
    use hound::{WavSpec, WavWriter};
    use std::fs::File;
    use std::io::BufWriter;
    let num_channels = 5;

    let mut writer = WavWriter::new(
        BufWriter::new(File::create("filter-debug.wav").expect("cannot create `filter-debug.wav`")),
        WavSpec {
            channels: num_channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("cannot write wav file");

    let mut sample_buffer = vec![0.0; sample_rate as usize * num_channels as usize / 100];

    for _frame in 0..1000 {
        generator(&mut sample_buffer);
        for &sample in &sample_buffer {
            let converted = (sample * 32767.0) as i16;
            writer
                .write_sample(converted)
                .expect("failed to write sample");
        }
    }
    writer.flush().unwrap();
    writer.finalize().unwrap();
}
