mod boot_rom;
mod cartridge;
mod config;
mod context;
mod cpu;
mod events;
mod system;
mod time;

pub use boot_rom::BootRom;
pub use cartridge::{Cartridge, Rom};
pub use config::Config;
pub use context::{AudioOutput, Input, Palette};
pub use cpu::{Cpu, opcode::Opcode};
pub use events::Events;
pub use system::System;
pub use time::SystemTime;

pub const WINDOW_WIDTH: u8 = 160;
pub const WINDOW_HEIGHT: u8 = 144;
pub const WINDOW_ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
pub const CLOCKS_PER_FRAME: usize = 70_224;
pub const REFRESH_RATE: f32 = BASE_CPU_FREQUENCY as f32 / CLOCKS_PER_FRAME as f32;
pub const BASE_CPU_FREQUENCY: usize = 4_194_304;
pub const APU_SAMPLE_RATE: usize = BASE_CPU_FREQUENCY / 4;
