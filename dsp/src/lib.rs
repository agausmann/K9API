pub mod buffer;
pub mod channel;
pub mod filter;
pub mod math;
pub mod pll;
pub mod resample;
pub mod wave;

use crate::math::Real;

/// Amplify (or attenuate) samples by multiplying by a constant factor.
pub fn amplify(factor: Real, samples: &mut [Real]) {
    for sample in samples {
        *sample *= factor;
    }
}
