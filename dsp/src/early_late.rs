//! Early-late timing recovery and resampling

use crate::math::Real;
use crate::sample::Sample;

pub struct EarlyLate<T = Real> {
    window: [T; 5],
    tolerance: Real,
    decimation_factor: usize,
    position: usize,
}

impl<T: Sample> EarlyLate<T> {
    pub fn new(tolerance: Real, decimation_factor: usize) -> Self {
        // Need a decently large oversampling for this method to work.
        assert!(decimation_factor >= 5);

        Self {
            window: [T::ZERO; 5],
            tolerance,
            decimation_factor,
            position: decimation_factor,
        }
    }

    pub fn process(&mut self, sample: T) -> Option<T> {
        self.window.copy_within(1.., 0);
        *self.window.last_mut().unwrap() = sample;
        self.position = self.position.saturating_sub(1);

        if self.position != 0 {
            // Don't output a sample yet.
            return None;
        }

        // Otherwise, output a sample and reset timing.
        let early = self.window.first().unwrap().magnitude();
        let late = self.window.last().unwrap().magnitude();

        self.position = if (early - late).abs() <= self.tolerance {
            // On time
            self.decimation_factor
        } else if early > late {
            // Late, take next sample earlier
            self.decimation_factor - 1
        } else {
            // Early, take next sample later
            self.decimation_factor + 1
        };

        // Output = center of window
        Some(self.window[2])
    }
}
