#![allow(dead_code)]

mod app;
mod game_boy;

use app::CvgbApp;
use winit::{error::EventLoopError, event_loop::EventLoop};

fn main() -> Result<(), EventLoopError> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = CvgbApp::default();
    event_loop.run_app(&mut app)
}
