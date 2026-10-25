//! The HUD pass: textured quads blended over the finished scene.

use std::mem::size_of;

use wgpu::util::DeviceExt;

use crate::hud::{HudBatch, HudVertex, MAX_QUADS, font};
use crate::resolution::RenderResolution;

/// Indices per quad: two triangles.
const QUAD_INDICES: usize = 6;

pub(super) struct HudPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

impl HudPass {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        resolution: RenderResolution,
    ) -> Self {
        let module = device.create_shader_module(wgpu::include_wgsl!("hud.wgsl"));

        let atlas = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("hud atlas"),
                size: wgpu::Extent3d {
                    width: font::ATLAS_WIDTH,
                    height: font::ATLAS_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &font::pixels(),
        );
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        // Nearest, so glyph edges stay hard.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hud sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let screen = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hud screen"),
            contents: bytemuck::bytes_of(&[
                resolution.width() as f32,
                resolution.height() as f32,
                0.0,
                0.0,
            ]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hud layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud"),
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<HudVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x4
                    ],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        // Quads always share this index pattern, so it is built once and
        // only the vertices change from frame to frame.
        let quad_indices: Vec<u16> = (0..MAX_QUADS)
            .flat_map(|quad| {
                let first = (quad * 4) as u16;
                [first, first + 1, first + 2, first + 2, first + 1, first + 3]
            })
            .collect();
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hud indices"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud vertices"),
            size: (MAX_QUADS * 4 * size_of::<HudVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            vertices,
            indices,
            index_count: 0,
        }
    }

    /// Copies the batch to the GPU; it is drawn until the next call.
    pub(super) fn upload(&mut self, queue: &wgpu::Queue, batch: &HudBatch) {
        // `HudBatch` caps its quads at `MAX_QUADS`, the size of the buffers.
        self.index_count = (batch.quad_count() * QUAD_INDICES) as u32;
        if self.index_count > 0 {
            queue.write_buffer(&self.vertices, 0, bytemuck::cast_slice(batch.vertices()));
        }
    }

    /// Draws the uploaded batch, if it has any quads.
    pub(super) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.index_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
