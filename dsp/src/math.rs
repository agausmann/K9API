use std::f32::consts::{PI, SQRT_2};

/// `sin(PI * x) / (PI * x)`
pub fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        f32::sin(PI * x) / (PI * x)
    }
}

/// Impulse response of a raised cosine filter.
pub fn rc(t: f32, rolloff: f32, sps: f32) -> f32 {
    let tn = t / sps;
    let d = 2.0 * rolloff * tn;
    if d.abs() == 1.0 {
        PI / (4.0 * sps) * sinc(1.0 / (2.0 * rolloff))
    } else {
        sinc(tn) * f32::cos(PI * rolloff * tn) / (sps * (1.0 - d * d))
    }
}

/// Impulse response of a root-raised-cosine filter.
pub fn rrc(t: f32, rolloff: f32, sps: f32) -> f32 {
    if t == 0.0 {
        return (1.0 + rolloff * (4.0 / PI - 1.0)) / sps;
    }

    let tn = t / sps;
    let d = 4.0 * rolloff * tn;
    if d == 1.0 {
        rolloff / (sps * SQRT_2)
            * ((1.0 + 2.0 / PI) * f32::sin(PI / (4.0 * rolloff))
                + (1.0 - 2.0 / PI) * f32::cos(PI / (4.0 * rolloff)))
    } else {
        (f32::sin(PI * tn * (1.0 - rolloff)) + d * f32::cos(PI * tn * (1.0 + rolloff)))
            / (PI * t * (1.0 - d * d))
    }
}
