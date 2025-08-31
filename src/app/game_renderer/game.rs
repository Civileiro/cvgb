use crate::{
    app::{renderer::WgpuRenderState, state::AppState},
    game_boy,
};

#[derive(Debug)]
pub struct GameRendererImpl {
    // The game renderer updates the game texture with a new frame
    game_texture: wgpu::Texture,
    game_view: wgpu::TextureView,

    // The texture can be assembled by software, which skips rendering
    skip_render: bool,
    // or by rendering with a shader
    // pipeline: wgpu::RenderPipeline,
    // bind_group: wgpu::BindGroup,
    // uniform_buffer: wgpu::Buffer,
}

#[derive(Debug, Default)]
pub enum GameRenderingType {
    #[default]
    Software,
    Shader,
}

impl GameRendererImpl {
    pub fn new(device: &wgpu::Device) -> Self {
        let game_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gb"),
            size: wgpu::Extent3d {
                width: game_boy::WINDOW_WIDTH.into(),
                height: game_boy::WINDOW_HEIGHT.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let game_view = game_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("gb"),
            usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
            ..Default::default()
        });

        // let software_buffer_texture = device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("gb"),
        //     size: wgpu::Extent3d {
        //         width: game_boy::WINDOW_WIDTH.into(),
        //         height: game_boy::WINDOW_HEIGHT.into(),
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8Unorm,
        //     usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        //     view_formats: &[],
        // });

        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("gb"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("./game.wgsl").into()),
        // });
        //
        // let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("gb"),
        //     entries: &[wgpu::BindGroupLayoutEntry {
        //         binding: 0,
        //         visibility: wgpu::ShaderStages::VERTEX,
        //         ty: wgpu::BindingType::Buffer {
        //             ty: wgpu::BufferBindingType::Uniform,
        //             has_dynamic_offset: false,
        //             min_binding_size: NonZeroU64::new(16),
        //         },
        //         count: None,
        //     }],
        // });
        //
        // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("gb"),
        //     bind_group_layouts: &[&bind_group_layout],
        //     push_constant_ranges: &[],
        // });
        //
        // let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("gb"),
        //     layout: Some(&pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: None,
        //         buffers: &[],
        //         compilation_options: wgpu::PipelineCompilationOptions::default(),
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("fs_main"),
        //         targets: &[Some(game_texture.format().into())],
        //         compilation_options: wgpu::PipelineCompilationOptions::default(),
        //     }),
        //     primitive: wgpu::PrimitiveState::default(),
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState::default(),
        //     multiview: None,
        //     cache: None,
        // });
        //
        // let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("gb"),
        //     contents: &[0; 16],
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        // });
        //
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("gb"),
        //     layout: &bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: uniform_buffer.as_entire_binding(),
        //     }],
        // });

        Self {
            game_texture,
            game_view,
            skip_render: false,
            // pipeline,
            // bind_group,
            // uniform_buffer,
        }
    }

    pub fn output_texture(&self) -> &wgpu::Texture {
        &self.game_texture
    }

    fn update_buffer_texture(&self, queue: &wgpu::Queue, rgba_buffer: &[u8]) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.game_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_buffer,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(game_boy::WINDOW_WIDTH as u32 * 4),
                rows_per_image: Some(game_boy::WINDOW_HEIGHT as u32),
            },
            wgpu::Extent3d {
                width: game_boy::WINDOW_WIDTH as u32,
                height: game_boy::WINDOW_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn update(&mut self, wgpu_state: &WgpuRenderState, state: &AppState) {
        match state.game_state.game_rendering_type {
            GameRenderingType::Software => {
                let Some(emu) = state.emulation_state.as_ref() else {
                    return;
                };
                if state.game_state.new_game_frame_requested {
                    log::debug!("Software assembling new frame...");
                    let rgba_buffer = emu.get_video_buffer().make_rgba_buffer();
                    self.update_buffer_texture(&wgpu_state.queue, &rgba_buffer);
                    self.skip_render = true
                }
            }
            GameRenderingType::Shader => todo!(),
        }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        if self.skip_render {
            self.skip_render = false;
            return;
        }
        todo!()
        // render_pass.set_pipeline(&self.pipeline);
        // render_pass.set_bind_group(0, &self.bind_group, &[]);
        // render_pass.draw(0..3, 0..1);
    }
}
