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

//renderer/types.rs
use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: glam::Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl_from_into! {Vertex -> GpuVertex {position, tex_coords}}
impl_from_into! {GpuVertex -> Vertex {position, tex_coords}}

pub const VERTICES: &[GpuVertex] = &[
    GpuVertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    },
    GpuVertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    },
    GpuVertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    },
    GpuVertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    },
    GpuVertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    },
];

pub const INDICES: &[u16] = [[0, 1, 4], [1, 2, 4], [2, 3, 4]].as_flattened();

impl GpuVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    view: Affine3A,
    projection: Mat4,
}

#[allow(unused)]
impl Camera {
    #[inline]
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        fovy: f32,
        aspect: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            fovy,
            aspect,
            znear,
            zfar,
            view: glam::camera::rh::view::look_at_affine3a(eye, target, up),
            projection: glam::camera::rh::proj::directx::perspective(fovy, aspect, znear, zfar),
        }
    }

    #[inline]
    pub fn update_view(&mut self) {
        self.view = glam::camera::rh::view::look_at_affine3a(self.eye, self.target, self.up);
    }

    #[inline]
    pub fn update_projection(&mut self) {
        self.projection = glam::camera::rh::proj::directx::perspective(
            self.fovy,
            self.aspect,
            self.znear,
            self.zfar,
        );
    }

    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from(self.view)
    }

    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection
    }

    #[inline]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection * Mat4::from(self.view)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GpuCamera {
    pub view_projection: [[f32; 4]; 4],
}

impl From<&Camera> for GpuCamera {
    #[inline]
    fn from(camera: &Camera) -> Self {
        Self {
            view_projection: camera.view_projection_matrix().to_cols_array_2d(),
        }
    }
}
