pub mod agc;
pub mod buffer;
pub mod channel;
pub mod codec;
pub mod early_late;
pub mod filter;
pub mod iq;
pub mod math;
pub mod modem;
pub mod pll;
pub mod resample;
pub mod sample;
pub mod wave;

use sample::Sample;

use crate::math::Real;

/// Amplify (or attenuate) samples by multiplying by a constant factor.
pub fn amplify<T: Sample>(factor: Real, samples: &mut [T]) {
    for sample in samples {
        *sample *= factor;
    }
}
