// Most things here are placeholders

use std::fmt::Debug;

#[derive(Default)]
pub struct VideoBuffer {}

#[derive(Default)]
pub struct Display {
    buffer: VideoBuffer,
}

impl Display {
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        &self.buffer
    }
}

#[derive(Default)]
pub struct Ppu {
    display: Display,
}

impl Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ppu").finish()
    }
}

impl Ppu {
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        self.display.get_video_buffer()
    }
}
