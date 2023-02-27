use crate::math::Real;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::Normal;

pub struct Awgn<R = ThreadRng> {
    distr: Normal<Real>,
    rng: R,
}

impl Awgn {
    pub fn new(std_dev: Real) -> Self {
        Self::with_rng(thread_rng(), std_dev)
    }
}

impl<R: Rng> Awgn<R> {
    pub fn with_rng(rng: R, std_dev: Real) -> Self {
        Self {
            distr: Normal::new(0.0, std_dev).unwrap(),
            rng,
        }
    }

    pub fn apply(&mut self, buffer: &mut [Real]) {
        for slot in buffer {
            *slot += self.rng.sample(&self.distr);
        }
    }
}
