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

// utils lib.rs
use winit::event::*;
use winit::keyboard::*;

#[allow(unused)]
pub use macros::*;
pub type Gresult<T> = anyhow::Result<T>;

#[macro_export]
macro_rules! impl_from_into {
    ($src:ident -> $dst:ident { $($field:ident),* $(,)? }) => {
        impl From<$src> for $dst {
            fn from(value: $src) -> Self {
                Self {
                    $( $field: value.$field.into(), )*
                }
            }
        }
    };
}

pub mod input;
