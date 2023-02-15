pub mod buffer;
pub mod filter;
pub mod math;
pub mod resample;

pub type Sample = f32;

/// Amplify (or attenuate) samples by multiplying by a constant factor.
pub fn amplify(factor: Sample, samples: &mut [Sample]) {
    for sample in samples {
        *sample *= factor;
    }
}
