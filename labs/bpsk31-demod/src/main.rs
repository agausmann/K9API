use hound::{WavReader, WavWriter};
use k9api_dsp::{
    filter::{Passband, Window, WindowMethod},
    math::Real,
    pll::Costas,
};

fn main() {
    let wav_file = WavReader::open("bpsk31.wav").expect("cannot open `bpsk31.wav`");
    let mut filtered =
        WavWriter::create("filtered.wav", wav_file.spec()).expect("cannot create `filtered.wav`");
    let mut baseband =
        WavWriter::create("baseband.wav", wav_file.spec()).expect("cannot create `baseband.wav`");
    let mut carrier =
        WavWriter::create("carrier.wav", wav_file.spec()).expect("cannot create `carrier.wav`");

    let sample_rate = wav_file.spec().sample_rate;
    let symbol_rate = 31.25;
    let sps = sample_rate as Real / symbol_rate;
    let carrier_freq = 800.0;

    let decimation_factor = 16;

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

    for result in wav_file.into_samples() {
        let sample: i16 = result.unwrap();
        let sample = sample as Real / i16::MAX as Real;
        let filtered_sample = bpf.process_sample(sample);
        let pll_out = costas.process(filtered_sample);
        filtered
            .write_sample((filtered_sample * 32767.0) as i16)
            .unwrap();
        baseband
            .write_sample((pll_out.baseband_i * 32767.0) as i16)
            .unwrap();
        carrier
            .write_sample((pll_out.carrier_i * 32767.0) as i16)
            .unwrap();
    }
}
