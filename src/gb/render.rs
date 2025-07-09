use std::cell::Ref;

use super::{GameBoy, OUTPUT_HEIGHT, OUTPUT_WIDTH, ppu::Pixel};

use macroquad::prelude::*;
const FONT_SIZE: f32 = 30.0;
const LINE_SPACING: f32 = FONT_SIZE * 0.8;

impl GameBoy {
    pub fn get_frame(&self) -> Box<[Pixel]> {
        self.ppu.get_frame()
    }
    pub fn draw_core(&self, x: f32, y: f32) {
        let core = self.cpu.core();
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

    pub fn draw_ppu(&self, x: f32, y: f32) {
        let data = self.ppu.data();
        let size = draw_text("PPU: ", x, y, FONT_SIZE, WHITE);
        let mut line_x = x + size.width;
        let mut line_y = size.height;

        for (flag_char, flag) in [("E", data.lcdc.ppu_enable())] {
            let line_size = draw_text(flag_char, line_x, y, 30., if flag { GREEN } else { RED });
            line_x += 10. + line_size.width;
        }
        line_y += LINE_SPACING;
        for (data_str, val) in [("LY", data.ly), ("Mode", data.mode as u8)] {
            draw_text(
                &format!("{data_str}: ${val:04x} [{val}]"),
                x,
                line_y,
                30.,
                WHITE,
            );
            line_y += LINE_SPACING;
        }
    }

    pub fn draw_ram(&self, x: f32, y: f32, mut addr: u16, rows: usize, cols: usize) {
        let mut ram_y = y;
        for _row in 00..rows {
            let mut line = String::with_capacity(20 + cols * 6);
            line += &format!("${addr:04x}:");
            for _col in 0..cols {
                line += &format!(" {:02x}", self.bus.const_read(addr));
                addr = addr.wrapping_add(1);
            }
            draw_text(&line, x, ram_y, FONT_SIZE, WHITE);
            ram_y += LINE_SPACING;
        }
    }
}
