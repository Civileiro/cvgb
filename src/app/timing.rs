use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct FrameTiming {
    frame_duration: Duration,
    target_frame_time: Instant,
}

impl FrameTiming {
    pub fn new(target_frame_duration: Duration) -> Self {
        let now = Instant::now();
        Self {
            frame_duration: target_frame_duration,
            target_frame_time: now + target_frame_duration,
        }
    }
    pub fn next_frame_start_time(&mut self) -> Instant {
        let now = Instant::now();
        if now < self.target_frame_time {
            return self.target_frame_time;
        }
        self.target_frame_time += self.frame_duration;
        self.target_frame_time = self.target_frame_time.max(now);
        self.target_frame_time
    }
}
