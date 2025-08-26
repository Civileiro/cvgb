use std::{cell::RefCell, fmt::Debug, rc::Rc, time::Duration};

use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::game_boy;

use super::{game_renderer::GameRenderingType, tasks::TaskManager, windows::WindowRegistry};

#[derive(Default)]
pub struct AppState {
    pub app_config: super::Config,
    pub game_state: GameState,
    pub emulation_state: Option<game_boy::System>,

    pub window_registry: WindowRegistry,
    pub task_manager: TaskManager,
    pub init_rom_file: Rc<RefCell<Option<Box<[u8]>>>>,
    pub new_game_frame_requested: bool,
    pub game_rendering_type: GameRenderingType,
}

/// Stores all the information about the currently-running game
#[derive(Debug, Default)]
pub struct GameState {
    pub gameboy_config: game_boy::Config,
}

impl AppState {
    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        if event.repeat {
            return;
        }
        if let PhysicalKey::Code(code) = event.physical_key {
            // TODO: remappable keys
            if let Some(input) = match code {
                KeyCode::ArrowRight => Some(game_boy::Input::RIGHT),
                KeyCode::ArrowLeft => Some(game_boy::Input::LEFT),
                KeyCode::ArrowUp => Some(game_boy::Input::UP),
                KeyCode::ArrowDown => Some(game_boy::Input::DOWN),
                KeyCode::KeyX => Some(game_boy::Input::A),
                KeyCode::KeyZ => Some(game_boy::Input::B),
                KeyCode::KeyA => Some(game_boy::Input::SELECT),
                KeyCode::KeyS => Some(game_boy::Input::START),
                _ => None,
            } && let Some(system) = self.emulation_state.as_mut()
            {
                if event.state.is_pressed() {
                    system.press_key(input);
                } else {
                    system.unpress_key(input);
                }
            }
        }
    }
    pub fn advance_pre_render(&mut self, delta_time: Duration) {
        self.task_manager.poll();

        if let Some(rom_file) = self
            .init_rom_file
            .try_borrow_mut()
            .ok()
            .and_then(|mut rfb| rfb.take())
        {
            match game_boy::System::new(rom_file) {
                Ok(emu) => self.emulation_state = Some(emu),
                Err(err) => log::warn!("Couldn't start emulation from file: {err}"),
            }
        }
        if let Some(emu) = self.emulation_state.as_mut() {
            let mut frame_duration = game_boy::SystemTime::from_seconds(delta_time.as_secs_f64());
            loop {
                let (events, elapsed_time) = emu.advance(frame_duration);
                if events.has_vblank() {
                    log::debug!("VBlank detected, signaling game render");
                    self.new_game_frame_requested = true;
                    break;
                }
                if elapsed_time > frame_duration {
                    self.new_game_frame_requested = true;
                    break;
                } else {
                    frame_duration -= elapsed_time
                }
            }
        }
    }
    pub fn advance_post_render(&mut self) {}
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("app_config", &self.app_config)
            .field("game_state", &self.game_state)
            .field("emulation_state", &self.emulation_state)
            .field("window_registry", &self.window_registry)
            .finish()
    }
}
