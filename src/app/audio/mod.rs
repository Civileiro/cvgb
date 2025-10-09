mod resampler;
mod stream;
mod sync;

use std::{
    fmt::Debug,
    sync::{Arc, atomic::AtomicU32},
};

use cpal::traits::{DeviceTrait, HostTrait};

use stream::AudioStream;
use sync::EmulationAudioSync;

use resampler::Resampler;
const CHUNK_SIZE: usize = 8192;
const NUM_CHANNELS: usize = 2;

/// The Audio Driver takes audio samples from the emulation and processes them
/// into something your system audio driver can accept.
/// It is synced with the emulation using VBLANK events to estimate how fast
/// the emulation is producing samples
pub struct AudioDriver {
    volume: Arc<AtomicU32>,
    gb_output: Option<game_boy::AudioOutput>,
    host: cpal::Host,
    device: cpal::Device,
    config_range: cpal::SupportedStreamConfigRange,
    stream: Option<AudioStream>,
    sync: EmulationAudioSync,
    resampler: Resampler,
}

impl Debug for AudioDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDriver")
            .field("volume", &self.volume)
            .field("host", &self.host.id().name())
            .field("device", &self.device.name())
            .field("config_range", &self.config_range)
            .finish()
    }
}

impl Default for AudioDriver {
    fn default() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config_range = device
            .supported_output_configs()
            .unwrap()
            .find(|config| {
                config.channels() == 2 && matches!(config.sample_format(), cpal::SampleFormat::F32)
            })
            .expect("system should support requested audio config");

        Self {
            volume: Arc::new(1.0_f32.to_bits().into()),
            gb_output: None,
            host,
            device,
            stream: None,
            config_range,
            sync: Default::default(),
            resampler: Resampler::new(NUM_CHANNELS, CHUNK_SIZE),
        }
    }
}

impl AudioDriver {
    /// Tell the Audio Driver that the emulation reached VBLANK
    pub fn signal_vblank(&mut self) {
        self.sync.signal_vblank();
        self.resample();
    }
    /// Tell the Audio Driver that the emulation has been paused
    pub fn signal_sleep(&mut self) {
        self.sync.signal_sleep();
    }
    fn resample(&mut self) {
        puffin::profile_function!();
        let gb_sample_rate = self.sync.estimate_emulation_sample_rate();
        if gb_sample_rate == 0 {
            log::info!("waiting for more audio samples");
            return;
        }
        let Some(gb_output) = self.gb_output.as_mut() else {
            log::warn!("Audio Driver doesnt have GB output");
            return;
        };
        let Some(stream) = self.stream.as_mut() else {
            log::warn!("Audio Driver doesnt have an output stream");
            return;
        };
        self.resampler.set_in_sample_rate(gb_sample_rate);
        while gb_output.curr_size() >= CHUNK_SIZE * NUM_CHANNELS {
            self.resampler.load_interleaved_iter(gb_output.dump_iter());
            self.resampler.resample();
            stream.add_samples(self.resampler.dump_interleaved_iter());
        }
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.volume.store(
            volume.clamp(0.0, 1.0).to_bits(),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        if self
            .stream
            .as_ref()
            .is_none_or(|s| s.sample_rate() != sample_rate)
        {
            self.restart_stream(sample_rate);
        }
        self.resampler.set_out_sample_rate(sample_rate);
    }
    fn restart_stream(&mut self, sample_rate: u32) {
        let config = self
            .config_range
            .with_sample_rate(cpal::SampleRate(sample_rate));
        let stats = self.sync.get_stream_stats_collector();
        let stream = AudioStream::new(&self.device, config.into(), stats);
        self.stream = Some(stream);
    }
    pub fn set_gb_output(&mut self, gb_output: game_boy::AudioOutput) {
        self.gb_output = Some(gb_output)
    }
}
