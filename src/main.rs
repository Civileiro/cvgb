#![allow(dead_code)]

mod app;
mod core;

use app::CvgbApp;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

fn main() -> Result<(), EventLoopError> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // Don't wait between loop iterations
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = CvgbApp::default();
    event_loop.run_app(&mut app)
}
