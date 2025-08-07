use std::collections::HashMap;

use enum_assoc::Assoc;
use winit::window::WindowId;

use super::ui::UiLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Assoc)]
#[func(pub fn layout(&self) -> Option<UiLayout>)]
#[func(pub fn is_main(&self) -> bool { false })]
pub enum AppWindow {
    #[assoc(layout = UiLayout::MainScreen, is_main = true)]
    MainWindow,
    #[assoc(layout = UiLayout::OptionsScreen)]
    OptionsWindow,
}

#[derive(Debug, Default)]
pub struct WindowRegistry {
    id_to_app: HashMap<WindowId, AppWindow>,
    app_to_id: HashMap<AppWindow, WindowId>,
}

impl WindowRegistry {
    pub fn register_window(&mut self, window_id: WindowId, app_window: AppWindow) {
        log::info!("registering window {window_id:?} {app_window:?}");
        self.id_to_app.insert(window_id, app_window);
        self.app_to_id.insert(app_window, window_id);
    }
    pub fn unregister_by_id(&mut self, window_id: WindowId) -> Option<AppWindow> {
        log::info!("unregistering window {window_id:?}");
        self.id_to_app.remove(&window_id).inspect(|app_window| {
            self.app_to_id.remove(app_window);
        })
    }
    pub fn unregister_by_app_window(&mut self, app_window: AppWindow) -> Option<WindowId> {
        log::info!("unregistering window {app_window:?}");
        self.app_to_id.remove(&app_window).inspect(|window_id| {
            self.id_to_app.remove(window_id);
        })
    }
    pub fn get_app_window(&self, window_id: WindowId) -> Option<AppWindow> {
        self.id_to_app.get(&window_id).copied()
    }
    pub fn get_id(&self, app_window: AppWindow) -> Option<WindowId> {
        self.app_to_id.get(&app_window).copied()
    }
}
