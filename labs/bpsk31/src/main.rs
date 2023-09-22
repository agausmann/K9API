use std::iter::repeat;

use k9api_dsp::amplify;
use k9api_dsp::buffer::Buffer;
use k9api_dsp::channel::Awgn;
use k9api_dsp::filter::{Fir, Passband, Window, WindowMethod};
use k9api_dsp::math::Real;
use k9api_dsp::resample::Upsample;
use k9api_dsp::wave::Sine;

fn main() {
    let sample_rate = 8000;
    let carrier_frequency = 800.0;
    let mut carrier = Sine::new(sample_rate as Real / carrier_frequency, 0.0);

    let premod_factor = 16;
    let premod_sample_rate = sample_rate / premod_factor;
    let symbol_rate = 31.25;
    let sps = premod_sample_rate as f32 / symbol_rate;
    assert_eq!(premod_sample_rate, 500);
    assert_eq!(sps, 16.0);

    let preamble = repeat(false).take(80);
    let postamble = repeat(false).take(20).chain(repeat(true).take(30));

    let bytes = b"CQ CQ CQ de K9API K9API K9API pse K\n";
    let mut diff = Differential::new();
    let mut bits = preamble
        .chain(bytes.iter().flat_map(|&b| bits(VARICODE[b as usize] << 2)))
        .chain(postamble)
        .map(move |bit| diff.process(bit));

    let filter_size = (sps as usize) * 4 + 1;
    let rolloff = 1.0;
    let mut symbol_filter = Fir::raised_cosine(filter_size, rolloff, sps);
    let mut filter_buffer = vec![0.0; sps as usize];

    let upsample_filter = WindowMethod {
        gain: premod_factor as Real,
        sample_rate: sample_rate as Real,
        passband: Passband::LowPass {
            cutoff: 0.5 * premod_sample_rate as Real,
        },
        transition_width: None,
        num_taps: Some(65),
        window: Window::HAMMING,
    };
    let mut upsample = Upsample::new(premod_factor, upsample_filter.build());
    let premod_chunk_size = sps as usize * premod_factor;
    assert_eq!(premod_chunk_size, 256);

    let premod_samples = move |buffer: &mut [Real]| {
        debug_assert_eq!(buffer.len(), premod_chunk_size);

        filter_buffer.fill(0.0);
        filter_buffer[0] = match bits.next() {
            Some(true) => 1.0,
            Some(false) => -1.0,
            None => 0.0,
        };

        symbol_filter.process_inplace(&mut filter_buffer);
        upsample.process(&filter_buffer, buffer);
    };
    let mut premod_buffer = Buffer::new(premod_samples, premod_chunk_size, premod_chunk_size);

    let mut awgn = Awgn::new(0.1);

    let generate_samples = move |buffer: &mut [Real]| {
        for chunk in buffer.chunks_mut(premod_chunk_size) {
            premod_buffer.fill_buffer(chunk.len());
            chunk.copy_from_slice(&premod_buffer.available()[..chunk.len()]);
            premod_buffer.consume(chunk.len());
        }

        // Generate and modulate carrier
        for slot in buffer.iter_mut() {
            let modulation = *slot;
            *slot = carrier.next() * modulation;
        }

        // Apply channel model
        amplify(0.2, buffer);
        awgn.apply(buffer);
    };

    //to_audio_device(sample_rate as u32, generate_samples);
    to_wav_file(sample_rate as u32, generate_samples);
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

    for _frame in 0..1500 {
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

fn bits(x: u32) -> impl Iterator<Item = bool> + Clone {
    // Extract bits, starting from the most significant 1-bit.
    (0..=x.ilog2()).rev().map(move |i| ((x >> i) & 1) != 0)
}

#[rustfmt::skip]
const VARICODE: [u32; 128] = [
    // 0x00
    0b1010101011, 0b1011011011, 0b1011101101, 0b1101110111,
    0b1011101011, 0b1101011111, 0b1011101111, 0b1011111101,
    0b1011111111, 0b11101111, 0b11101, 0b1101101111,
    0b1011011101, 0b11111, 0b1101110101, 0b1110101011,
    // 0x10
    0b1011110111, 0b1011110101, 0b1110101101, 0b1110101111,
    0b1101011011, 0b1101101011, 0b1101101101, 0b1101010111,
    0b1101111011, 0b1101111101, 0b1110110111, 0b1101010101,
    0b1101011101, 0b1110111011, 0b1011111011, 0b1101111111,
    // 0x20
    0b1, 0b111111111, 0b101011111, 0b111110101,
    0b111011011, 0b1011010101, 0b1010111011, 0b101111111,
    0b11111011, 0b11110111, 0b101101111, 0b111011111,
    0b1110101, 0b110101, 0b1010111, 0b110101111,
    // 0x30
    0b10110111, 0b10111101, 0b11101101, 0b11111111,
    0b101110111, 0b101011011, 0b101101011, 0b110101101,
    0b110101011, 0b110110111, 0b11110101, 0b110111101,
    0b111101101, 0b1010101, 0b111010111, 0b1010101111,
    // 0x40
    0b1010111101, 0b1111101, 0b11101011, 0b10101101,
    0b10110101, 0b1110111, 0b11011011, 0b11111101,
    0b101010101, 0b1111111, 0b111111101, 0b101111101,
    0b11010111, 0b10111011, 0b11011101, 0b10101011,
    // 0x50
    0b11010101, 0b111011101, 0b10101111, 0b1101111,
    0b1101101, 0b101010111, 0b110110101, 0b101011101,
    0b101110101, 0b101111011, 0b1010101101, 0b111110111,
    0b111101111, 0b111111011, 0b1010111111, 0b101101101,
    // 0x60
    0b1011011111, 0b1011, 0b1011111, 0b101111,
    0b101101, 0b11, 0b111101, 0b1011011,
    0b101011, 0b1101, 0b111101011, 0b10111111,
    0b11011, 0b111011, 0b1111, 0b111,
    // 0x70
    0b111111, 0b110111111, 0b10101, 0b10111,
    0b101, 0b110111, 0b1111011, 0b1101011,
    0b11011111, 0b1011101, 0b111010101, 0b1010110111,
    0b110111011, 0b1010110101, 0b1011010111, 0b1110110101,
];

#[derive(Clone)]
struct Differential {
    acc: bool,
}

impl Differential {
    fn new() -> Self {
        Self { acc: false }
    }

    fn process(&mut self, bit: bool) -> bool {
        self.acc ^= !bit;
        self.acc
    }
}
