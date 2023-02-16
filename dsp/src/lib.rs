pub mod buffer;
pub mod filter;
pub mod math;
pub mod resample;
pub mod wave;

use crate::math::Real;

/// Amplify (or attenuate) samples by multiplying by a constant factor.
pub fn amplify(factor: Real, samples: &mut [Real]) {
    for sample in samples {
        *sample *= factor;
    }
}
