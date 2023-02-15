use std::f32::consts::{PI, SQRT_2, TAU};

fn main() {
    let sample_rate = 8000;
    let carrier_frequency = 800.0;
    let period = sample_rate as f32 / carrier_frequency;
    let phase_increment = period.recip();
    let mut phase = 0.0;

    let bytes = b"Hello World";
    let symbol_rate = 31.25;
    let sps = sample_rate as f32 / symbol_rate;
    let mut sample_position = 0.0;
    let mut bit_index = 0;

    // TODO do filtering at a lower sample rate e.g. 1kHz and then resample
    // This will reduce the filter size a lot.
    let filter_period = sps;
    let filter_size = (filter_period as isize) * 4 + 1;
    let filter_center = filter_size / 2;
    let rolloff = 1.0;
    let filter_taps: Vec<f32> = (0..filter_size)
        .map(|i| {
            let t = (i - filter_center) as f32;
            rc(t, rolloff, filter_period)
        })
        .collect();
    let mut filter_buffer = vec![0.0; filter_size as usize].into_boxed_slice();
    let mut filter_index = 0;

    let generate_samples = move |buffer: &mut [f32]| {
        // Calculate phase offsets
        for slot in buffer.iter_mut() {
            // TODO implement Varicode and differential coding
            let bit = (bytes[bit_index / 8] >> (bit_index % 8)) & 1;
            if sample_position < 1.0 {
                *slot = if bit == 0 { -1.0 } else { 1.0 };
            } else {
                *slot = 0.0;
            }

            sample_position += 1.0;
            if sample_position >= sps {
                sample_position -= sps;
                bit_index = (bit_index + 1) % (bytes.len() * 8);
            }
        }

        // Cosine filter
        for slot in buffer.iter_mut() {
            let sample = *slot;
            filter_buffer[filter_index] = sample;
            *slot = filter_period
                * filter_buffer[filter_index..]
                    .iter()
                    .chain(&filter_buffer[..filter_index])
                    .zip(&filter_taps)
                    .map(|(&s, &tap)| tap * s)
                    .sum::<f32>();

            filter_index = (filter_index + 1) % filter_buffer.len();
        }

        // Generate and modulate carrier
        for slot in buffer.iter_mut() {
            let modulation = *slot;
            *slot = modulation * (phase * TAU).sin();
            phase = (phase + phase_increment) % 1.0;
        }

        // Attenuate
        for slot in buffer.iter_mut() {
            *slot *= 0.5;
        }
    };

    //to_audio_device(sample_rate, generate_samples);
    to_wav_file(sample_rate, generate_samples);
}

fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

fn rc(t: f32, rolloff: f32, symbol_period: f32) -> f32 {
    let tn = t / symbol_period;
    let d = 2.0 * rolloff * tn;
    if d.abs() == 1.0 {
        PI / (4.0 * symbol_period) * sinc(1.0 / (2.0 * rolloff))
    } else {
        sinc(tn) * (PI * rolloff * tn).cos() / (1.0 - d * d) / symbol_period
    }
}

fn rrc(t: f32, rolloff: f32, symbol_period: f32) -> f32 {
    if t == 0.0 {
        return (1.0 + rolloff * (4.0 / PI - 1.0)) / symbol_period;
    }

    let tn = t / symbol_period;
    let x = 4.0 * rolloff * tn;

    if x == 1.0 {
        return rolloff / (symbol_period * SQRT_2)
            * ((1.0 + 2.0 / PI) * (PI / (4.0 * rolloff)).sin()
                + (1.0 - 2.0 / PI) * (PI / (4.0 * rolloff).cos()));
    }

    ((PI * tn * (1.0 - rolloff)).sin() + x * (PI * tn * (1.0 + rolloff)).cos())
        / (PI * tn * (1.0 - x * x))
        / symbol_period
}

fn to_audio_device(sample_rate: u32, mut generator: impl FnMut(&mut [f32]) + Send + 'static) {
    use cpal::traits::*;
    use cpal::{BufferSize, SampleRate, StreamConfig};

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no default output device");

    let output_config = StreamConfig {
        channels: 1,
        sample_rate: SampleRate(sample_rate),
        buffer_size: BufferSize::Default,
    };
    let output_stream = device
        .build_output_stream::<f32, _, _>(
            &output_config,
            move |buffer, _info| {
                generator(buffer);
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

fn to_wav_file(sample_rate: u32, mut generator: impl FnMut(&mut [f32])) {
    use hound::{WavSpec, WavWriter};
    use std::fs::File;
    use std::io::BufWriter;

    let mut writer = WavWriter::new(
        BufWriter::new(File::create("bpsk31.wav").expect("cannot create file")),
        WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("cannot write wav file");

    let mut sample_buffer = vec![0.0; sample_rate as usize / 100];

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
