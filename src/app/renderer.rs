use std::{collections::HashMap, fmt::Debug, sync::Arc};

use winit::window::{Window, WindowId};

use super::{
    game_renderer::GameRenderer, gui_renderer::EguiRenderer, state::AppState, windows::AppScreen,
};

#[derive(Debug)]
pub struct RenderState {
    render_state: WgpuRenderState,
    window_data: HashMap<WindowId, WindowData>,
}

#[derive(Debug)]
pub struct WgpuRenderState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Debug)]
struct WindowData {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
    pub renderer: WindowRenderer,
}

#[derive(Debug)]
struct WindowRenderer {
    app_screen: AppScreen,
    gui_renderer: Option<EguiRenderer>,
    game_renderer: Option<GameRenderer>,
}

impl WindowData {
    fn new(render_state: &WgpuRenderState, window: Arc<Window>, app_screen: AppScreen) -> Self {
        let size = window.inner_size();
        let surface = render_state
            .instance
            .create_surface(window.clone())
            .unwrap();
        let cap = surface.get_capabilities(&render_state.adapter);
        let surface_format = cap.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 1,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&render_state.device, &surface_config);

        let renderer = WindowRenderer {
            app_screen,
            gui_renderer: app_screen.layout().map(|layout| {
                EguiRenderer::new(layout, &render_state.device, &window, surface_format)
            }),
            game_renderer: app_screen
                .is_main()
                .then(|| GameRenderer::new(render_state, surface_format)),
        };

        WindowData {
            window,
            surface,
            surface_config,
            surface_format,
            renderer,
        }
    }
}

impl RenderState {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        RenderState {
            render_state: WgpuRenderState {
                instance,
                adapter,
                device,
                queue,
            },
            window_data: Default::default(),
        }
    }

    pub fn register_window(&mut self, window: Arc<Window>, app_screen: AppScreen) {
        let window_id = window.id();
        let data = WindowData::new(&self.render_state, window, app_screen);
        self.window_data.insert(window_id, data);
    }
    pub fn unregister_window(&mut self, window_id: WindowId) {
        self.window_data.remove(&window_id);
    }

    fn get_window_data(&self, window_id: WindowId) -> &WindowData {
        self.window_data
            .get(&window_id)
            .expect("Window should be registered before use")
    }

    pub fn get_window(&self, window_id: WindowId) -> &Window {
        &self.get_window_data(window_id).window
    }

    pub fn resize(&mut self, window_id: WindowId, new_size: winit::dpi::PhysicalSize<u32>) {
        let data = self
            .window_data
            .get_mut(&window_id)
            .expect("Window should be registered before use");
        data.surface_config.width = new_size.width;
        data.surface_config.height = new_size.height;
        data.surface
            .configure(&self.render_state.device, &data.surface_config);
    }

    pub fn handle_gui_input(
        &mut self,
        window_id: WindowId,
        event: &winit::event::WindowEvent,
    ) -> Option<egui_winit::EventResponse> {
        if let Some(data) = self.window_data.get_mut(&window_id)
            && let Some(gui_renderer) = data.renderer.gui_renderer.as_mut()
        {
            Some(gui_renderer.handle_input(&data.window, event))
        } else {
            None
        }
    }

    pub fn render(&mut self, state: &mut AppState) {
        for window_data in self.window_data.values_mut() {
            let mut encoder = self
                .render_state
                .device
                .create_command_encoder(&Default::default());
            let surface_texture = window_data
                .surface
                .get_current_texture()
                .expect("swapchain should have texture to acquire");
            let texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(window_data.surface_format),
                    ..Default::default()
                });

            // Game render pass
            if let Some(game_renderer) = window_data.renderer.game_renderer.as_mut() {
                let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Draw commands
                game_renderer.render(&mut renderpass);
            }
            // Gui render pass
            if let Some(gui_renderer) = window_data.renderer.gui_renderer.as_mut() {
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [
                        window_data.surface_config.width,
                        window_data.surface_config.height,
                    ],
                    pixels_per_point: window_data.window.scale_factor() as f32,
                };
                gui_renderer.build_render_ui(
                    &self.render_state,
                    &mut encoder,
                    &window_data.window,
                    &texture_view,
                    screen_descriptor,
                    state,
                );
            }
            // Submit command in the queue to execute
            self.render_state.queue.submit([encoder.finish()]);
            window_data.window.pre_present_notify();
            surface_texture.present();
        }
    }
}
