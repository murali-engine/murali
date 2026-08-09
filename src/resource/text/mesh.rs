use crate::backend::renderer::vertex::text::TextVertex;
use crate::projection::{Mesh, MeshData};
use crate::resource::text::atlas::GlyphAtlas;
use crate::resource::text::layout::LabelLayout;
use glam::Vec4;
use std::sync::Arc;

/// Generates a GPU-ready Mesh from a high-level LabelLayout.
/// This acts as the "Baker" for text geometry.
pub fn build_label_mesh(layout: &LabelLayout, atlas: &GlyphAtlas, color: Vec4) -> Arc<Mesh> {
    let mut vertices: Vec<TextVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut index_offset: u32 = 0;

    for glyph in &layout.glyphs {
        let info = match atlas.glyphs.get(&glyph.ch) {
            Some(info) => info,
            None => continue,
        };

        let x0 = glyph.x + glyph.bearing_x;
        let x1 = x0 + glyph.width;
        let y0 = glyph.bearing_y;
        let y1 = y0 + glyph.height;

        let uv_min = info.uv_min;
        let uv_max = info.uv_max;
        let x_offset = -layout.width * 0.5;
        let y_offset = -(layout.ascent - layout.height * 0.5);

        vertices.extend_from_slice(&[
            TextVertex {
                position: [x0 + x_offset, y0 + y_offset, 0.0],
                uv: [uv_min[0], uv_max[1]],
                color: color.into(),
            },
            TextVertex {
                position: [x1 + x_offset, y0 + y_offset, 0.0],
                uv: [uv_max[0], uv_max[1]],
                color: color.into(),
            },
            TextVertex {
                position: [x1 + x_offset, y1 + y_offset, 0.0],
                uv: [uv_max[0], uv_min[1]],
                color: color.into(),
            },
            TextVertex {
                position: [x0 + x_offset, y1 + y_offset, 0.0],
                uv: [uv_min[0], uv_min[1]],
                color: color.into(),
            },
        ]);

        indices.extend_from_slice(&[
            index_offset,
            index_offset + 1,
            index_offset + 2,
            index_offset,
            index_offset + 2,
            index_offset + 3,
        ]);

        index_offset += 4;
    }

    Arc::new(Mesh {
        data: MeshData::Text(vertices),
        indices,
        texture: None,
    })
}
