// Copyright (C) 2026 XYZabc123456789
//
// This file is part of Voxelerium.
//
// Voxelerium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Voxelerium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Voxelerium.  If not, see <https://www.gnu.org/licenses/>.

//renderer/lib.rs
use ::utils::*;
use anyhow::bail;
use bytemuck::*;
use egui::*;
use egui_wgpu::Renderer;
use egui_winit::State;
use glam::*;
use image::*;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::{debug, error, info, instrument, trace};
use utils::input::KeyboardInput;
use wgpu::util::DeviceExt;
use wgpu::wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor};
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

mod ui;
pub use ui::UiMessage;
use ui::*;

mod types;
use types::*;

mod texture;
use texture::*;

const SPEED: f32 = 0.15;

#[derive(Debug)]
pub struct Gpu {
    window: Arc<Window>,
    #[allow(unused)]
    handle: Handle,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    ui: EguiRenderer,
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    diffuse_bind_group: BindGroup,
    #[allow(unused)]
    diffuse_texture: GameTexture,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: GameTexture,
}

impl Gpu {
    #[instrument(skip_all, name = "GPU init")]
    pub async fn init(window: Arc<Window>, handle: Handle) -> Gresult<Self> {
        trace!("Creating instance");
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::DX12,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        trace!("instance created");
        trace!("Creating surface");
        let surface = instance.create_surface(window.clone())?;
        debug!("surface created");
        trace!("creating adapter");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await;
        let adapter = match adapter {
            Ok(a) => a,
            Err(_) => {
                error!("adapter creating failed, retrying");
                instance
                    .request_adapter(&RequestAdapterOptions {
                        power_preference: PowerPreference::LowPower,
                        force_fallback_adapter: true,
                        compatible_surface: Some(&surface),
                        apply_limit_buckets: false,
                    })
                    .await?
            }
        };
        debug!("adapter created");
        let info = adapter.get_info();
        info!(
            Adapter_name = info.name,
            GPU_type = ?info.device_type,
            Driver_name = info.driver,
            Driver_info = info.driver_info,
            Backend = ?info.backend,
        );
        trace!("creating device & queue");
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;
        debug!("device & queue created");
        let width = window.inner_size().width;
        let height = window.inner_size().height;
        trace!("configuring surface");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        trace!("found compatible surface");
        surface.configure(&device, &config);
        debug!("configured surface");

        let diffuse_bytes = include_bytes!("../../assets/placeholder.png");
        let diffuse_texture =
            GameTexture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let shader = device.create_shader_module(include_wgsl!("./shaders/triangle.wgsl"));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let camera = Camera::new(
            Vec3::new(0.0, 1.0, 2.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            45.0,
            config.width as f32 / config.height as f32,
            0.1,
            100.0,
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[GpuCamera::from(&camera)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&camera_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let depth_texture = GameTexture::create_depth_texture(&device, &config, "depth_texture");

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Some(GpuVertex::desc())],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        debug!("GPU initialisation complete");
        trace!("Setting up ui");
        let ui = EguiRenderer::new(&window, &device, config.format);
        debug!("Ui settup complete");

        trace!("GPU INITIALISATION COMPLETE");

        Ok(Self {
            window,
            handle,
            surface,
            adapter,
            device,
            queue,
            config,
            ui,
            pipeline,

            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_buffer,
            camera_bind_group,
            depth_texture,
        })
    }

    pub fn resize(&mut self, s: PhysicalSize<u32>) -> Gresult<()> {
        self.config.width = s.width;
        self.config.height = s.height;

        if self.config.width == 0 || self.config.height == 0 {
            return Ok(());
        };
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            GameTexture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        self.camera.update_projection();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[GpuCamera::from(&self.camera)]),
        );
        Ok(())
    }

    pub fn render(&mut self, _focus: bool, fps: f64) -> Gresult<Rc<[UiMessage]>> {
        let output = self.surface.get_current_texture();

        let tex = match output {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(t) => {
                self.resize(self.window.inner_size())?;
                t
            }
            CurrentSurfaceTexture::Lost => {
                bail!("Lost device");
            }
            _ => return Ok(Rc::from([])),
        };

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let surface_view = tex.texture.create_view(&TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        let raw_input = self.ui.state.take_egui_input(&self.window);

        let mut messages = Rc::from([]);

        let mut output = self.ui.context.run_ui(raw_input, |ctx| {
            messages = EguiRenderer::draw(ctx, fps);
        });

        self.ui
            .state
            .handle_platform_output(&self.window, output.platform_output);

        for (id, image_deltas) in output.textures_delta.set.drain() {
            for image_delta in image_deltas {
                self.ui
                    .renderer
                    .update_texture(&self.device, &self.queue, id, &image_delta);
            }
        }

        let paint_jobs = self
            .ui
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        let width = self.config.width;
        let height = self.config.height;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: output.pixels_per_point,
        };

        let textures_to_free = output.textures_delta.free.drain().collect::<Vec<_>>();

        let user_cmd_buffers = self.ui.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            #[allow(unused_mut)]
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.ui
                .renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen_descriptor);
        }
        self.queue.submit(
            user_cmd_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        self.queue.present(tex);

        for id in textures_to_free {
            self.ui.renderer.free_texture(&id);
        }
        Ok(messages)
    }

    #[inline]
    pub fn get_ctx(&self) -> GpuCtx<'_> {
        GpuCtx {
            window: self.window.clone(),
            adapter: &self.adapter,
            device: &self.device,
            queue: &self.queue,
            config: &self.config,
        }
    }

    #[inline]
    pub fn pass_to_ui(&mut self, event: &WindowEvent) -> egui_winit::EventResponse {
        self.ui.state.on_window_event(&self.window, event)
    }

    pub fn update_movement<const N: usize>(&mut self, ki: &mut KeyboardInput<N>) {
        let mut w = false;
        let mut a = false;
        let mut s = false;
        let mut d = false;
        for key in ki.events.into_iter() {
            match key {
                winit::keyboard::Key::Named(_) => {}
                winit::keyboard::Key::Character(c) => match c.as_str() {
                    "w" => {
                        w = true;
                    }
                    "a" => {
                        a = true;
                    }
                    "s" => {
                        s = true;
                    }
                    "d" => {
                        d = true;
                    }
                    _ => {
                        trace!("Unknown Key Pressed");
                    }
                },
                winit::keyboard::Key::Unidentified(_) => {}
                winit::keyboard::Key::Dead(_) => {}
            }
        }
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        if w && forward_mag > SPEED {
            self.camera.eye += forward_norm * SPEED;
        }
        if s {
            self.camera.eye -= forward_norm * SPEED;
        }
        let right = forward_norm.cross(self.camera.up);
        let forward = self.camera.target - self.camera.eye;
        let forward_mag = forward.length();
        if d {
            self.camera.eye =
                self.camera.target - (forward + right * SPEED).normalize() * forward_mag;
        }
        if a {
            self.camera.eye =
                self.camera.target - (forward - right * SPEED).normalize() * forward_mag;
        }

        self.camera.update_view();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[GpuCamera::from(&self.camera)]),
        );
    }
}

#[derive(Debug)]
pub struct GpuCtx<'a> {
    pub window: Arc<Window>,
    pub adapter: &'a Adapter,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub config: &'a SurfaceConfiguration,
}
