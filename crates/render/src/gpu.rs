//! The `wgpu` renderer: the scene is drawn into a low-resolution target, which
//! is then scaled onto the window with nearest-neighbor filtering.

use std::fmt;
use std::mem::size_of;

use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::mesh::{Mesh, Vertex};
use crate::resolution::RenderResolution;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

/// Why the renderer could not start.
#[derive(Debug)]
pub enum RenderError {
    /// The window could not provide a surface.
    Surface(wgpu::CreateSurfaceError),
    /// No GPU can present to the window.
    Adapter(wgpu::RequestAdapterError),
    /// The GPU refused to open a device.
    Device(wgpu::RequestDeviceError),
    /// The surface and adapter share no usable configuration.
    UnsupportedSurface,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Surface(err) => write!(f, "could not create a surface: {err}"),
            Self::Adapter(err) => write!(f, "no suitable GPU: {err}"),
            Self::Device(err) => write!(f, "could not open the GPU: {err}"),
            Self::UnsupportedSurface => f.write_str("the GPU cannot present to this window"),
        }
    }
}

impl std::error::Error for RenderError {}

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

/// The offscreen color and depth the scene is drawn into.
struct SceneTarget {
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
}

impl SceneTarget {
    fn new(
        device: &wgpu::Device,
        resolution: RenderResolution,
        format: wgpu::TextureFormat,
    ) -> Self {
        let make = |label, format, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: resolution.width(),
                        height: resolution.height(),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        Self {
            color: make(
                "scene color",
                format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            depth: make(
                "scene depth",
                DEPTH_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        }
    }
}

/// Draws meshes seen through a [`Camera`] onto a window.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    resolution: RenderResolution,
    target: SceneTarget,
    scene_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
    meshes: Vec<GpuMesh>,
}

impl Renderer {
    /// Opens the GPU and builds both pipelines. `size` is the window's
    /// current inner size in pixels.
    ///
    /// Blocks while the GPU is opened, which happens once at startup.
    pub fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        size: (u32, u32),
        resolution: RenderResolution,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .map_err(RenderError::Surface)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(RenderError::Adapter)?;
        eprintln!("gpu: {:?}", adapter.get_info());
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(RenderError::Device)?;

        // A minimized window reports a zero side, which no surface accepts.
        let mut config = surface
            .get_default_config(&adapter, size.0.max(1), size.1.max(1))
            .ok_or(RenderError::UnsupportedSurface)?;
        // Fifo is the one mode every platform has, and it caps the frame rate.
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        // The target shares the surface format so the blit is a plain copy
        // of what the scene pass wrote.
        let target = SceneTarget::new(&device, resolution, config.format);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (scene_pipeline, camera_bind_group) =
            scene_pipeline(&device, config.format, &camera_buffer);
        let (blit_pipeline, blit_bind_group) = blit_pipeline(&device, config.format, &target);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            resolution,
            target,
            scene_pipeline,
            camera_buffer,
            camera_bind_group,
            blit_pipeline,
            blit_bind_group,
            meshes: Vec::new(),
        })
    }

    /// Copies a mesh to the GPU; it is drawn on every later frame.
    pub fn add_mesh(&mut self, mesh: &Mesh) {
        let vertices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh vertices"),
                contents: bytemuck::cast_slice(mesh.vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let indices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh indices"),
                contents: bytemuck::cast_slice(mesh.indices()),
                usage: wgpu::BufferUsages::INDEX,
            });
        self.meshes.push(GpuMesh {
            vertices,
            indices,
            // `Mesh` guarantees the count fits: its indices address u32 vertices.
            index_count: mesh.indices().len() as u32,
        });
    }

    /// Follows the window's new inner size. A zero side (minimized) is
    /// ignored until the window comes back.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Draws one frame. A frame the surface cannot take right now (minimized,
    /// resized mid-frame) is skipped; the next call tries again.
    pub fn render(&mut self, camera: &Camera) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            other => {
                eprintln!("could not get a frame: {other:?}");
                return;
            }
        };
        let Some(viewport) = self.resolution.fit(self.config.width, self.config.height) else {
            return;
        };

        let view_projection = camera
            .view_projection(self.resolution.aspect())
            .to_cols_array_2d();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&view_projection));

        let surface_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.target.color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.target.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.scene_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            for mesh in &self.meshes {
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    // Clears the letterbox bars.
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(
                viewport.x() as f32,
                viewport.y() as f32,
                viewport.width() as f32,
                viewport.height() as f32,
                0.0,
                1.0,
            );
            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, &self.blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }
}

fn scene_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_buffer: &wgpu::Buffer,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let module = device.create_shader_module(wgpu::include_wgsl!("gpu/scene.wgsl"));
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("camera layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("camera"),
        layout: &bind_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scene layout"),
        bind_group_layouts: &[Some(&bind_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scene"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
            })],
        },
        // No culling: the grey-box room is seen from the inside, and the
        // winding of placeholder geometry is not worth policing yet.
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });
    (pipeline, bind_group)
}

fn blit_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    target: &SceneTarget,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let module = device.create_shader_module(wgpu::include_wgsl!("gpu/blit.wgsl"));
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blit layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    // Nearest, so upscaled pixels stay hard-edged.
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("blit sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blit"),
        layout: &bind_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&target.color),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blit layout"),
        bind_group_layouts: &[Some(&bind_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blit"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });
    (pipeline, bind_group)
}
