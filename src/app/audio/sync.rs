use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use egui::mutex::Mutex;
use ringbuf::traits::{Consumer, Observer, Producer, RingBuffer, Split};

use super::game_boy;

const SAMPLES_PER_VBLANK: u32 = game_boy::CLOCKS_PER_FRAME as u32 / 4;

type StatsBuffer = Arc<Mutex<ringbuf::StaticRb<StreamStats, 10>>>;

pub struct EmulationAudioSync {
    is_sleeping: bool,
    deltas: ringbuf::StaticRb<Duration, 10>,
    last_vblank: Option<Instant>,
    stats: StatsBuffer,
}

impl Default for EmulationAudioSync {
    fn default() -> Self {
        Self {
            is_sleeping: true,
            deltas: Default::default(),
            last_vblank: None,
            stats: Default::default(),
        }
    }
}

impl EmulationAudioSync {
    pub fn signal_vblank(&mut self) {
        if !self.is_sleeping
            && let Some(last) = self.last_vblank
        {
            let delta = last.elapsed();
            self.deltas.push_overwrite(delta);
        }
        self.is_sleeping = false;
        self.last_vblank = Some(Instant::now())
    }
    pub fn signal_sleep(&mut self) {
        self.is_sleeping = true
    }
    pub fn estimate_emulation_sample_rate(&self) -> u32 {
        let total_delta: f32 = self.deltas.iter().map(|d| d.as_secs_f32()).sum();
        let num_deltas = self.deltas.occupied_len();
        let avg_seconds_per_vblank = total_delta / num_deltas as f32;
        let avg_sample_rate = SAMPLES_PER_VBLANK as f32 / avg_seconds_per_vblank;
        (avg_sample_rate * self.get_adjustment()) as u32
    }
    /// A minor dynamic adjustment is needed to avoid the buildup/shortage of
    /// samples over time
    fn get_adjustment(&self) -> f32 {
        let ratios: Vec<_> = {
            let stats = self.stats.lock();
            stats.iter().map(|s| s.buffer_ratio).collect()
        };
        if ratios.is_empty() {
            return 1.0;
        }

        let min = ratios.iter().copied().reduce(f32::min).unwrap();
        // the target is for the minimum ratio to be a bit over 1.0
        let target = 1.05;
        // diff > 0.0 -> adjustment < 0.0
        // diff < 0.0 -> adjustment > 0.0
        let diff = target - min;
        1.0 - diff / 100.0
    }
    pub fn get_stream_stats_collector(&mut self) -> StreamStatsCollector {
        StreamStatsCollector(self.stats.clone())
    }
}

#[derive(Debug)]
pub struct StreamStats {
    pub buffer_ratio: f32,
}

pub struct StreamStatsCollector(StatsBuffer);

impl StreamStatsCollector {
    pub fn push(&mut self, stats: StreamStats) {
        self.0.lock().push_overwrite(stats);
    }
}
