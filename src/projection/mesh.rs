use crate::backend::renderer::vertex::{mesh::MeshVertex, text::TextVertex};
use crate::resource::texture::TextureImage;
use glam::{Vec3, Vec4};
use std::sync::Arc;

/// CPU-side vertex storage emitted by projection.
#[derive(Debug, Clone)]
pub enum MeshData {
    Empty,
    Mesh(Vec<MeshVertex>),
    Textured(Vec<TextVertex>),
    Text(Vec<TextVertex>),
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use glam::Vec4;

    #[test]
    fn generated_mesh_indices_can_address_vertices_beyond_u16() {
        let mesh = Mesh::circle(1.0, u16::MAX as u32 + 1, Vec4::ONE);

        assert_eq!(mesh.indices.iter().copied().max(), Some(65_536));
    }
}

/// CPU-side mesh description.
///
/// This lives in the projection layer because it is part of the
/// backend-neutral output produced before GPU upload.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub data: MeshData,
    pub indices: Vec<u32>,
    pub texture: Option<Arc<TextureImage>>,
}

impl Mesh {
    pub fn from_tessellation(vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Arc<Self> {
        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    pub fn from_textured_vertices(
        vertices: Vec<TextVertex>,
        indices: Vec<u32>,
        texture: Arc<TextureImage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            data: MeshData::Textured(vertices),
            indices,
            texture: Some(texture),
        })
    }

    pub fn empty() -> Self {
        Self {
            data: MeshData::Empty,
            indices: Vec::new(),
            texture: None,
        }
    }

    /// Square in XY plane centered at origin.
    pub fn square(size: f32, color: impl Into<crate::projection::style::ColorSource>) -> Arc<Self> {
        Self::rectangle(size, size, color)
    }

    /// Axis-aligned rectangle in XY plane centered at origin.
    pub fn rectangle(
        width: f32,
        height: f32,
        color: impl Into<crate::projection::style::ColorSource>,
    ) -> Arc<Self> {
        let hw = width * 0.5;
        let hh = height * 0.5;
        let color_source = color.into();

        let get_color = |pos: [f32; 3]| -> [f32; 4] {
            match &color_source {
                crate::projection::style::ColorSource::Solid(c) => [c[0], c[1], c[2], c[3]],
                crate::projection::style::ColorSource::LinearGradient { start, end, stops } => {
                    let c =
                        Self::evaluate_gradient(glam::vec2(pos[0], pos[1]), *start, *end, stops);
                    [c[0], c[1], c[2], c[3]]
                }
            }
        };

        let vertices = vec![
            MeshVertex {
                position: [-hw, -hh, 0.0],
                color: get_color([-hw, -hh, 0.0]),
            },
            MeshVertex {
                position: [hw, -hh, 0.0],
                color: get_color([hw, -hh, 0.0]),
            },
            MeshVertex {
                position: [hw, hh, 0.0],
                color: get_color([hw, hh, 0.0]),
            },
            MeshVertex {
                position: [-hw, hh, 0.0],
                color: get_color([-hw, hh, 0.0]),
            },
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    /// Axis-aligned cube centered at origin.
    pub fn cube(size: f32, color: Vec4) -> Arc<Self> {
        let h = size * 0.5;

        let vertices = vec![
            MeshVertex {
                position: [-h, -h, -h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [h, -h, -h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [h, h, -h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [-h, h, -h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [-h, -h, h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [h, -h, h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [h, h, h],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [-h, h, h],
                color: [color[0], color[1], color[2], color[3]],
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 0, 4, 7, 7, 3, 0, 1, 5, 6, 6, 2, 1, 0, 1, 5, 5, 4,
            0, 3, 2, 6, 6, 7, 3,
        ];

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    /// Triangle fan circle in XY plane.
    pub fn circle(
        radius: f32,
        segments: u32,
        color: impl Into<crate::projection::style::ColorSource>,
    ) -> Arc<Self> {
        let seg = segments.max(3);
        let mut vertices = Vec::with_capacity((seg + 1) as usize);
        let color_source = color.into();

        let get_color = |pos: [f32; 3]| -> [f32; 4] {
            match &color_source {
                crate::projection::style::ColorSource::Solid(c) => [c[0], c[1], c[2], c[3]],
                crate::projection::style::ColorSource::LinearGradient { start, end, stops } => {
                    let c =
                        Self::evaluate_gradient(glam::vec2(pos[0], pos[1]), *start, *end, stops);
                    [c[0], c[1], c[2], c[3]]
                }
            }
        };

        vertices.push(MeshVertex {
            position: [0.0, 0.0, 0.0],
            color: get_color([0.0, 0.0, 0.0]),
        });

        for i in 0..seg {
            let t = (i as f32 / seg as f32) * std::f32::consts::TAU;
            let px = radius * t.cos();
            let py = radius * t.sin();
            vertices.push(MeshVertex {
                position: [px, py, 0.0],
                color: get_color([px, py, 0.0]),
            });
        }

        let mut indices = Vec::with_capacity((seg * 3) as usize);
        for i in 0..seg {
            indices.push(0);
            indices.push(i + 1);
            indices.push(if i + 2 <= seg { i + 2 } else { 1 });
        }

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    /// Triangle fan ellipse in XY plane.
    pub fn ellipse(
        radius_x: f32,
        radius_y: f32,
        segments: u32,
        color: impl Into<crate::projection::style::ColorSource>,
    ) -> Arc<Self> {
        let seg = segments.max(3);
        let mut vertices = Vec::with_capacity((seg + 1) as usize);
        let color_source = color.into();

        let get_color = |pos: [f32; 3]| -> [f32; 4] {
            match &color_source {
                crate::projection::style::ColorSource::Solid(c) => [c[0], c[1], c[2], c[3]],
                crate::projection::style::ColorSource::LinearGradient { start, end, stops } => {
                    let c =
                        Self::evaluate_gradient(glam::vec2(pos[0], pos[1]), *start, *end, stops);
                    [c[0], c[1], c[2], c[3]]
                }
            }
        };

        vertices.push(MeshVertex {
            position: [0.0, 0.0, 0.0],
            color: get_color([0.0, 0.0, 0.0]),
        });

        for i in 0..seg {
            let t = (i as f32 / seg as f32) * std::f32::consts::TAU;
            let px = radius_x * t.cos();
            let py = radius_y * t.sin();
            vertices.push(MeshVertex {
                position: [px, py, 0.0],
                color: get_color([px, py, 0.0]),
            });
        }

        let mut indices = Vec::with_capacity((seg * 3) as usize);
        for i in 0..seg {
            indices.push(0);
            indices.push(i + 1);
            indices.push(if i + 2 <= seg { i + 2 } else { 1 });
        }

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    /// Triangle fan polygon in XY plane.
    /// Assumes vertices are convex and provided in order.
    pub fn polygon(
        vertices_2d: Vec<glam::Vec2>,
        color: impl Into<crate::projection::style::ColorSource>,
    ) -> Arc<Self> {
        let n = vertices_2d.len();
        if n < 3 {
            return Arc::new(Self::empty());
        }

        let mut vertices = Vec::with_capacity(n);
        let color_source = color.into();

        let get_color = |pos: [f32; 2]| -> [f32; 4] {
            match &color_source {
                crate::projection::style::ColorSource::Solid(c) => [c[0], c[1], c[2], c[3]],
                crate::projection::style::ColorSource::LinearGradient { start, end, stops } => {
                    let c =
                        Self::evaluate_gradient(glam::vec2(pos[0], pos[1]), *start, *end, stops);
                    [c[0], c[1], c[2], c[3]]
                }
            }
        };

        for p in &vertices_2d {
            vertices.push(MeshVertex {
                position: [p.x, p.y, 0.0],
                color: get_color([p.x, p.y]),
            });
        }

        let mut indices = Vec::with_capacity((n - 2) * 3);
        for i in 1..(n - 1) {
            indices.push(0);
            indices.push(i as u32);
            indices.push((i + 1) as u32);
        }

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    /// Line rendered as a thin rectangle using triangles.
    pub fn line(start: [f32; 3], end: [f32; 3], thickness: f32, color: Vec4) -> Arc<Self> {
        let sx = start[0];
        let sy = start[1];
        let sz = start[2];

        let ex = end[0];
        let ey = end[1];
        let ez = end[2];

        let dx = ex - sx;
        let dy = ey - sy;
        let len = (dx * dx + dy * dy).sqrt();

        let (nx, ny) = if len > 1e-6 {
            (-dy / len, dx / len)
        } else {
            (0.0, 1.0)
        };

        let ox = nx * thickness * 0.5;
        let oy = ny * thickness * 0.5;

        let vertices = vec![
            MeshVertex {
                position: [sx + ox, sy + oy, sz],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [sx - ox, sy - oy, sz],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [ex - ox, ey - oy, ez],
                color: [color[0], color[1], color[2], color[3]],
            },
            MeshVertex {
                position: [ex + ox, ey + oy, ez],
                color: [color[0], color[1], color[2], color[3]],
            },
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Arc::new(Self {
            data: MeshData::Mesh(vertices),
            indices,
            texture: None,
        })
    }

    pub fn vertex_kind(&self) -> &'static str {
        match self.data {
            MeshData::Empty => "Empty",
            MeshData::Mesh(_) => "Mesh",
            MeshData::Textured(_) => "Textured",
            MeshData::Text(_) => "Text",
        }
    }

    pub fn translated(&self, offset: Vec3) -> Arc<Self> {
        match &self.data {
            MeshData::Empty => Arc::new(self.clone()),
            MeshData::Mesh(vertices) => Arc::new(Self {
                data: MeshData::Mesh(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] += offset.x;
                            v.position[1] += offset.y;
                            v.position[2] += offset.z;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Textured(vertices) => Arc::new(Self {
                data: MeshData::Textured(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] += offset.x;
                            v.position[1] += offset.y;
                            v.position[2] += offset.z;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Text(vertices) => Arc::new(Self {
                data: MeshData::Text(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] += offset.x;
                            v.position[1] += offset.y;
                            v.position[2] += offset.z;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
        }
    }

    pub fn scaled(&self, scale: f32) -> Arc<Self> {
        match &self.data {
            MeshData::Empty => Arc::new(self.clone()),
            MeshData::Mesh(vertices) => Arc::new(Self {
                data: MeshData::Mesh(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] *= scale;
                            v.position[1] *= scale;
                            v.position[2] *= scale;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Textured(vertices) => Arc::new(Self {
                data: MeshData::Textured(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] *= scale;
                            v.position[1] *= scale;
                            v.position[2] *= scale;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Text(vertices) => Arc::new(Self {
                data: MeshData::Text(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.position[0] *= scale;
                            v.position[1] *= scale;
                            v.position[2] *= scale;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
        }
    }

    pub fn with_opacity(&self, alpha: f32) -> Arc<Self> {
        // Optimization: return Arc to self if opacity is 1.0
        if (alpha - 1.0).abs() < 1e-6 {
            return Arc::new(self.clone());
        }

        match &self.data {
            MeshData::Empty => Arc::new(self.clone()),
            MeshData::Mesh(vertices) => Arc::new(Self {
                data: MeshData::Mesh(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.color[3] *= alpha;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Textured(vertices) => Arc::new(Self {
                data: MeshData::Textured(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.color[3] *= alpha;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
            MeshData::Text(vertices) => Arc::new(Self {
                data: MeshData::Text(
                    vertices
                        .iter()
                        .map(|v| {
                            let mut v = *v;
                            v.color[3] *= alpha;
                            v
                        })
                        .collect(),
                ),
                indices: self.indices.clone(),
                texture: self.texture.clone(),
            }),
        }
    }

    /// Evaluates a linear gradient at a given 2D point.
    pub fn evaluate_gradient(
        point: glam::Vec2,
        start: glam::Vec2,
        end: glam::Vec2,
        stops: &[(f32, Vec4)],
    ) -> Vec4 {
        if stops.is_empty() {
            return Vec4::ONE;
        }
        if stops.len() == 1 {
            return stops[0].1;
        }

        let d = end - start;
        let p = point - start;
        let t = (p.dot(d) / d.length_squared()).clamp(0.0, 1.0);

        // Find the two stops that t is between
        let mut lower = &stops[0];
        let mut upper = &stops[stops.len() - 1];

        for i in 0..stops.len() - 1 {
            if t >= stops[i].0 && t <= stops[i + 1].0 {
                lower = &stops[i];
                upper = &stops[i + 1];
                break;
            }
        }

        let range = upper.0 - lower.0;
        if range < 1e-6 {
            upper.1
        } else {
            lower.1.lerp(upper.1, (t - lower.0) / range)
        }
    }
}
