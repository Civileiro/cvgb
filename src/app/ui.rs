use std::cell::Ref;

use crate::game_boy;

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
                if state.emulation_state.is_none() {
                    show_main_menu(ctx, state);
                }
                preview_dropped_files(ctx);
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

fn show_main_menu(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Cvgb main menu");
        if ui.button("Open rom file").clicked() {
            let state_rom_file = state.init_rom_file.clone();
            let file_dialog_future = async move {
                let Some(file_handle) = rfd::AsyncFileDialog::new()
                    .set_title("Choose Rom")
                    .pick_file()
                    .await
                else {
                    return;
                };
                *state_rom_file.borrow_mut() = Some(file_handle.read().await.into_boxed_slice());
            };
            state.task_manager.add_task(file_dialog_future);
        }
    });
}

fn preview_dropped_files(ctx: &egui::Context) {
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("file_drop_target"),
        ));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::TextStyle::Heading.resolve(&ctx.style()),
            egui::Color32::WHITE,
        );
    }
}
