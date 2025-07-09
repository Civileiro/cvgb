use std::ops::Deref;

use gb::{GameBoy, OUTPUT_HEIGHT, OUTPUT_WIDTH};
use macroquad::prelude::*;

mod gb;

#[macroquad::main("CVGB")]
async fn main() {
    let rom = std::fs::read("test/Tetris (World) (Rev 1).gb").unwrap();

    let game_frame = Texture2D::from_rgba8(
        OUTPUT_WIDTH as _,
        OUTPUT_HEIGHT as _,
        &vec![0x0F; 4 * (OUTPUT_HEIGHT as usize) * (OUTPUT_WIDTH as usize)],
    );
    let system = GameBoy::new(rom.into_boxed_slice());
    let mut to_next_frame = false;

    loop {
        clear_background(DARKBLUE);

        if is_key_pressed(KeyCode::Space) {
            system.step_instruction();
        }
        if is_key_pressed(KeyCode::F) {
            to_next_frame = true;
        } else if is_key_pressed(KeyCode::G) {
            to_next_frame = false;
        }

        to_next_frame = true;
        while to_next_frame && !system.frame_is_ready() {
            system.step_instruction();
        }
        to_next_frame = false;

        let frame = system.get_frame();
        let mut bytes = Vec::with_capacity(frame.len() * 4);
        for pixel in frame {
            bytes.push(pixel.r);
            bytes.push(pixel.g);
            bytes.push(pixel.b);
            bytes.push(0xFF);
        }

        game_frame.update_from_bytes(OUTPUT_WIDTH as _, OUTPUT_HEIGHT as _, &bytes);
        let h = screen_height() - 40.;
        let frame_width = h * OUTPUT_WIDTH as f32 / OUTPUT_HEIGHT as f32;
        let frame_height = h;
        draw_texture_ex(
            &game_frame,
            20.,
            20.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(frame_width, frame_height)),
                ..Default::default()
            },
        );
        system.draw_core(frame_width + 40., 20.);
        system.draw_ppu(frame_width + 400., 20.);
        system.draw_ram(frame_width + 40., 300., 0xFF80, 8, 16);
        next_frame().await
    }
}
