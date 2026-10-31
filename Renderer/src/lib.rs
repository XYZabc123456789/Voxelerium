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

// Renderer lib.rs
use anyhow::{anyhow, bail};
use egui::*;
use egui_wgpu::Renderer;
use egui_winit::State;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::{debug, error, info, instrument, trace};
use utils::*;
use wgpu::wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor};
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

mod ui;
pub use ui::UiMessage;
use ui::*;

#[derive(Debug)]
pub struct Gpu {
    pub window: Arc<Window>,
    #[allow(unused)]
    handle: Handle,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    ui: EguiRenderer,
}

impl Gpu {
    #[instrument(skip_all, name = "GPU init")]
    pub async fn init(window: Arc<Window>, handle: Handle) -> Gresult<Self> {
        trace!("Creating instance");
        let mut instdesc = InstanceDescriptor::new_without_display_handle();
        instdesc.backends = Backends::DX12 | Backends::VULKAN;
        let instance = Instance::new(instdesc);
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
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;
        debug!("device & queue created");
        let width = window.inner_size().width;
        let height = window.inner_size().height;
        trace!("configuring surface");
        let config = surface.get_default_config(&adapter, width, height);
        let mut config = match config {
            Some(s) => s,
            None => {
                return Err(anyhow!(
                    "Failed graphics initialisation. No Compatible Surface was found"
                ));
            }
        };
        trace!("found compatible surface");
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &config);
        debug!("configured surface");
        let shader = device.create_shader_module(include_wgsl!("./shaders/triangle.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
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

        Ok(Self {
            window,
            handle,
            surface,
            adapter,
            device,
            queue,
            config,
            pipeline,
            ui,
        })
    }

    pub fn resize(&mut self, s: PhysicalSize<u32>) -> Gresult<()> {
        self.config.width = s.width;
        self.config.height = s.height;

        if self.config.width == 0 || self.config.height == 0 {
            return Ok(());
        };

        self.surface.configure(&self.device, &self.config);
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
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
}

#[derive(Debug)]
pub struct GpuCtx<'a> {
    pub window: Arc<Window>,
    pub adapter: &'a Adapter,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub config: &'a SurfaceConfiguration,
}
