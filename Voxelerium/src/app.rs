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

//voxelerium/app.rs
use super::*;

const ALPHA: f64 = 0.25;

#[derive(Debug)]
pub struct App {
    window: Option<Arc<Window>>,
    focus: bool,
    gpu: Option<Gpu>,
    handle: Handle,
    last_frame: Instant,
    fps: f64,
    time_accum: f64,
    disp_fps: f64,
    keyboard_input: KeyboardInput<16>,
    mouse_input: MouseInput<16>,
}

impl App {
    pub fn new(handle: Handle) -> Self {
        Self {
            window: None,
            focus: false,
            gpu: None,
            handle,
            last_frame: Instant::now(),
            fps: 60.0,
            time_accum: 0.0,
            disp_fps: 0.0,
            keyboard_input: KeyboardInput::new(),
            mouse_input: MouseInput::new(),
        }
    }
    #[inline]
    pub fn window(&self) -> &Arc<Window> {
        self.window.as_ref().expect("Window not initialised")
    }
    #[inline]
    pub fn gpu(&mut self) -> &mut Gpu {
        self.gpu.as_mut().expect("GPU not initialised")
    }
    #[inline]
    #[allow(unused)]
    pub fn gpu_ctx(&self) -> GpuCtx<'_> {
        self.gpu.as_ref().expect("GPU not initialised").get_ctx()
    }
    fn ui_handler(&self, uimsg: Rc<[UiMessage]>) -> Gresult<()> {
        for msg in uimsg.iter() {
            match msg {
                UiMessage::SpawnVoxel => info!("Voxel spawn called"),
            }
        }
        Ok(())
    }
}

impl ApplicationHandler for App {
    #[instrument(
        skip_all,
        name = "window_creation"
        fields(
            window_title = "Voxelerium",
            gpu_initialized = tracing::field::Empty
        )
    )]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        debug!("Creating Window...");
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Voxelerium"))
                .expect("Could not create window"),
        ));
        trace!("Window creared");
        debug!("Setting up GPU...");
        self.gpu = Some(
            self.handle
                .block_on(Gpu::init(self.window().clone(), self.handle.clone()))
                .expect("GPU initialization failed"),
        );
        tracing::Span::current().record("gpu_initialized", true);
        trace!("GPU Setup Complete");
        trace!("Initialising delta time");
        self.last_frame = Instant::now();
        self.disp_fps = 60.0;
        info!("WINDOW INIT COMPLETE")
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let response = self.gpu().pass_to_ui(&event);
        match event {
            WindowEvent::CloseRequested => {
                info!("Closing...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f64();
                self.last_frame = now;
                let raw_fps = 1.0 / dt;
                self.time_accum += dt;
                self.fps = (raw_fps * ALPHA) + (self.fps * (1.0 - ALPHA));
                if self.time_accum >= 1.0 {
                    self.disp_fps = self.fps;
                    self.time_accum = 0.0;
                }
                let _span = tracing::trace_span!("render_frame", focused = self.focus).entered();
                let r = self
                    .gpu
                    .as_mut()
                    .expect("GPU not initialised")
                    .render(self.focus, self.disp_fps);
                match r {
                    Ok(m) => {
                        let r = self.ui_handler(m);
                        match r {
                            Ok(_) => {}
                            Err(e) => panic!("{:#?}", e),
                        }
                    }
                    Err(e) => panic!("{:#?}", e),
                }
                self.window().request_redraw();
            }
            WindowEvent::Resized(s) => {
                self.gpu().resize(s).expect("Window resize failed");
            }
            WindowEvent::Focused(f) => {
                self.focus = f;
            }
            _ => (),
        }
        if response.consumed {
            return;
        }
        self.keyboard_input.process_event(&event);
        self.mouse_input.handle_window_event(&event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.mouse_input.handle_device_event(&event);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        warn!("EXITING VOXELERIUM");
    }
}
