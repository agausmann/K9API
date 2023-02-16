use k9api_dsp::amplify;
use k9api_dsp::filter::Fir;
use k9api_dsp::math::Real;
use k9api_dsp::wave::Sine;

fn main() {
    let sample_rate = 8000;
    let carrier_frequency = 800.0;
    let mut carrier = Sine::new(sample_rate as Real / carrier_frequency, 0.0);

    let bytes = b"Hello World";
    let symbol_rate = 31.25;
    let sps = sample_rate as Real / symbol_rate;
    let mut sample_position = 0.0;
    let mut bit_index = 0;

    // TODO do filtering at a lower sample rate e.g. 1kHz and then resample
    // This will reduce the filter size a lot.
    let filter_size = (sps as usize) * 4 + 1;
    let rolloff = 1.0;
    let mut symbol_filter = Fir::raised_cosine(filter_size, rolloff, sps);

    let generate_samples = move |buffer: &mut [Real]| {
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
        symbol_filter.process_inplace(buffer);
        amplify(sps, buffer);

        // Generate and modulate carrier
        for slot in buffer.iter_mut() {
            let modulation = *slot;
            *slot = carrier.next() * modulation;
        }

        // Attenuation to avoid clipping on filter overshoot
        amplify(0.5, buffer);
    };

    //to_audio_device(sample_rate, generate_samples);
    to_wav_file(sample_rate, generate_samples);
}

fn to_audio_device(sample_rate: u32, mut generator: impl FnMut(&mut [Real]) + Send + 'static) {
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
        .build_output_stream::<Real, _, _>(
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

fn to_wav_file(sample_rate: u32, mut generator: impl FnMut(&mut [Real])) {
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
