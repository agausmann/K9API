use std::collections::HashMap;

use hound::{WavReader, WavSpec, WavWriter};
use k9api_dsp::{
    agc::Agc,
    early_late::EarlyLate,
    filter::{Fir, Passband, Window, WindowMethod},
    iq::IQ,
    math::Real,
    pll::Costas,
    resample::Downsample,
    sample::Sample,
};

fn main() {
    let wav_file = WavReader::open("bpsk31.wav").expect("cannot open `bpsk31.wav`");

    let sample_rate = wav_file.spec().sample_rate;
    let symbol_rate = 31.25;
    let sps = sample_rate as Real / symbol_rate;
    let carrier_freq = 800.0;

    let decimation_factor = 16;

    let mut baseband = WavWriter::create(
        "baseband.wav",
        WavSpec {
            channels: 2,
            sample_rate: sample_rate / decimation_factor as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("cannot create `baseband.wav`");

    let mut symbols = WavWriter::create(
        "symbols.wav",
        WavSpec {
            channels: 2,
            sample_rate: 31, // 31.25 :(
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("cannot create `symbols.wav`");

    let mut input_samples = wav_file.into_samples().peekable();
    let mut pll_input = vec![0.0; decimation_factor];

    let mut af_domain = AFDomain::new(sample_rate, carrier_freq, decimation_factor);

    let mut matched_filter = Fir::raised_cosine(65, 1.0, 16.0);
    let mut timing = EarlyLate::new(0.0, 16);

    let mut varicode_lookup = HashMap::new();
    for (i, &code) in VARICODE.iter().enumerate() {
        varicode_lookup.insert(code, i as u8);
    }

    let mut differential = InverseDifferential::new();
    let mut bits: u32 = 0;
    let mut output = String::new();

    loop {
        if input_samples.peek().is_none() {
            break;
        }

        pll_input.fill_with(|| {
            input_samples.next().transpose().unwrap().unwrap_or(0) as Real / i16::MAX as Real
        });

        let bb = af_domain.process_af(&mut pll_input);

        baseband.write_sample((bb.i * 32767.0) as i16).unwrap();
        baseband.write_sample((bb.q * 32767.0) as i16).unwrap();

        if let Some(bit_sample) = timing.process(matched_filter.process_sample(bb)) {
            af_domain.agc_feedback(bit_sample);

            symbols
                .write_sample((bit_sample.i * 32767.0) as i16)
                .unwrap();
            symbols
                .write_sample((bit_sample.q * 32767.0) as i16)
                .unwrap();

            // TODO may need phase correction. Right now it seems to be in phase
            let bit = !differential.process(bit_sample.i > 0.0);

            bits <<= 1;
            bits |= bit as u32;
            if bits != 0 && bits & 0b11 == 0 {
                println!("{:b}", bits);
                // End-of-character (two zeros) detected, and not idle:
                output.push(
                    varicode_lookup
                        .get(&(bits >> 2))
                        .map(|&byte| byte as char)
                        .unwrap_or(char::REPLACEMENT_CHARACTER),
                );

                bits = 0;
            }
        }
    }
    baseband.flush().unwrap();
    baseband.finalize().unwrap();
    symbols.flush().unwrap();
    symbols.finalize().unwrap();

    println!("{:?}", output);
}

struct AFDomain {
    bpf: Fir<Real>,
    agc: Agc,
    costas: Costas,
    downsample: Downsample<IQ>,
    pll_output: Vec<IQ>,
}

impl AFDomain {
    pub fn new(sample_rate: u32, carrier_freq: Real, decimation_factor: usize) -> Self {
        let bpf_design = WindowMethod {
            gain: 1.0,
            sample_rate: sample_rate as Real,
            passband: Passband::centered_band_pass(carrier_freq, 100.0),
            transition_width: Some(50.0),
            num_taps: None,
            window: Window::HAMMING,
        };
        let bpf = bpf_design.build();

        let agc = Agc::new(0.1, 0.5);

        let loop_filter_design = WindowMethod {
            gain: 1.0,
            sample_rate: sample_rate as Real,
            passband: Passband::LowPass {
                cutoff: 0.5 * sample_rate as Real / decimation_factor as Real,
            },
            transition_width: Some(100.0),
            num_taps: None,
            window: Window::HAMMING,
        };

        let costas = Costas::new(
            carrier_freq / sample_rate as Real,
            0.01,
            loop_filter_design.build(),
        );

        let downsample_design = WindowMethod {
            gain: 1.0,
            sample_rate: sample_rate as Real,
            passband: Passband::LowPass { cutoff: 50.0 },
            transition_width: Some(50.0),
            num_taps: None,
            window: Window::HAMMING,
        };
        let downsample = Downsample::new(decimation_factor, downsample_design.build());

        let pll_output = vec![IQ::ZERO; decimation_factor];
        Self {
            bpf,
            agc,
            costas,
            downsample,
            pll_output,
        }
    }

    pub fn process_af(&mut self, buf: &mut [Real]) -> IQ {
        self.bpf.process_inplace(buf);
        self.agc.process_inplace(buf);

        for (pin, pout) in buf.iter().zip(&mut self.pll_output) {
            *pout = self.costas.process(*pin).baseband;
        }

        let mut output = [IQ::ZERO];
        self.downsample.process(&self.pll_output, &mut output);
        output[0]
    }

    pub fn agc_feedback<S: Sample>(&mut self, sample: S) {
        self.agc.feedback(sample);
    }
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
struct InverseDifferential {
    last: bool,
}

impl InverseDifferential {
    fn new() -> Self {
        Self { last: false }
    }

    fn process(&mut self, bit: bool) -> bool {
        let result = self.last != bit;
        self.last = bit;
        result
    }
}
