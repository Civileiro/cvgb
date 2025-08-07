use super::state::AppState;

#[derive(Debug, Clone, Copy)]
pub enum UiLayout {
    MainLayout,
    OptionsLayout,
}

impl UiLayout {
    pub fn main_ui() -> Self {
        Self::MainLayout
    }
    pub fn build(&self, ctx: &egui::Context, state: &mut AppState) {
        match self {
            UiLayout::MainLayout => {
                // egui::CentralPanel::default().show(ctx, |ui| {
                //     ui.label("Main Screen UI!");
                //     if ui.button("Click me!").clicked() {
                //         println!("Main screen button clicked!")
                //     }
                // });
            }
            UiLayout::OptionsLayout => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Options Screen UI!");
                    if ui.button("Click me!").clicked() {
                        println!("Options screen button clicked!")
                    }
                });
            }
        }
    }
}
