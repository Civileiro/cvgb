use std::{cell::RefCell, rc::Rc, time::Duration};

use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::game_boy;

use super::{
    audio::AudioDriver, game_renderer::GameRenderingType, tasks::TaskManager,
    ui::options_ui::OptionsUiState, windows::WindowRegistry,
};

#[derive(Debug, Default)]
pub struct AppState {
    pub app_config: super::Config,
    pub game_state: GameState,
    pub emulation_state: Option<game_boy::System>,

    pub window_registry: WindowRegistry,
    pub task_manager: TaskManager,
    pub audio_driver: AudioDriver,

    pub options_ui_state: OptionsUiState,
}

/// Stores all the information about the currently-running game
#[derive(Debug, Default)]
pub struct GameState {
    pub gameboy_config: game_boy::Config,

    pub init_rom_file: Rc<RefCell<Option<Box<[u8]>>>>,
    pub new_game_frame_requested: bool,
    pub game_rendering_type: GameRenderingType,
    pub paused: bool,
    pub breakpoint_addr: Option<u16>,
}

impl AppState {
    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        if event.repeat {
            return;
        }
        if let PhysicalKey::Code(code) = event.physical_key {
            // TODO: remappable keys
            if let Some(input) = match code {
                KeyCode::ArrowRight => Some(game_boy::Input::Right),
                KeyCode::ArrowLeft => Some(game_boy::Input::Left),
                KeyCode::ArrowUp => Some(game_boy::Input::Up),
                KeyCode::ArrowDown => Some(game_boy::Input::Down),
                KeyCode::KeyX => Some(game_boy::Input::A),
                KeyCode::KeyZ => Some(game_boy::Input::B),
                KeyCode::KeyA => Some(game_boy::Input::Select),
                KeyCode::KeyS => Some(game_boy::Input::Start),
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
        puffin::set_scopes_on(self.options_ui_state.is_profiling());
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();
        self.task_manager.poll();

        if let Some(rom_file) = self
            .game_state
            .init_rom_file
            .try_borrow_mut()
            .ok()
            .and_then(|mut rfb| rfb.take())
        {
            self.game_state.paused = self.app_config.start_paused;
            match game_boy::System::new(rom_file) {
                Ok(mut emu) => {
                    self.audio_driver.set_gb_output(emu.get_audio_output());
                    self.emulation_state = Some(emu)
                }
                Err(err) => log::warn!("Couldn't start emulation from file: {err}"),
            }
        }
        if let Some(emu) = self.emulation_state.as_mut() {
            emu.set_breakpoint_addr(self.game_state.breakpoint_addr);
        }
        if !self.game_state.paused {
            self.advance_emulation_timed(delta_time);
        }

        self.audio_driver
            .set_sample_rate(self.app_config.audio_sample_rate.get());
    }
    fn advance_emulation_timed(&mut self, time: Duration) {
        puffin::profile_function!();
        let mut frame_duration = game_boy::SystemTime::from_seconds(time.as_secs_f64());
        loop {
            let Some(emu) = self.emulation_state.as_mut() else {
                break;
            };
            let (events, elapsed_time) = emu.advance(frame_duration);
            match self.receive_emulation_events(events) {
                EmulationEventReaction::Continue => (),
                EmulationEventReaction::Pause => {
                    self.game_state.paused = true;
                    return;
                }
            };
            if elapsed_time > frame_duration {
                break;
            } else {
                frame_duration -= elapsed_time
            }
        }
    }
    fn receive_emulation_events(&mut self, events: game_boy::Events) -> EmulationEventReaction {
        let mut res = EmulationEventReaction::Continue;
        if events.has_vblank() {
            log::debug!("VBlank detected, signaling game render");
            self.game_state.new_game_frame_requested = true;
            self.audio_driver.signal_vblank();
        }
        if events.has_breakpoint() {
            log::debug!("Reached breakpoint, pausing emulation");
            res = EmulationEventReaction::Pause;
        }
        res
    }
    pub fn advance_post_render(&mut self) {
        if self.game_state.paused {
            self.audio_driver.signal_sleep();
        }
    }
    pub fn config_gui_ctx(&self, ctx: &egui::Context) {
        ctx.set_zoom_factor(self.app_config.gui_scale.zoom_factor());
        ctx.set_theme(self.app_config.theme);
    }
}

enum EmulationEventReaction {
    Continue,
    Pause,
}
