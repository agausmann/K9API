use crate::math::{sin, Real, TAU};

/// Sine wave generator.
pub struct Sine {
    phase: Real,
    period: Real,
}

impl Sine {
    /// Construct a sine wave with a given period and starting phase.
    ///
    /// Both the period and phase should be given in terms of the number of samples.
    ///
    /// # Examples
    ///
    /// A sine wave with a period of 16 samples, starting 90 degrees (1/4 wave)
    /// ahead of zero:
    ///
    /// ```
    /// let sine = Sine::new(16.0, 4.0);
    /// ````
    pub fn new(period: f32, starting_phase: f32) -> Self {
        Self {
            period,
            phase: starting_phase,
        }
    }

    /// Generate the next sample of this wave.
    ///
    /// This automatically increments the internal state; the next call to
    /// `next()` will produce the next sample in succession.
    pub fn next(&mut self) -> Real {
        let sample = sin(self.phase / self.period * TAU);
        self.phase = (self.phase + 1.0) % self.period;
        sample
    }

    /// Fill the provided buffer with the next samples of this wave.
    ///
    /// This automatically increments the internal state by the length of the
    /// provided buffer, same as calling `next()` as many times as the buffer
    /// length.
    pub fn fill(&mut self, buffer: &mut [Real]) {
        for slot in buffer {
            *slot = self.next();
        }
    }
}
