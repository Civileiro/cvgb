use std::{collections::HashMap, fmt::Debug, sync::Arc};

use winit::window::{Window, WindowId};

use super::{gui_renderer::EguiRenderer, state::AppState, ui::UiLayout};

#[derive(Debug)]
pub struct RendererState {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window_data: HashMap<WindowId, WindowData>,
}

#[derive(Debug)]
struct WindowData {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
    pub ui: Option<WindowUi>,
}

struct WindowUi {
    layout: UiLayout,
    renderer: EguiRenderer,
}

impl Debug for WindowUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowUi")
            .field("layout", &self.layout)
            .finish()
    }
}

impl WindowData {
    fn new(
        window: Arc<Window>,
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        ui_layout: Option<UiLayout>,
    ) -> Self {
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(adapter);
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
        surface.configure(device, &surface_config);

        let ui = ui_layout.map(|layout| {
            let renderer = EguiRenderer::new(device, &window, surface_format, None, 1);
            WindowUi { layout, renderer }
        });

        WindowData {
            window,
            surface,
            surface_config,
            surface_format,
            ui,
        }
    }
}

impl RendererState {
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

        RendererState {
            instance,
            adapter,
            device,
            queue,
            window_data: Default::default(),
        }
    }

    pub fn register_window(&mut self, window: Arc<Window>, ui: Option<UiLayout>) {
        let window_id = window.id();
        let data = WindowData::new(window, &self.instance, &self.device, &self.adapter, ui);
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
        data.surface.configure(&self.device, &data.surface_config);
    }

    pub fn handle_gui_input(
        &mut self,
        window_id: WindowId,
        event: &winit::event::WindowEvent,
    ) -> Option<egui_winit::EventResponse> {
        if let Some(data) = self.window_data.get_mut(&window_id)
            && let Some(ui) = data.ui.as_mut()
        {
            Some(ui.renderer.handle_input(&data.window, event))
        } else {
            None
        }
    }

    pub fn render(&mut self, state: &mut AppState) {
        for window_data in self.window_data.values_mut() {
            let mut encoder = self.device.create_command_encoder(&Default::default());
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
            {
                let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            }
            // Gui render pass
            if let Some(ui) = window_data.ui.as_mut() {
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [
                        window_data.surface_config.width,
                        window_data.surface_config.height,
                    ],
                    pixels_per_point: window_data.window.scale_factor() as f32,
                };
                ui.renderer.build_render_ui(
                    &self.device,
                    &self.queue,
                    &mut encoder,
                    &window_data.window,
                    &texture_view,
                    screen_descriptor,
                    state,
                    ui.layout,
                );
            }
            // Submit command in the queue to execute
            self.queue.submit([encoder.finish()]);
            window_data.window.pre_present_notify();
            surface_texture.present();
        }
    }
}
