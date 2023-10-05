use crate::{math::Real, sample::Sample};

pub struct Agc {
    a: Real,
    mu: Real,
    target_level_squared: Real,
}

impl Agc {
    pub fn new(mu: Real, target_level: Real) -> Self {
        Self {
            a: 1.0,
            mu,
            target_level_squared: target_level.powi(2),
        }
    }

    pub fn process_sample<S: Sample>(&self, sample: S) -> S {
        sample * self.a
    }

    pub fn process_inplace<S: Sample>(&self, buffer: &mut [S]) {
        for slot in buffer {
            *slot = self.process_sample(*slot);
        }
    }

    pub fn feedback<S: Sample>(&mut self, sample: S) {
        // SRD "Naive" formula
        self.a = self.a
            - self.mu * self.a.signum() * (sample.magnitude_squared() - self.target_level_squared);

        // SRD "Least Squares" formula
        // self.a = self.a
        //     - self.mu
        //         * (sample.magnitude_squared() - self.target_level_squared)
        //         * (sample.magnitude_squared() / self.a);
    }
}
