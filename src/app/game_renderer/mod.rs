mod game;
mod screen;

use game::GameRendererImpl;
use screen::ScreenRenderer;

use super::{renderer::WgpuRenderState, state::AppState};

#[derive(Debug)]
pub struct GameRenderer {
    // For rendering the game onto a texture
    game_renderer_impl: GameRendererImpl,
    // For rendering the texture onto the screen
    screen_renderer: ScreenRenderer,
}

impl GameRenderer {
    pub fn new(wgpu_state: &WgpuRenderState, surface_format: wgpu::TextureFormat) -> Self {
        let device = &wgpu_state.device;

        let game_renderer_impl = GameRendererImpl::new(device);
        let screen_renderer =
            ScreenRenderer::new(device, surface_format, game_renderer_impl.output_texture());

        Self {
            game_renderer_impl,
            screen_renderer,
        }
    }

    pub fn update(
        &mut self,
        wgpu_state: &WgpuRenderState,
        state: &AppState,
        screen_size: (u32, u32),
    ) {
        // TODO
        // self.game_renderer_impl.update(wgpu_state, state);
        self.screen_renderer.update(wgpu_state, state, screen_size);
    }

    /// Renders the game to an internal texture
    /// Should only be called when the game has a new frame to render
    pub fn render_game(&self, render_pass: &mut wgpu::RenderPass) {
        // TODO
        // self.game_renderer_impl.render(render_pass);
    }
    /// Renders the game texture
    pub fn render_screen(&self, render_pass: &mut wgpu::RenderPass) {
        self.screen_renderer.render(render_pass);
    }
}
