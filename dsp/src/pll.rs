use crate::{
    filter::Fir,
    math::{cos, sin, Real, TAU},
};

pub struct Costas {
    carrier_freq: Real,
    k: Real,
    osc_phase: Real,
    phase_offset: Real,
    filter_i: Fir,
    filter_q: Fir,
}

impl Costas {
    pub fn new(carrier_freq: Real, k: Real, filter: Fir) -> Self {
        Self {
            carrier_freq,
            k,
            osc_phase: 0.0,
            phase_offset: 0.0,
            filter_i: filter.clone(),
            filter_q: filter,
        }
    }

    pub fn process(&mut self, sample: Real) -> (Real, Real) {
        let angle = self.osc_phase * TAU + self.phase_offset;
        let xi = sample * cos(angle);
        let xq = sample * -sin(angle);
        let yi = self.filter_i.process_sample(xi);
        let yq = self.filter_q.process_sample(xq);
        let err = yi * yq;
        self.phase_offset += self.k * err;
        self.osc_phase = (self.osc_phase + self.carrier_freq) % 1.0;
        (yi, cos(angle))
    }
}
