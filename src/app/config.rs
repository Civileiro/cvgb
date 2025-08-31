use enum_assoc::Assoc;

#[derive(Debug, Default)]
pub struct Config {
    pub gui_scale: GuiScale,
    pub theme: egui::ThemePreference,
    pub start_paused: bool,
}

#[derive(Debug, Default, Assoc, Clone, Copy, PartialEq, Eq)]
#[func(pub fn zoom_factor(&self) -> f32)]
pub enum GuiScale {
    #[assoc(zoom_factor = 1.0)]
    Small,
    #[default]
    #[assoc(zoom_factor = 1.5)]
    Medium,
    #[assoc(zoom_factor = 2.25)]
    Large,
}

impl GuiScale {
    pub fn radio_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(self, Self::Small, "Small");
            ui.selectable_value(self, Self::Medium, "Medium");
            ui.selectable_value(self, Self::Large, "Large");
        });
    }
}
