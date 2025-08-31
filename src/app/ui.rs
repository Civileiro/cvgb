use std::cell::Ref;

use options_ui::OptionsUiState;

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
                OptionsUiState::build(ctx, state);
            }
        }
    }
}

fn show_main_menu(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Cvgb main menu");
        if ui.button("Open rom file").clicked() {
            let state_rom_file = state.game_state.init_rom_file.clone();
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

pub mod options_ui {

    use compact_str::format_compact;
    use enum_assoc::Assoc;

    use crate::{
        app::state::AppState,
        game_boy::{self, Opcode},
    };

    #[derive(Debug, Default)]
    pub struct OptionsUiState {
        menu: Menu,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Assoc)]
    #[func(pub fn name(&self) -> &str)]
    enum Menu {
        #[assoc(name = "Options")]
        #[default]
        Options,
        #[assoc(name = "Debug")]
        Debug,
    }

    impl Menu {
        const fn all() -> [Self; 2] {
            [Self::Options, Self::Debug]
        }
    }

    impl OptionsUiState {
        pub fn build(ctx: &egui::Context, state: &mut AppState) {
            let slf = &mut state.options_ui_state;
            egui::TopBottomPanel::top("options_top_bar")
                // .frame(egui::Frame::new().inner_margin(4))
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for menu in Menu::all() {
                            if ui.selectable_label(slf.menu == menu, menu.name()).clicked() {
                                slf.menu = menu;
                            }
                        }
                    });
                });

            match slf.menu {
                Menu::Options => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::Grid::new("options_grid")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Theme");
                                state.app_config.theme.radio_buttons(ui);
                                ui.end_row();

                                ui.label("GUI Scale");
                                state.app_config.gui_scale.radio_buttons(ui);
                                ui.end_row();

                                ui.label("Start emulation paused");
                                ui.checkbox(&mut state.app_config.start_paused, ());
                                ui.end_row();
                            });
                    });
                }
                Menu::Debug => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let Some(emu) = state.emulation_state.as_mut() else {
                            ui.vertical_centered_justified(|ui| {
                                ui.label("Emulation is not running...");
                                ui.checkbox(
                                    &mut state.app_config.start_paused,
                                    "Start emulation paused",
                                );
                            });
                            return;
                        };
                        egui::SidePanel::left("cpu_panel")
                            .resizable(true)
                            .default_width(160.0)
                            .width_range(120.0..=200.0)
                            .show_inside(ui, |ui| {
                                ui.vertical_centered(|ui| ui.heading("CPU"));
                                show_cpu(ui, emu.get_cpu());
                            });
                        egui::CentralPanel::default().show_inside(ui, |ui| {
                            show_disassembly(ui, emu);
                            ui.horizontal(|ui| {
                                ui.toggle_value(&mut state.game_state.paused, "Pause");

                                if ui
                                    .add_enabled(state.game_state.paused, egui::Button::new("Step"))
                                    .clicked()
                                {
                                    emu.step();
                                }
                            });
                        });
                    });
                }
            }
        }
    }

    fn show_cpu(ui: &mut egui::Ui, cpu: &game_boy::Cpu) {
        let regs = cpu.get_registers();
        let itrs = cpu.requested_interrupts();
        let flag_color = |flag: bool| {
            if flag {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            }
        };

        ui.horizontal_wrapped(|ui| {
            for (label, flag) in [
                ("VBLANK", itrs.vblank()),
                ("LCD", itrs.lcd()),
                ("TIMER", itrs.timer()),
                ("SERIAL", itrs.serial()),
                ("JOYPAD", itrs.joypad()),
            ] {
                ui.label(egui::RichText::new(label).color(flag_color(flag)));
            }
        });

        ui.horizontal(|ui| {
            for (label, flag) in [
                ("Z", regs.get_z_flag()),
                ("N", regs.get_n_flag()),
                ("H", regs.get_h_flag()),
                ("C", regs.get_c_flag()),
                ("IME", cpu.get_ime()),
            ] {
                ui.label(egui::RichText::new(label).color(flag_color(flag)));
            }
        });

        egui::Grid::new("reg_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for ((label0, value0), (label1, value1)) in [
                    (("A", regs.a), ("F", regs.f.into())),
                    (("B", regs.b), ("C", regs.c)),
                    (("D", regs.d), ("E", regs.e)),
                    (("H", regs.h), ("L", regs.l)),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(label0);
                        ui.monospace(format!("0x{value0:02x}"));
                    });
                    ui.horizontal(|ui| {
                        ui.label(label1);
                        ui.monospace(format!("0x{value1:02x}"));
                    });
                    ui.end_row();
                }
                ui.horizontal(|ui| {
                    ui.label("SP");
                    ui.monospace(format!("0x{:04x}", regs.sp));
                });
                ui.horizontal(|ui| {
                    ui.label("PC");
                    ui.monospace(format!("0x{:04x}", regs.pc));
                });
                ui.end_row();
            });
    }

    fn prev_isntruction_addr(addr: u16, system: &game_boy::System) -> Option<u16> {
        for i in (1..=3).rev() {
            if addr < i {
                continue;
            }
            let prev_addr = addr - i;
            let Some(data) = system.get_context().debug_read_memory(prev_addr) else {
                continue;
            };
            if Opcode::size(data) == i.into() {
                return Some(prev_addr);
            }
        }
        None
    }

    fn disassembly(system: &game_boy::System) -> Vec<(u16, String)> {
        let (cpu, ctx) = system.get_cpu_context();
        let (_, center_addr) = cpu.current_opcode();
        const LEN: u16 = 10;
        let half = LEN / 2;
        let mut res = Vec::with_capacity(LEN.into());

        let mut curr_addr = center_addr;
        for _ in 0..half {
            let Some(prev_addr) = prev_isntruction_addr(center_addr, system) else {
                continue;
            };
            curr_addr = prev_addr
        }
        while res.len() != LEN.into() {
            let Some(data) = [
                ctx.debug_read_memory(curr_addr),
                ctx.debug_read_memory(curr_addr + 1),
                ctx.debug_read_memory(curr_addr + 2),
            ]
            .into_iter()
            .collect::<Option<Vec<u8>>>() else {
                break;
            };
            let instruction_disassembly = Opcode::disassemble(&data);
            res.push((curr_addr, instruction_disassembly));
            curr_addr += Opcode::size(data[0]) as u16;
        }
        res
    }

    fn show_disassembly(ui: &mut egui::Ui, system: &game_boy::System) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            let (_, curr_addr) = system.get_cpu().current_opcode();
            for (addr, instruction_str) in disassembly(system) {
                ui.horizontal(|ui| {
                    let addr_str = format!("${addr:04x}");
                    if addr != curr_addr {
                        ui.label(addr_str)
                    } else {
                        ui.label(egui::RichText::new(addr_str).color(egui::Color32::CYAN))
                    };
                    ui.label(instruction_str);
                });
            }
        });
    }
}
