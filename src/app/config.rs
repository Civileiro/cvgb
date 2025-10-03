use enum_assoc::Assoc;

#[derive(Debug, Default)]
pub struct Config {
    pub gui_scale: GuiScale,
    pub theme: egui::ThemePreference,
    pub start_paused: bool,
    pub audio_sample_rate: SampleRate,
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

#[derive(Debug, Clone, Copy)]
pub struct SampleRate(u32);

impl Default for SampleRate {
    fn default() -> Self {
        Self(44_100)
    }
}

impl SampleRate {
    pub fn get(&self) -> u32 {
        self.0
    }
    pub fn set(&mut self, rate: u32) {
        self.0 = rate
    }

    const ALL_SAMPLE_RATES: [u32; 4] = [22050, 44100, 48000, 96000];

    pub fn all_sample_rates() -> &'static [u32; Self::ALL_SAMPLE_RATES.len()] {
        &Self::ALL_SAMPLE_RATES
    }
    pub fn combo_box(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("sample_rate")
            .selected_text(format!("{} Hz", self.get()))
            .show_ui(ui, |ui| {
                for &opt in Self::all_sample_rates() {
                    ui.selectable_value(&mut self.0, opt, format!("{opt} Hz"));
                }
            });
    }
}
