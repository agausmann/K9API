use std::{iter::Sum, ops};

use crate::{math::Real, sample::Sample};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IQ {
    pub i: Real,
    pub q: Real,
}

impl IQ {
    pub fn new(i: Real, q: Real) -> Self {
        Self { i, q }
    }

    pub fn new_polar(phase: Real, magnitude: Real) -> Self {
        Self {
            i: magnitude * phase.cos(),
            q: magnitude * phase.sin(),
        }
    }

    pub fn phase(&self) -> Real {
        Real::atan2(self.q, self.i)
    }
}

impl ops::Add for IQ {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i + rhs.i,
            q: self.q + rhs.q,
        }
    }
}

impl ops::Sub for IQ {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i - rhs.i,
            q: self.q - rhs.q,
        }
    }
}

impl ops::Mul<Real> for IQ {
    type Output = Self;

    fn mul(self, rhs: Real) -> Self::Output {
        Self {
            i: self.i * rhs,
            q: self.q * rhs,
        }
    }
}

impl ops::MulAssign<Real> for IQ {
    fn mul_assign(&mut self, rhs: Real) {
        self.i *= rhs;
        self.q *= rhs;
    }
}

impl ops::Div<Real> for IQ {
    type Output = Self;

    fn div(self, rhs: Real) -> Self::Output {
        Self {
            i: self.i / rhs,
            q: self.q / rhs,
        }
    }
}

impl Sum for IQ {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(IQ::ZERO, |a, b| a + b)
    }
}

impl Sample for IQ {
    const ZERO: Self = Self { i: 0.0, q: 0.0 };

    fn magnitude_squared(&self) -> Real {
        (self.i * self.i) + (self.q * self.q)
    }

    fn magnitude(&self) -> Real {
        self.magnitude_squared().sqrt()
    }
}
