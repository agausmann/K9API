use crate::amplify;
use crate::filter::Fir;
use crate::math::Real;
use crate::sample::Sample;

pub struct Upsample<T = Real> {
    factor: usize,
    filter: Fir<T>,
}

impl<T: Sample> Upsample<T> {
    pub fn new(factor: usize, filter: Fir<T>) -> Self {
        Self { factor, filter }
    }

    pub fn process(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(input.len() * self.factor, output.len());
        output.fill(T::ZERO);
        for (out, inp) in output.chunks_mut(self.factor).zip(input) {
            out[0] = *inp;
            self.filter.process_inplace(out);
        }
        amplify(self.factor as Real, output);
    }
}

pub struct Downsample<T = Real> {
    factor: usize,
    filter: Fir<T>,
}

impl<T: Sample> Downsample<T> {
    pub fn new(factor: usize, filter: Fir<T>) -> Self {
        Self { factor, filter }
    }

    pub fn process(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(input.len(), output.len() * self.factor);
        for (inp, out) in input.chunks(self.factor).zip(output) {
            *out = self.filter.decimate(inp);
        }
    }
}
