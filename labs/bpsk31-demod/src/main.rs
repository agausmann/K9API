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
            // TODO phase correction and value decision
            // TODO varicode decoding
            symbols
                .write_sample((bit_sample.i * 32767.0) as i16)
                .unwrap();
            symbols
                .write_sample((bit_sample.q * 32767.0) as i16)
                .unwrap();
        }
    }
    baseband.flush().unwrap();
    baseband.finalize().unwrap();
    symbols.flush().unwrap();
    symbols.finalize().unwrap();
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
