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
#[derive(Debug)]
pub struct App {
    focus: bool,
    gpu: Option<Gpu>,
    handle: Handle,
    keyboard_input: KeyboardInput<16>,
    mouse_input: MouseInput<16>,
}

impl App {
    pub fn new(handle: Handle) -> Self {
        Self {
            focus: false,
            gpu: None,
            handle,
            keyboard_input: KeyboardInput::new(),
            mouse_input: MouseInput::new(),
        }
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
        let window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Voxelerium"))
                .expect("Could not create window"),
        ));
        trace!("Window creared");
        debug!("Setting up GPU...");
        self.gpu = Some(
            self.handle
                .block_on(Gpu::init(
                    window.as_ref().unwrap().clone(),
                    self.handle.clone(),
                ))
                .expect("GPU initialization failed"),
        );
        tracing::Span::current().record("gpu_initialized", true);
        trace!("GPU Setup Complete")
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Closing...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let _span = tracing::trace_span!("render_frame", focused = self.focus).entered();
                let r = self
                    .gpu
                    .as_mut()
                    .expect("GPU not initialised")
                    .render(self.focus);
                match r {
                    Ok(_) => {}
                    Err(e) => panic!("{:#?}", e),
                }
                self.gpu.as_ref().unwrap().window.request_redraw();
            }
            WindowEvent::Resized(s) => {
                self.gpu
                    .as_mut()
                    .expect("GPU not initialised")
                    .resize(s)
                    .expect("Window resize failed");
            }
            WindowEvent::Focused(f) => {
                self.focus = f;
            }
            _ => (),
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
