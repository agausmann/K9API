use cpal::traits::*;
use cpal::SampleRate;
use k9api_dsp::math::PI;
use k9api_dsp::modem::fm::FmDemod;
use k9api_dsp::{iq::IQ, math::Real};
use num_complex::Complex;
use soapysdr::Device;

fn main() -> anyhow::Result<()> {
    for dev in soapysdr::enumerate("")? {
        println!("{}", dev);
    }

    let dev = Device::new("type=rtlsdr")?;

    let sample_rate = 250000;

    let mut in_stream = dev.rx_stream::<Complex<Real>>(&[0])?;
    let mtu = in_stream.mtu()?;
    dev.set_frequency(soapysdr::Direction::Rx, 0, 89.7e6, "")?;
    dev.set_sample_rate(soapysdr::Direction::Rx, 0, sample_rate as f64)?;

    let ahost = cpal::default_host();
    let adev = ahost
        .default_output_device()
        .expect("no default output device");
    let mut output_configs: Vec<_> = adev
        .supported_output_configs()?
        .filter_map(|cfg| cfg.try_with_sample_rate(SampleRate(sample_rate)))
        .filter(|cfg| cfg.channels() == 1)
        .collect();

    output_configs.sort_by_key(|cfg| {
        (
            cfg.sample_format().is_float(),
            cfg.sample_format().sample_size(),
        )
    });

    let output_config = output_configs.first().unwrap().config();

    let mut complex_buffer: Vec<Complex<Real>> = vec![Default::default(); mtu];
    let mut complex_buffer_position = 0;
    let mut fm = FmDemod::new();

    in_stream.activate(None)?;

    let mut generator = move |buffer: &mut [Real]| {
        let mut rem = buffer;
        while rem.len() > 0 {
            if complex_buffer_position == 0 {
                let read = in_stream
                    .read(
                        &mut [&mut complex_buffer[complex_buffer_position..]],
                        1000000,
                    )
                    .unwrap();
                complex_buffer_position += read;
            }
            for (i, o) in complex_buffer[..complex_buffer_position]
                .iter()
                .zip(rem.iter_mut())
            {
                *o = fm.next(IQ::from(*i)) / PI;
            }
            let copied = complex_buffer_position.min(rem.len());
            complex_buffer.copy_within(copied..complex_buffer_position, 0);
            complex_buffer_position -= copied;
            rem = &mut rem[copied..];
        }
    };

    let output_stream = adev
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
