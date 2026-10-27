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

//voxelerium/input/mod.rs
use super::*;

mod keyboard;
pub use keyboard::*;

mod mouse;
pub use mouse::*;

#[allow(unused)]
#[derive(Debug)]
pub struct InputBuffer<T, const N: usize> {
    entries: [Option<T>; N],
    head: usize,
    len: usize,
}
#[allow(unused)]
impl<T, const N: usize> InputBuffer<T, N> {
    pub fn new() -> Self {
        assert!(N > 0);

        Self {
            entries: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, event: T) {
        self.head = (self.head + N - 1) % N;

        self.entries[self.head] = Some(event);

        if self.len < N {
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let event = self.entries[self.head].take();

        self.len -= 1;

        if self.len > 0 {
            self.head = (self.head + 1) % N;
        }

        event
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
        self.head = 0;
        self.len = 0;
    }
}

impl<T, const N: usize> Default for InputBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
