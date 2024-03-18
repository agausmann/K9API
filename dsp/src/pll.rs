use crate::{filter::Fir, iq::IQ, math::Real, wave::Oscillator};

pub struct Costas {
    k: Real,
    osc: Oscillator,
    filter: Fir<IQ>,
    phase_offset: Real,
}

pub struct Output {
    pub baseband: IQ,
    pub carrier: IQ,
    pub error: Real,
}

impl Costas {
    pub fn new(carrier_freq: Real, k: Real, filter: Fir<IQ>) -> Self {
        Self {
            k,
            osc: Oscillator::new(1.0 / carrier_freq, 0.0),
            filter,
            phase_offset: 0.0,
        }
    }

    pub fn process(&mut self, sample: Real) -> Output {
        let carrier = self.osc.next_with_offset(self.phase_offset);
        let baseband = self.filter.process_sample(carrier * sample);
        let bnorm = baseband.unit();
        let error = bnorm.i * bnorm.q;
        self.phase_offset -= self.k * error;
        Output {
            baseband,
            carrier,
            error,
        }
    }
}
