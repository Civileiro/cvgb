use std::ops::Index;

use rubato::VecResampler;

pub struct Resampler {
    inner_resampler: rubato::SincFixedIn<f32>,
    in_buffer: Vec<Vec<f32>>,
    out_buffer: Vec<Vec<f32>>,
    out_buffer_size: usize,
    chunk_size: usize,
    in_sample_rate: u32,
    out_sample_rate: u32,
}

const DEFAULT_SAMPLE_RATE: u32 = 44_100;

impl Resampler {
    pub fn new(num_channels: usize, chunk_size: usize) -> Self {
        let in_sample_rate = game_boy::APU_SAMPLE_RATE as u32;
        let out_sample_rate = DEFAULT_SAMPLE_RATE;
        let resampler = rubato::SincFixedIn::new(
            out_sample_rate as f64 / in_sample_rate as f64,
            10.0,
            rubato::SincInterpolationParameters {
                sinc_len: 256,
                f_cutoff: 0.95,
                oversampling_factor: 128,
                interpolation: rubato::SincInterpolationType::Linear,
                window: rubato::WindowFunction::Blackman,
            },
            chunk_size,
            num_channels,
        )
        .unwrap();
        Self {
            in_buffer: resampler.input_buffer_allocate(false),
            in_sample_rate,
            out_buffer: resampler.output_buffer_allocate(true),
            out_buffer_size: 0,
            out_sample_rate,
            inner_resampler: resampler,
            chunk_size,
        }
    }
    pub fn set_in_sample_rate(&mut self, sample_rate: u32) {
        self.in_sample_rate = sample_rate;
        self.set_resample_ratio();
    }
    pub fn set_out_sample_rate(&mut self, sample_rate: u32) {
        self.out_sample_rate = sample_rate;
        self.set_resample_ratio();
    }
    pub fn set_resample_ratio(&mut self) {
        self.inner_resampler
            .set_resample_ratio(
                self.out_sample_rate as f64 / self.in_sample_rate as f64,
                false,
            )
            .unwrap()
    }
    pub fn load_interleaved_iter(&mut self, iter: impl Iterator<Item = f32>) {
        for channel in &mut self.in_buffer {
            channel.clear()
        }
        let mut channel = 0;
        for sample in iter.take(self.chunk_size * self.inner_resampler.nbr_channels()) {
            self.in_buffer[channel].push(sample);
            channel += 1;
            channel %= self.inner_resampler.nbr_channels();
        }
        for channel in &self.in_buffer {
            debug_assert_eq!(channel.len(), self.chunk_size);
        }
    }
    pub fn resample(&mut self) {
        let (consumed, produced) = self
            .inner_resampler
            .process_into_buffer(&self.in_buffer, &mut self.out_buffer, None)
            .unwrap();
        log::debug!("resampled {consumed} frames -> {produced} frames");
        self.out_buffer_size = produced;
    }
    pub fn dump_interleaved_iter(&self) -> impl Iterator<Item = f32> {
        InterleavedBufferIterator::new(&self.out_buffer)
            .copied()
            .take(self.out_buffer_size * self.inner_resampler.nbr_channels())
    }
}

pub struct InterleavedBufferIterator<'a, T, I: AsRef<[T]>> {
    buffer: &'a [I],
    curr_channel: usize,
    max_channel: usize,
    curr_frame: usize,
    max_frame: usize,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T, I: AsRef<[T]>> InterleavedBufferIterator<'a, T, I> {
    fn new(buffer: &'a [I]) -> Self {
        Self {
            buffer,
            curr_channel: 0,
            max_channel: buffer.len(),
            curr_frame: 0,
            max_frame: buffer[0].as_ref().len(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T: 'a, I: AsRef<[T]>> Iterator for InterleavedBufferIterator<'a, T, I> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_frame == self.max_frame {
            return None;
        }
        let res = self.buffer[self.curr_channel]
            .as_ref()
            .index(self.curr_frame);
        self.curr_channel += 1;
        if self.curr_channel == self.max_channel {
            self.curr_channel = 0;
            self.curr_frame += 1;
        }
        Some(res)
    }
}

impl<'a, T: 'a, I: AsRef<[T]>> ExactSizeIterator for InterleavedBufferIterator<'a, T, I> {
    fn len(&self) -> usize {
        let leftover_frames = self.max_frame - self.curr_frame;
        leftover_frames * self.max_channel - self.curr_channel
    }
}
