use cpal::{BufferSize, traits::DeviceTrait};
use ringbuf::traits::{Consumer, Observer, Producer, Split};

use crate::app::audio::sync::{StreamStats, StreamStatsCollector};

pub struct AudioStream {
    buffer_producer: ringbuf::HeapProd<f32>,
    config: cpal::StreamConfig,
    stream: cpal::Stream,
}

impl AudioStream {
    pub fn new(
        device: &cpal::Device,
        config: cpal::StreamConfig,
        mut stats: StreamStatsCollector,
    ) -> Self {
        // TODO: configurable buffer size
        let rb = ringbuf::HeapRb::new(10000);
        let (prod, mut cons) = rb.split();
        let stream_config = config.clone();
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _info| {
                    let data_available = cons.occupied_len();
                    if data_available == 0 {
                        data.fill(0.0);
                    } else {
                        stats.push(StreamStats {
                            buffer_ratio: data_available as f32 / data.len() as f32,
                        });
                        cons.pop_slice(data);
                    }
                },
                move |err| log::error!("audio stream error: {err}"),
                None,
            )
            .unwrap();

        Self {
            buffer_producer: prod,
            config,
            stream,
        }
    }
    pub fn add_samples(&mut self, samples: impl Iterator<Item = f32>) {
        self.buffer_producer.push_iter(samples);
    }
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}
