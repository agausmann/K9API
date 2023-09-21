use crate::{
    filter::Fir,
    iq::IQ,
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

pub struct Output {
    pub baseband: IQ,
    pub carrier: IQ,
    pub error: Real,
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

    pub fn process(&mut self, sample: Real) -> Output {
        let angle = self.osc_phase * TAU + self.phase_offset;
        let carrier_i = cos(angle);
        let carrier_q = -sin(angle);
        let baseband_i = self.filter_i.process_sample(carrier_i * sample);
        let baseband_q = self.filter_q.process_sample(carrier_q * sample);
        let error = baseband_i * baseband_q;
        self.phase_offset += self.k * error;
        self.osc_phase = (self.osc_phase + self.carrier_freq) % 1.0;
        Output {
            baseband: IQ::new(baseband_i, baseband_q),
            carrier: IQ::new(carrier_i, carrier_q),
            error,
        }
    }
}
