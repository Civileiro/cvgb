mod cartridge;
mod config;
mod context;
mod cpu;
mod events;
mod input;
mod system;
mod time;

pub use cartridge::{Cartridge, Rom};
pub use config::Config;
pub use input::Input;
pub use system::System;

pub const WINDOW_WIDTH: u8 = 160;
pub const WINDOW_HEIGHT: u8 = 144;
pub const WINDOW_ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
pub const REFRESH_RATE: f32 = 59.73;
