use crate::math::Real;

/// A utility for buffering the output of a sample generator.
///
/// This is useful when the output buffer requirements of a generator do not
/// match the rate that samples are consumed. It allows the user to consume
/// any number of bytes at any time, and will call the provided generator
/// function when more samples are required.
pub struct Buffer<G> {
    generator: G,
    buffer: Box<[Real]>,
    buffer_size: usize,
    chunk_size: usize,
    position: usize,
    available: usize,
}

impl<G> Buffer<G>
where
    G: FnMut(&mut [Real]),
{
    /// Construct an empty buffer.
    ///
    /// The provided generator function will be called whenever more samples
    /// are required. It will always be provided with a buffer of size
    /// `chunk_size`.
    ///
    /// This internally heap-allocates a buffer large enough to hold
    /// `buffer_size + chunk_size` samples. (Worst case is when there are
    /// `buffer_size - 1` available samples in the buffer, and
    /// `fill_buffer(buffer_size)` is called, requiring another `chunk_size`
    /// samples to be appended to the buffer to meet the requested size.
    pub fn new(generator: G, buffer_size: usize, chunk_size: usize) -> Self {
        Self {
            generator,
            buffer: vec![0.0; 2 * chunk_size].into_boxed_slice(),
            buffer_size,
            chunk_size,
            position: 0,
            available: 0,
        }
    }

    /// The "buffer size" of this buffer; the maximum size that can be
    /// requested with `fill_buffer`.
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// The "chunk size" of this buffer; how many samples it retrieves
    /// for each call to the generator function.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// The samples that are currently waiting to be consumed.
    pub fn available(&self) -> &[Real] {
        &self.buffer[self.position..][..self.available]
    }

    /// Ensure at least `num_samples` are available in the buffer.
    ///
    /// After this function returns, `available().len()` will be greater than
    /// or equal to `num_samples`.
    ///
    /// # Requirements
    ///
    /// - `num_samples` must be less than or equal to `buffer_size.
    pub fn fill_buffer(&mut self, num_samples: usize) {
        if self.available >= num_samples {
            return;
        }

        self.buffer
            .copy_within(self.position..(self.position + self.available), 0);
        self.position = 0;

        while self.available < num_samples {
            (self.generator)(&mut self.buffer[self.available..][..self.chunk_size]);
            self.available += self.chunk_size;
        }
    }

    /// Consume some samples from the buffer.
    ///
    /// The first `num_samples` samples in the buffer will be discarded,
    /// effectively decreaasing the number of available samples by `num_samples`.
    ///
    /// # Requirements
    /// - `num_samples` must be less than or equal to `available().len()`
    pub fn consume(&mut self, num_samples: usize) {
        self.available -= num_samples;
        self.position += num_samples;
    }
}
