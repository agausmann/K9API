use hound::{WavReader, WavSpec, WavWriter};
use k9api_dsp::{
    filter::{Passband, Window, WindowMethod},
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
            channels: 1,
            sample_rate: sample_rate / decimation_factor as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("cannot create `baseband.wav`");

    let bpf_design = WindowMethod {
        gain: 1.0,
        sample_rate: sample_rate as Real,
        passband: Passband::centered_band_pass(carrier_freq, 100.0),
        transition_width: Some(50.0),
        num_taps: None,
        window: Window::HAMMING,
    };
    let mut bpf = bpf_design.build();

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

    let mut costas = Costas::new(
        carrier_freq / sample_rate as Real,
        0.05,
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
    let mut downsample = Downsample::new(decimation_factor, downsample_design.build());

    let mut pll_input = vec![0.0; decimation_factor];
    let mut pll_output = vec![IQ::ZERO; decimation_factor];
    let mut input_samples = wav_file.into_samples().peekable();

    let mut baseband_sample = move || {
        if input_samples.peek().is_none() {
            return None;
        }

        pll_input.fill_with(|| {
            input_samples.next().transpose().unwrap().unwrap_or(0) as Real / i16::MAX as Real
        });
        bpf.process_inplace(&mut pll_input);

        for (pin, pout) in pll_input.iter().zip(&mut pll_output) {
            *pout = costas.process(*pin).baseband;
        }

        let mut output = [IQ::ZERO];
        downsample.process(&pll_output, &mut output);
        Some(output[0])
    };

    while let Some(bb) = baseband_sample() {
        baseband.write_sample((bb.i * 32767.0) as i16).unwrap();
    }
    baseband.flush().unwrap();
    baseband.finalize().unwrap();
}
