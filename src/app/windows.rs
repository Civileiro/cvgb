use std::collections::HashMap;

use enum_assoc::Assoc;
use winit::{
    dpi::PhysicalSize,
    window::{WindowAttributes, WindowId},
};

use crate::game_boy;

use super::ui::UiLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Assoc)]
#[func(pub fn is_main(&self) -> bool { false })]
#[func(pub fn layout(&self) -> Option<UiLayout>)]
#[func(pub fn attributes(&self) -> WindowAttributes { Default::default() })]
pub enum AppScreen {
    #[assoc(
        is_main = true,
        layout = UiLayout::MainLayout,
        attributes =
            WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(
                game_boy::WINDOW_WIDTH as u32 * 4,
                game_boy::WINDOW_HEIGHT as u32 * 4,
            ))
    )]
    MainScreen,
    #[assoc(
        layout = UiLayout::OptionsLayout,
    )]
    OptionsScreen,
}

#[derive(Debug, Default)]
pub struct WindowRegistry {
    id_to_app: HashMap<WindowId, AppScreen>,
    app_to_id: HashMap<AppScreen, WindowId>,
}

impl WindowRegistry {
    pub fn register_window(&mut self, window_id: WindowId, app_screen: AppScreen) {
        log::info!("registering window {window_id:?} {app_screen:?}");
        self.id_to_app.insert(window_id, app_screen);
        self.app_to_id.insert(app_screen, window_id);
    }
    pub fn unregister_by_id(&mut self, window_id: WindowId) -> Option<AppScreen> {
        log::info!("unregistering window {window_id:?}");
        self.id_to_app.remove(&window_id).inspect(|app_screen| {
            self.app_to_id.remove(app_screen);
        })
    }
    pub fn unregister_by_screen(&mut self, app_screen: AppScreen) -> Option<WindowId> {
        log::info!("unregistering window {app_screen:?}");
        self.app_to_id.remove(&app_screen).inspect(|window_id| {
            self.id_to_app.remove(window_id);
        })
    }
    pub fn get_screen(&self, window_id: WindowId) -> Option<AppScreen> {
        self.id_to_app.get(&window_id).copied()
    }
    pub fn get_id(&self, app_screen: AppScreen) -> Option<WindowId> {
        self.app_to_id.get(&app_screen).copied()
    }
}
