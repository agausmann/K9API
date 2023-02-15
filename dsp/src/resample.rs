use crate::filter::Fir;
use crate::Sample;

pub struct Upsample {
    factor: usize,
    filter: Fir,
}

impl Upsample {
    pub fn new(factor: usize, filter: Fir) -> Self {
        Self { factor, filter }
    }

    pub fn process(&mut self, input: &[Sample], output: &mut [Sample]) {
        assert_eq!(input.len() * self.factor, output.len());
        output.fill(0.0);
        for (out, inp) in output.chunks_mut(self.factor).zip(input) {
            out[0] = *inp;
            self.filter.process_inplace(out);
        }
    }
}

pub struct Downsample {
    factor: usize,
    filter: Fir,
}

impl Downsample {
    pub fn new(factor: usize, filter: Fir) -> Self {
        Self { factor, filter }
    }

    pub fn process(&mut self, input: &[Sample], output: &mut [Sample]) {
        assert_eq!(input.len(), output.len() * self.factor);
        for (inp, out) in input.chunks(self.factor).zip(output) {
            *out = self.filter.decimate(inp);
        }
    }
}
