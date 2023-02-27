use hound::{WavReader, WavWriter};
use k9api_dsp::{filter::Fir, math::Real, pll::Costas};

fn main() {
    let wav_file = WavReader::open("bpsk31.wav").expect("cannot open `bpsk31.wav`");
    let mut baseband =
        WavWriter::create("baseband.wav", wav_file.spec()).expect("cannot create `baseband.wav`");
    let mut carrier =
        WavWriter::create("carrier.wav", wav_file.spec()).expect("cannot create `carrier.wav`");

    let sample_rate = wav_file.spec().sample_rate;
    let symbol_rate = 31.25;
    let sps = sample_rate as Real / symbol_rate;
    let carrier_freq = 800.0;

    let mut costas = Costas::new(
        carrier_freq / sample_rate as Real,
        0.05,
        Fir::raised_cosine(sps as usize + 1, 1.0, 0.5 * sps),
    );

    for result in wav_file.into_samples() {
        let sample: i16 = result.unwrap();
        let sample = sample as Real / i16::MAX as Real;
        let (baseband_sample, carrier_sample) = costas.process(sample);
        baseband
            .write_sample((baseband_sample * 32767.0) as i16)
            .unwrap();
        carrier
            .write_sample((carrier_sample * 32767.0) as i16)
            .unwrap();
    }
}
