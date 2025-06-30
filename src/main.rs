use std::rc::Rc;

use gb::{Bus, Core};
use macroquad::prelude::*;

mod gb;

const FONT_SIZE: f32 = 30.0;
const LINE_SPACING: f32 = FONT_SIZE * 0.8;

fn draw_core(core: &Core, x: f32, y: f32) {
    let mut line_len = 0.;
    let size = draw_text("STATUS:", x, y, FONT_SIZE, WHITE);
    line_len += size.width;
    for (flag_char, flag) in [
        ("Z", core.get_z_flag()),
        ("N", core.get_n_flag()),
        ("H", core.get_h_flag()),
        ("C", core.get_c_flag()),
        ("IME", core.get_ime()),
    ] {
        line_len += 10.
            + draw_text(
                flag_char,
                x + line_len,
                y,
                30.,
                if flag { GREEN } else { RED },
            )
            .width;
    }
    let mut line_y = y + LINE_SPACING;
    for (register_str, register_val) in [
        ("A", core.get_a()),
        ("B", core.get_b()),
        ("C", core.get_c()),
        ("D", core.get_d()),
        ("E", core.get_e()),
    ] {
        draw_text(
            &format!("{register_str}: ${register_val:02x} [{register_val}]"),
            x,
            line_y,
            30.,
            WHITE,
        );
        line_y += LINE_SPACING;
    }
    for (register_str, register_val) in [
        ("HL", core.get_hl()),
        ("SP", core.get_sp()),
        ("PC", core.get_pc()),
    ] {
        draw_text(
            &format!("{register_str}: ${register_val:04x} [{register_val}]"),
            x,
            line_y,
            30.,
            WHITE,
        );
        line_y += LINE_SPACING;
    }
}

fn draw_ram(bus: &Bus, x: f32, y: f32, mut addr: u16, rows: usize, cols: usize) {
    let mut ram_y = y;
    for _row in 00..rows {
        let mut line = String::with_capacity(20 + cols * 6);
        line += &format!("${addr:04x}:");
        for _col in 0..cols {
            line += &format!(" {:02x}", bus.read(addr));
            addr += 1;
        }
        draw_text(&line, x, ram_y, FONT_SIZE, WHITE);
        ram_y += LINE_SPACING;
    }
}

#[macroquad::main("CVGB")]
async fn main() {
    let rom0 = std::fs::read("test/mult.gb").unwrap();
    let bus = Rc::new(Bus::new());
    let mut core = Core::new(bus.clone());
    for addr in 0..0x4000 {
        core.write(addr, rom0[addr as usize]);
    }

    loop {
        clear_background(DARKBLUE);

        if is_key_pressed(KeyCode::Space) {
            core.clock();
            while core.is_executing_instruction() {
                core.clock();
            }
        }

        draw_ram(&bus, 2., 20., 0x0000, 16, 16);
        draw_ram(&bus, 2., 20. + LINE_SPACING * 17., 0x8000, 16, 16);
        draw_core(&core, 800., 20.);

        next_frame().await
    }
}
