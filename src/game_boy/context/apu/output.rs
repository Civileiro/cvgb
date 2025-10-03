use std::fmt::Debug;

use ringbuf::{
    consumer::PopIter,
    traits::{Consumer, Observer, Producer, Split},
};

#[derive(Default)]
pub struct AudioBuffer {
    output: Option<ringbuf::HeapProd<f32>>,
}

impl Debug for AudioBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutput").finish()
    }
}

impl AudioBuffer {
    pub fn get_output(&mut self) -> AudioOutput {
        let rb = ringbuf::HeapRb::new(1_000_000);
        let (prod, cons) = rb.split();
        self.output = Some(prod);
        AudioOutput {
            buffer_consumer: cons,
        }
    }
    pub fn add_sample(&mut self, sample: [f32; 2]) {
        if let Some(output) = self.output.as_mut() {
            let num_pushed = output.push_slice(&sample);
            debug_assert_eq!(num_pushed, 2);
        }
    }
}

pub struct AudioOutput {
    buffer_consumer: ringbuf::HeapCons<f32>,
}

impl Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutput")
            .field("buffer_size", &self.curr_size())
            .finish()
    }
}

impl AudioOutput {
    pub fn dump_audio(&mut self, buffer: &mut [f32]) {
        self.buffer_consumer.pop_slice(buffer);
    }
    pub fn dump_iter(&'_ mut self) -> PopIter<'_, ringbuf::HeapCons<f32>> {
        self.buffer_consumer.pop_iter()
    }
    pub fn curr_size(&self) -> usize {
        self.buffer_consumer.occupied_len()
    }
}
