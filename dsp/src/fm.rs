use crate::{iq::IQ, math::Real, sample::Sample};

/// Demodulates FM from baseband IQ samples.
///
/// This simply calculates the relative phase angle between successive baseband
/// samples (between -PI and +PI). It does not handle the modulation index; that
/// would just alter the amplitude of the output.
pub struct FmDemod {
    last: IQ,
}

impl FmDemod {
    pub fn new() -> Self {
        Self { last: IQ::ZERO }
    }

    pub fn next(&mut self, sample: IQ) -> Real {
        let output = (sample * self.last.conj()).phase();
        self.last = sample;
        output
    }

    pub fn fill(&mut self, inp: &[IQ], out: &mut [Real]) {
        for (i, o) in inp.iter().zip(out) {
            *o = self.next(*i);
        }
    }
}
