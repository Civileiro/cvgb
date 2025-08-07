use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::game_boy;

use super::windows::WindowRegistry;

#[derive(Debug, Default)]
pub struct AppState {
    pub app_config: super::Config,
    pub game_state: GameState,
    emulation_state: Option<game_boy::System>,

    pub window_registry: WindowRegistry,
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
}
