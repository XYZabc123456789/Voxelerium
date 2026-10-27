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

//voxelerium/input/keyboard.rs
use super::*;
#[allow(unused)]
#[derive(Debug)]
pub struct KeyboardInput<const N: usize> {
    pub events: InputBuffer<Key, N>,
}

#[allow(unused)]
impl<const N: usize> KeyboardInput<N> {
    pub fn new() -> Self {
        Self {
            events: InputBuffer::default()
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        let WindowEvent::KeyboardInput { event, .. } = event else {
            return;
        };

        let key = event.logical_key.clone();

        let raw_event = match event.state {
            ElementState::Pressed => key,
            ElementState::Released => return,
        };

        self.events.push(raw_event);
    }
}
