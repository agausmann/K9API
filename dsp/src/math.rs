pub type Real = f32;

#[doc(inline)]
pub use std::f32::consts::*;

/// Wrapper around `Real::sin`.
///
/// This allows the sine function to be imported and written as `sin(PI)`
/// instead of `Real::sin(PI)` or `PI.sin()`, which is the syntax I prefer.
pub fn sin(x: Real) -> Real {
    x.sin()
}

/// Wrapper around `Real::cos`.
///
/// This allows the cosine function to be imported and written as `cos(PI)`
/// instead of `Real::cos(PI)` or `PI.cos()`, which is the syntax I prefer.
pub fn cos(x: Real) -> Real {
    x.cos()
}

/// `sin(PI * x) / (PI * x)` but continuous.
pub fn sinc(x: Real) -> Real {
    if x == 0.0 {
        1.0
    } else {
        sin(PI * x) / (PI * x)
    }
}

/// Impulse response of a raised cosine filter.
pub fn rc(t: Real, rolloff: Real, sps: Real) -> Real {
    let tn = t / sps;
    let d = 2.0 * rolloff * tn;
    if d.abs() == 1.0 {
        PI / (4.0 * sps) * sinc(1.0 / (2.0 * rolloff))
    } else {
        sinc(tn) * Real::cos(PI * rolloff * tn) / (sps * (1.0 - d * d))
    }
}

/// Impulse response of a root-raised-cosine filter.
pub fn rrc(t: Real, rolloff: Real, sps: Real) -> Real {
    if t == 0.0 {
        return (1.0 + rolloff * (4.0 / PI - 1.0)) / sps;
    }

    let tn = t / sps;
    let d = 4.0 * rolloff * tn;
    if d == 1.0 {
        rolloff / (sps * SQRT_2)
            * ((1.0 + 2.0 / PI) * Real::sin(PI / (4.0 * rolloff))
                + (1.0 - 2.0 / PI) * Real::cos(PI / (4.0 * rolloff)))
    } else {
        (Real::sin(PI * tn * (1.0 - rolloff)) + d * Real::cos(PI * tn * (1.0 + rolloff)))
            / (PI * t * (1.0 - d * d))
    }
}
