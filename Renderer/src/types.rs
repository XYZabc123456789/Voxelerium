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
    // --- Front face (+Z) ---
    GpuVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [0.0, 1.0],
    }, // 0
    GpuVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
    }, // 1
    GpuVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 0.0],
    }, // 2
    GpuVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // 3
    // --- Back face (-Z) ---
    GpuVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // 4
    GpuVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // 5
    GpuVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // 6
    GpuVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [0.0, 0.0],
    }, // 7
    // --- Top face (+Y) ---
    GpuVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    }, // 8
    GpuVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 1.0],
    }, // 9
    GpuVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // 10
    GpuVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [0.0, 0.0],
    }, // 11
    // --- Bottom face (-Y) ---
    GpuVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // 12
    GpuVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // 13
    GpuVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
    }, // 14
    GpuVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // 15
    // --- Right face (+X) ---
    GpuVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 1.0],
    }, // 16
    GpuVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // 17
    GpuVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // 18
    GpuVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // 19
    // --- Left face (-X) ---
    GpuVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // 20
    GpuVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
    }, // 21
    GpuVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [1.0, 0.0],
    }, // 22
    GpuVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [0.0, 0.0],
    }, // 23
];

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // Front
    4, 5, 6, 6, 7, 4, // Back
    8, 9, 10, 10, 11, 8, // Top
    12, 13, 14, 14, 15, 12, // Bottom
    16, 17, 18, 18, 19, 16, // Right
    20, 21, 22, 22, 23, 20, // Left
];

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
