use crate::{
    iq::IQ,
    math::{Real, TAU},
};

/// Local oscillator outputting IQ samples.
pub struct Oscillator {
    phase: Real,
    period: Real,
}

impl Oscillator {
    /// Construct an oscillator with a given period and starting phase.
    ///
    /// Both the period and phase should be given in terms of the number of samples.
    ///
    /// # Examples
    ///
    /// An oscillator with a period of 16 samples, starting 90 degrees (1/4 wave)
    /// ahead of 1:
    ///
    /// ```
    /// let osc = Oscillator::new(16.0, 4.0);
    /// ````
    pub fn new(period: f32, starting_phase: f32) -> Self {
        Self {
            period,
            phase: starting_phase,
        }
    }

    /// Generate the next sample of this iscukkatir.
    ///
    /// This automatically increments the internal state; the next call to
    /// `next()` will produce the next sample in succession.
    pub fn next(&mut self) -> IQ {
        self.next_with_offset(0.0)
    }

    pub fn next_with_offset(&mut self, phase_offset_angle: Real) -> IQ {
        let sample = IQ::new_polar(self.phase_angle() + phase_offset_angle, 1.0);
        self.phase = (self.phase + 1.0) % self.period;
        sample
    }

    /// Fill the provided buffer with the next samples of this wave.
    ///
    /// This automatically increments the internal state by the length of the
    /// provided buffer, same as calling `next()` as many times as the buffer
    /// length.
    pub fn fill(&mut self, buffer: &mut [IQ]) {
        for slot in buffer {
            *slot = self.next();
        }
    }

    /// The phase of the next sample, represented as a value between 0 and `period`.
    pub fn phase(&self) -> Real {
        self.phase
    }

    /// Modify the phase of the next sample.
    ///
    /// In this context, phase is represented as a value between 0 and `period`.
    pub fn set_phase(&mut self, phase: Real) {
        self.phase = phase % self.period;
    }

    /// Scale factor from the internal phase value to the actual "phase angle"
    /// in radians.
    ///
    /// ```text
    /// phase_angle = osc.phase() * osc.phase_scale()
    /// ```
    ///
    /// For (admittedly dubious) accuracy reasons, the phase is stored as a
    /// value between 0 and `period`, and is manipulated in that format until it
    /// needs to be converted to radians to generate a sample.
    ///
    /// (This scheme should provide good stability for periods that are integer
    /// or some fractional number of samples)
    pub fn phase_scale(&self) -> Real {
        TAU / self.period
    }

    pub fn phase_angle(&self) -> Real {
        self.phase * self.phase_scale()
    }
}

/// Sine wave generator.
pub struct Sine {
    osc: Oscillator,
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
            osc: Oscillator::new(period, starting_phase),
        }
    }

    /// Generate the next sample of this wave.
    ///
    /// This automatically increments the internal state; the next call to
    /// `next()` will produce the next sample in succession.
    pub fn next(&mut self) -> Real {
        self.osc.next().q
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
