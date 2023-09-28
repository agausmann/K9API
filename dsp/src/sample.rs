use std::{iter::Sum, ops};

use crate::math::Real;

pub trait Sample:
    Copy
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Mul<Real, Output = Self>
    + ops::MulAssign<Real>
    + ops::Div<Real, Output = Self>
    + Sum
{
    const ZERO: Self;

    fn magnitude(&self) -> Real;
}
