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

//voxelerium/input/mouse.rs
use super::*;

#[derive(Debug)]
pub enum ScrollUnit {
    Line,
    Pixel,
}

#[allow(unused)]
#[derive(Debug)]
pub enum RawMouseEvent {
    Button(MouseButton),
    Scrolled { x: f64, y: f64, unit: ScrollUnit },
}

#[derive(Default, Debug)]
#[allow(unused)]
pub struct MouseMotion {
    pub dx: f64,
    pub dy: f64,
}

#[allow(unused)]
impl MouseMotion {
    #[inline]
    pub fn add(&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    #[inline]
    pub fn take_delta(&mut self) -> (f64, f64) {
        let delta = (self.dx, self.dy);
        self.dx = 0.0;
        self.dy = 0.0;
        delta
    }

    #[inline]
    pub fn clear(&mut self) {
        self.dx = 0.0;
        self.dy = 0.0;
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct MouseInput<const N: usize> {
    pub events: InputBuffer<RawMouseEvent, N>,
    pub motion: MouseMotion,
}

#[allow(unused)]
impl<const N: usize> MouseInput<N> {
    pub fn new() -> Self {
        Self {
            events: InputBuffer::new(),
            motion: MouseMotion::default(),
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                let event = match state {
                    ElementState::Pressed => RawMouseEvent::Button(*button),
                    ElementState::Released => return,
                };

                self.events.push(event);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let event = match delta {
                    MouseScrollDelta::LineDelta(x, y) => RawMouseEvent::Scrolled {
                        x: *x as f64,
                        y: *y as f64,
                        unit: ScrollUnit::Line,
                    },

                    MouseScrollDelta::PixelDelta(pos) => RawMouseEvent::Scrolled {
                        x: pos.x,
                        y: pos.y,
                        unit: ScrollUnit::Pixel,
                    },
                };

                self.events.push(event);
            }

            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.motion.add(delta.0, delta.1);
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.motion.clear();
    }
}
