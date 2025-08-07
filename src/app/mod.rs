mod config;
mod gui_renderer;
mod renderer;
mod state;
mod timing;
mod ui;
mod windows;

pub use config::Config;
use log::log;
use renderer::RendererState;
use windows::AppWindow;

use std::{sync::Arc, time::Duration};

use state::AppState;
use timing::FrameTiming;
use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};

use crate::game_boy;

#[derive(Debug)]
pub struct CvgbApp {
    renderer_state: Option<RendererState>,
    timing: FrameTiming,
    state: AppState,
}

impl Default for CvgbApp {
    fn default() -> Self {
        let target_fps = game_boy::REFRESH_RATE;
        let frame_duration = Duration::from_secs_f32(1.0 / target_fps);
        Self {
            renderer_state: None,
            timing: FrameTiming::new(frame_duration),
            // TODO: Load config from file
            state: AppState::default(),
        }
    }
}

impl CvgbApp {
    fn toggle_window(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        app_window: AppWindow,
    ) {
        log::info!("toggling {app_window:?}");
        let Some(render_state) = self.renderer_state.as_mut() else {
            return;
        };
        if let Some(window_id) = self
            .state
            .window_registry
            .unregister_by_app_window(app_window)
        {
            log::info!("Closing window {window_id:?}");
            render_state.unregister_window(window_id);
        } else {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            log::info!("Opening window {:?}", window.id());
            self.state
                .window_registry
                .register_window(window.id(), app_window);
            render_state.register_window(window, app_window.layout());
        }
    }
    fn request_redraw(&self) {
        if let Some(window_id) = self.state.window_registry.get_id(AppWindow::MainWindow)
            && let Some(render_state) = self.renderer_state.as_ref()
        {
            let window = render_state.get_window(window_id);
            window.request_redraw();
        }
    }
    fn set_next_wait_time(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.timing.next_frame_start_time(),
        ));
    }
}

impl ApplicationHandler for CvgbApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let state = pollster::block_on(RendererState::new());
        self.renderer_state = Some(state);
        self.toggle_window(event_loop, AppWindow::MainWindow);

        self.request_redraw();
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached {
            start: _,
            requested_resume: _,
        } = cause
        {
            let render_state = self.renderer_state.as_mut().unwrap();
            if let Some(main_window_id) = self.state.window_registry.get_id(AppWindow::MainWindow) {
                let main_window = render_state.get_window(main_window_id);
                main_window.request_redraw();
                self.set_next_wait_time(event_loop);
            } else {
                log::error!("Main window doesnt exist: Exiting App");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let render_state = self.renderer_state.as_mut().unwrap();

        // The GUI has first dibs on events
        // we ignore gui repaint requests because our app has a constant refresh rate
        let gui_event_response = render_state.handle_gui_input(window_id, &event);

        if gui_event_response.map(|r| r.consumed).unwrap_or(false) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                if let Some(app_window) = self.state.window_registry.get_app_window(window_id) {
                    if app_window.is_main() {
                        log::info!("Close requested on main window: Exiting app");
                        event_loop.exit();
                    } else {
                        self.toggle_window(event_loop, app_window);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // We only listen to main screen redraw requests and redraw
                // every screen at the same time
                if !self
                    .state
                    .window_registry
                    .get_app_window(window_id)
                    .map(|app_window| app_window.is_main())
                    .unwrap_or(false)
                {
                    return;
                }
                render_state.render(&mut self.state);
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.timing.next_frame_start_time(),
                ));
            }
            WindowEvent::Resized(size) => {
                // No need to re-render as this event is
                // always followed by a redraw event
                render_state.resize(window_id, size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.handle_key_event(&event);
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if code == winit::keyboard::KeyCode::F3
                        && event.state.is_pressed()
                        && !event.repeat
                    {
                        self.toggle_window(event_loop, AppWindow::OptionsWindow);
                    }
                }
            }

            _ => (),
        }
    }
}
