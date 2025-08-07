use std::fmt::Debug;

use egui::ahash::HashMap;
use winit::{
    event::WindowEvent,
    window::{Window, WindowId},
};

use super::{renderer::WgpuRenderState, state::AppState, ui::UiLayout};

pub struct EguiRenderer {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    layout: UiLayout,
}

impl EguiRenderer {
    pub fn context(&self) -> egui::Context {
        self.state.egui_ctx().clone()
    }
    pub fn new(
        layout: UiLayout,
        device: &wgpu::Device,
        window: &winit::window::Window,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let egui_ctx = egui::Context::default();

        let renderer = egui_wgpu::Renderer::new(device, output_color_format, None, 1, true);
        let state = egui_winit::State::new(
            egui_ctx,
            egui::viewport::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(2048),
        );

        EguiRenderer {
            layout,
            state,
            renderer,
        }
    }

    pub fn handle_input(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn build_render_ui(
        &mut self,
        render_state: &WgpuRenderState,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        texture_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        app_state: &mut AppState,
    ) {
        let raw_input = self.state.take_egui_input(window);

        let ctx = self.context();
        ctx.begin_pass(raw_input);
        self.layout.build(&ctx, app_state);
        let full_output = ctx.end_pass();

        self.state
            .handle_platform_output(window, full_output.platform_output);
        let tris = self
            .context()
            .tessellate(full_output.shapes, ctx.pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(
                &render_state.device,
                &render_state.queue,
                *id,
                image_delta,
            );
        }
        self.renderer.update_buffers(
            &render_state.device,
            &render_state.queue,
            encoder,
            &tris,
            &screen_descriptor,
        );
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui main render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            &tris,
            &screen_descriptor,
        );

        for texture_id in &full_output.textures_delta.free {
            self.renderer.free_texture(texture_id);
        }
    }
}

impl Debug for EguiRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiState(???)")
    }
}
