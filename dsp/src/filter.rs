use crate::math::{cos, rc, sinc, Real, PI, TAU};

#[derive(Clone)]
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

    pub fn linear_interp(period: usize) -> Self {
        let half_width = period as isize - 1;
        let num_taps = half_width as usize * 2 + 1;
        let taps: Box<[Real]> = (-half_width..=half_width)
            .map(|t| 1.0 - (t.abs() as f32) / (period as f32))
            .collect();
        assert_eq!(taps.len(), num_taps);

        Self::new(taps)
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

#[derive(Debug, Clone, Copy)]
pub struct WindowMethod {
    pub gain: Real,
    pub sample_rate: Real,
    pub passband: Passband,
    pub transition_width: Option<Real>,
    pub num_taps: Option<usize>,
    pub window: Window,
}

impl WindowMethod {
    pub fn num_taps(&self) -> usize {
        let mut num_taps = self.num_taps.unwrap_or(0);
        if let Some(transition_width) = self.transition_width {
            num_taps = num_taps.max((4.0 * (self.sample_rate / transition_width)).ceil() as usize);
        }
        assert!(num_taps > 0);
        // Ensure num_taps is odd
        num_taps | 1
    }

    pub fn build(&self) -> Fir {
        let num_taps = self.num_taps();

        let taps: Box<[Real]> = (0..num_taps)
            .map(|x| {
                self.gain
                    * self
                        .passband
                        .sample(x as Real, num_taps as Real, self.sample_rate)
                    * self.window.sample(x as Real, num_taps as Real)
            })
            .collect();
        Fir::new(taps)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Passband {
    LowPass { cutoff: Real },
    HighPass { cutoff: Real },
    BandPass { low_cutoff: Real, high_cutoff: Real },
    BandReject { low_cutoff: Real, high_cutoff: Real },
}

impl Passband {
    pub fn centered_band_pass(center_freq: Real, bandwidth: Real) -> Self {
        Self::BandPass {
            low_cutoff: center_freq - 0.5 * bandwidth,
            high_cutoff: center_freq + 0.5 * bandwidth,
        }
    }

    pub fn centered_band_reject(center_freq: Real, bandwidth: Real) -> Self {
        Self::BandReject {
            low_cutoff: center_freq - 0.5 * bandwidth,
            high_cutoff: center_freq + 0.5 * bandwidth,
        }
    }

    pub fn sample(&self, x: Real, n: Real, sample_rate: Real) -> Real {
        let xn = x / n;
        let xc = 2.0 * xn - 1.0;

        match self {
            Self::LowPass { cutoff } => sinc(xc * sample_rate / cutoff),
            Self::HighPass { cutoff } => sinc(xc) - sinc(xc * sample_rate / cutoff),
            Self::BandPass {
                low_cutoff,
                high_cutoff,
            } => sinc(xc * sample_rate / high_cutoff) - sinc(xc * sample_rate / low_cutoff),
            Self::BandReject {
                low_cutoff,
                high_cutoff,
            } => {
                sinc(xc) - sinc(xc * sample_rate / high_cutoff)
                    + sinc(xc * sample_rate / low_cutoff)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Window {
    Rectangular,
    Bartlett,
    Welch,
    Gaussian { std_dev: Real },
    Tukey { param: Real },
    Kaiser { param: Real },
    Exponential { time_constant: Real },
    CosineSum2([Real; 2]),
    CosineSum3([Real; 3]),
    CosineSum4([Real; 4]),
    CosineSum5([Real; 5]),
}

impl Window {
    pub const HANN: Self = Self::CosineSum2([0.5, 0.5]);
    pub const HAMMING: Self = Self::CosineSum2([0.54, 0.46]);
    pub const BLACKMAN: Self = Self::CosineSum3([0.42, 0.5, 0.08]);
    pub const NUTTALL_CFD: Self = Self::CosineSum4([0.355768, 0.487396, 0.144232, 0.012604]);
    pub const BLACKMAN_NUTTALL: Self =
        Self::CosineSum4([0.3635819, 0.4891775, 0.1365995, 0.0106411]);
    pub const BLACKMAN_HARRIS: Self = Self::CosineSum4([0.35875, 0.48829, 0.14128, 0.01168]);
    pub const FLAT_TOP: Self = Self::CosineSum5([
        0.21557895,
        0.41663158,
        0.277263158,
        0.083578947,
        0.006947368,
    ]);

    pub fn sample(&self, x: Real, n: Real) -> Real {
        // "Normalized X", 0.0..1.0
        let xn = x / n;
        // "Centered X", -1.0..1.0
        let xc = 2.0 * xn - 1.0;

        match *self {
            Self::Rectangular => 1.0,
            Self::Bartlett => 1.0 - xc.abs(),
            Self::Welch => 1.0 - xc.powi(2),
            Self::Gaussian { std_dev } => (-0.5 * (xc / std_dev).powi(2)).exp(),
            Self::Tukey { param } => {
                if xc.abs() > param {
                    0.5 * (1.0 - cos(PI / param * xc))
                } else {
                    1.0
                }
            }
            Self::Kaiser { .. } => todo!(),
            Self::Exponential { time_constant } => (-(xn - 0.5).abs() / time_constant).exp(),
            Self::CosineSum2([a, b]) => a - b * cos(TAU * xn),
            Self::CosineSum3([a, b, c]) => a - b * cos(TAU * xn) + c * cos(2.0 * TAU * xn),
            Self::CosineSum4([a, b, c, d]) => {
                a - b * cos(TAU * xn) + c * cos(2.0 * TAU * xn) - d * cos(3.0 * TAU * xn)
            }
            Self::CosineSum5([a, b, c, d, e]) => {
                a - b * cos(TAU * xn) + c * cos(2.0 * TAU * xn) - d * cos(3.0 * TAU * xn)
                    + e * cos(4.0 * TAU * xn)
            }
        }
    }
}
