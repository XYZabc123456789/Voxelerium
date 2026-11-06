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

//renderer/ui/mod.rs
use super::*;

pub struct EguiRenderer {
    pub context: egui::Context,
    pub state: State,
    pub renderer: Renderer,
}

impl core::fmt::Debug for EguiRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiRenderer")
            .field("context", &self.context)
            .finish()
    }
}

impl EguiRenderer {
    pub fn new(
        window: &Arc<winit::window::Window>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let context = egui::Context::default();

        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            context,
            state,
            renderer,
        }
    }
    pub fn draw(ctx: &mut Ui, fps: f64) -> Rc<[UiMessage]> {
        let mut uimsg = Vec::new();
        egui::Window::new("Debug").show(ctx, |ui| {
            ui.label(format!("FPS: {:.0}", fps));
            if ui.button("Spawn voxel").clicked() {
                uimsg.push(UiMessage::SpawnVoxel);
            }
        });

        egui::Area::new(egui::Id::unique("crosshair"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .interactable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("+").size(32.0));
            });

        uimsg.into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UiMessage {
    SpawnVoxel,
}
