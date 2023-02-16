use crate::math::{Real, rc};

pub struct Fir {
    taps: Box<[Real]>,
    buffer: Box<[Real]>,
    position: usize,
}

impl Fir {
    pub fn new(taps: impl Into<Box<[Real]>>) -> Self {
        let taps = taps.into();
        let buffer = vec![0.0; taps.len()].into_boxed_slice();
        Self {
            taps,
            buffer,
            position: 0,
        }
    }

    pub fn raised_cosine(num_taps: usize, rolloff: Real, sps: Real) -> Self {
        // Ensure num_taps is odd
        let num_taps = num_taps | 1;
        let half_width = num_taps as isize / 2;
        let taps: Box<[Real]> = (-half_width..=half_width)
            .map(|t| rc(t as Real, rolloff, sps))
            .collect();
        assert_eq!(taps.len(), num_taps);

        Self::new(taps)
    }

    pub fn process_sample(&mut self, sample: Real) -> Real {
        self.buffer[self.position] = sample;
        self.position = (self.position + 1) % self.buffer.len();
        self.buffer[self.position..]
            .iter()
            .chain(&self.buffer[..self.position])
            .zip(&self.taps[..])
            .map(|(&sample, &tap)| sample * tap)
            .sum()
    }

    pub fn process_inplace(&mut self, buffer: &mut [Real]) {
        for slot in buffer {
            *slot = self.process_sample(*slot);
        }
    }

    pub fn decimate(&mut self, buffer: &[Real]) -> Real {
        buffer
            .iter()
            .map(|&sample| self.process_sample(sample))
            .sum()
    }
}
