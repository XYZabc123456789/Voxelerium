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

@vertex fn vs(
    @builtin(vertex_index) vertexIndex: u32
) -> @builtin(position) vec4f {
    let pos = array(
        vec2f(0.0, 0.5),  // top center
        vec2f(-0.5, -0.5),  // bottom left
        vec2f(0.5, -0.5)// bottom right
    );

    return vec4f(pos[vertexIndex], 0.0, 1.0);
}
 
@fragment fn fs() -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0);
}