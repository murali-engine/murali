use glam::{Vec2, Vec3, Vec4, vec2};
use std::sync::Arc;

use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::backend::renderer::vertex::text::TextVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::texture::TextureImage;
use crate::validation::ValidationError;
use std::path::Path;

/// How to render the parametric surface
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceRenderMode {
    /// Solid filled mesh
    Solid,
    /// Wireframe grid lines
    Wireframe,
    /// Both solid and wireframe
    SolidWithWireframe,
}

/// A parametric 3D surface defined by f(u, v) -> Vec3
///
/// Example: Sphere
/// ```ignore
/// let sphere = ParametricSurface::new(
///     (0.0, std::f32::consts::PI),      // u_range (theta)
///     (0.0, 2.0 * std::f32::consts::PI), // v_range (phi)
///     |u, v| {
///         let sin_u = u.sin();
///         Vec3::new(sin_u * v.cos(), sin_u * v.sin(), u.cos())
///     }
/// );
/// ```
#[derive(Clone)]
pub struct ParametricSurface {
    pub u_range: (f32, f32),
    pub v_range: (f32, f32),
    pub u_samples: usize,
    pub v_samples: usize,
    pub color: Vec4,
    pub f: Arc<dyn Fn(f32, f32) -> Vec3 + Send + Sync>,
    /// Animation progress: 0.0 = nothing drawn, 1.0 = fully drawn
    pub write_progress: f32,
    /// Render mode: Solid mesh or wireframe grid
    pub render_mode: SurfaceRenderMode,
    /// Optional color function based on height/parameter
    pub color_fn: Option<Arc<dyn Fn(f32) -> Vec4 + Send + Sync>>,
    /// Optional image texture mapped from the (u, v) parameter domain.
    pub texture: Option<Arc<TextureImage>>,
    /// Optional horizontal flip for textured surfaces.
    pub texture_flip_x: bool,
    /// Optional vertical flip for textured surfaces.
    pub texture_flip_y: bool,
}

impl ParametricSurface {
    pub fn new(
        u_range: (f32, f32),
        v_range: (f32, f32),
        f: impl Fn(f32, f32) -> Vec3 + Send + Sync + 'static,
    ) -> Self {
        Self {
            u_range,
            v_range,
            u_samples: 32,
            v_samples: 32,
            color: Vec4::new(0.44, 0.84, 0.71, 1.0),
            f: Arc::new(f),
            write_progress: 1.0,
            render_mode: SurfaceRenderMode::Solid,
            color_fn: None,
            texture: None,
            texture_flip_x: false,
            texture_flip_y: false,
        }
    }

    pub fn with_samples(mut self, u_samples: usize, v_samples: usize) -> Self {
        self.u_samples = u_samples.max(2);
        self.v_samples = v_samples.max(2);
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_write_progress(mut self, progress: f32) -> Self {
        self.write_progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn with_render_mode(mut self, mode: SurfaceRenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    pub fn with_color_fn(mut self, color_fn: impl Fn(f32) -> Vec4 + Send + Sync + 'static) -> Self {
        self.color_fn = Some(Arc::new(color_fn));
        self
    }

    pub fn with_texture(mut self, texture: TextureImage) -> Self {
        self.texture = Some(Arc::new(texture));
        self
    }

    pub fn with_texture_path(mut self, path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let texture = TextureImage::from_path(path.as_ref())?;
        self.texture = Some(Arc::new(texture));
        Ok(self)
    }

    pub fn with_texture_flip_x(mut self, flip: bool) -> Self {
        self.texture_flip_x = flip;
        self
    }

    pub fn with_texture_flip_y(mut self, flip: bool) -> Self {
        self.texture_flip_y = flip;
        self
    }

    fn validate_sampling(&self) -> Result<(), ValidationError> {
        validate_sample_count("u_samples", self.u_samples)?;
        validate_sample_count("v_samples", self.v_samples)?;
        validate_range("u_range", self.u_range)?;
        validate_range("v_range", self.v_range)?;
        if !self.write_progress.is_finite() {
            return Err(ValidationError::non_finite(
                "ParametricSurface",
                "write_progress",
                self.write_progress,
            ));
        }
        if !(0.0..=1.0).contains(&self.write_progress) {
            return Err(ValidationError::OutOfRange {
                component: "ParametricSurface",
                field: "write_progress",
                minimum: 0.0,
                maximum: 1.0,
                value: self.write_progress,
            });
        }
        Ok(())
    }

    fn emit_solid(&self, ctx: &mut ProjectionCtx) -> Result<(), ValidationError> {
        let mesh = if let Some(texture) = &self.texture {
            let (vertices, indices) = self.generate_textured_mesh()?;
            Mesh::from_textured_vertices(vertices, indices, texture.clone())
        } else {
            let (vertices, indices) = self.generate_mesh()?;
            Mesh::from_tessellation(vertices, indices)
        };
        ctx.emit(RenderPrimitive::Mesh(mesh));
        Ok(())
    }

    fn sample_position(&self, u: f32, v: f32) -> Result<Vec3, ValidationError> {
        let position = (self.f)(u, v);
        if !position.is_finite() {
            return Err(ValidationError::NonFiniteVector3 {
                component: "ParametricSurface",
                field: "f",
                x: position.x,
                y: position.y,
                z: position.z,
            });
        }
        Ok(position)
    }

    /// Generate mesh vertices and indices for the surface
    fn generate_mesh(&self) -> Result<(Vec<MeshVertex>, Vec<u32>), ValidationError> {
        self.validate_sampling()?;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if self.write_progress <= 0.0 {
            return Ok((vertices, indices));
        }

        let u_step = (self.u_range.1 - self.u_range.0) / (self.u_samples - 1) as f32;
        let v_step = (self.v_range.1 - self.v_range.0) / (self.v_samples - 1) as f32;

        // Calculate how many rows to draw based on write_progress
        let total_rows = self.u_samples;
        let rows_to_draw =
            ((self.write_progress * total_rows as f32).ceil() as usize).min(total_rows);

        // Generate vertices for rows we're drawing
        for i in 0..rows_to_draw {
            for j in 0..self.v_samples {
                let u = self.u_range.0 + i as f32 * u_step;
                let v = self.v_range.0 + j as f32 * v_step;
                let pos = self.sample_position(u, v)?;
                let color = self.get_color_for_point(pos);

                vertices.push(MeshVertex {
                    position: [pos.x, pos.y, pos.z],
                    color: [color.x, color.y, color.z, color.w],
                });
            }
        }

        // Generate indices (triangles) only for complete rows
        for i in 0..(rows_to_draw.saturating_sub(1)) {
            for j in 0..(self.v_samples - 1) {
                let a = (i * self.v_samples + j) as u32;
                let b = (i * self.v_samples + j + 1) as u32;
                let c = ((i + 1) * self.v_samples + j) as u32;
                let d = ((i + 1) * self.v_samples + j + 1) as u32;

                // First triangle
                indices.push(a);
                indices.push(b);
                indices.push(c);

                // Second triangle
                indices.push(b);
                indices.push(d);
                indices.push(c);
            }
        }

        Ok((vertices, indices))
    }

    fn generate_textured_mesh(&self) -> Result<(Vec<TextVertex>, Vec<u32>), ValidationError> {
        self.validate_sampling()?;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if self.write_progress <= 0.0 {
            return Ok((vertices, indices));
        }

        let u_step = (self.u_range.1 - self.u_range.0) / (self.u_samples - 1) as f32;
        let v_step = (self.v_range.1 - self.v_range.0) / (self.v_samples - 1) as f32;
        let total_rows = self.u_samples;
        let rows_to_draw =
            ((self.write_progress * total_rows as f32).ceil() as usize).min(total_rows);

        for i in 0..rows_to_draw {
            let u_progress = if self.u_samples > 1 {
                i as f32 / (self.u_samples - 1) as f32
            } else {
                0.0
            };
            for j in 0..self.v_samples {
                let v_progress = if self.v_samples > 1 {
                    j as f32 / (self.v_samples - 1) as f32
                } else {
                    0.0
                };
                let u = self.u_range.0 + i as f32 * u_step;
                let v = self.v_range.0 + j as f32 * v_step;
                let pos = self.sample_position(u, v)?;
                let mut uv_x = v_progress;
                let mut uv_y = 1.0 - u_progress;
                if self.texture_flip_x {
                    uv_x = 1.0 - uv_x;
                }
                if self.texture_flip_y {
                    uv_y = 1.0 - uv_y;
                }
                vertices.push(TextVertex {
                    position: [pos.x, pos.y, pos.z],
                    uv: [uv_x, uv_y],
                    color: [self.color.x, self.color.y, self.color.z, self.color.w],
                });
            }
        }

        for i in 0..(rows_to_draw.saturating_sub(1)) {
            for j in 0..(self.v_samples - 1) {
                let a = (i * self.v_samples + j) as u32;
                let b = (i * self.v_samples + j + 1) as u32;
                let c = ((i + 1) * self.v_samples + j) as u32;
                let d = ((i + 1) * self.v_samples + j + 1) as u32;

                indices.push(a);
                indices.push(b);
                indices.push(c);

                indices.push(b);
                indices.push(d);
                indices.push(c);
            }
        }

        Ok((vertices, indices))
    }

    /// Sample points on the surface for bounds calculation
    fn sample_points(&self) -> Result<Vec<Vec3>, ValidationError> {
        self.validate_sampling()?;
        let mut pts = Vec::new();
        let u_step = (self.u_range.1 - self.u_range.0) / (self.u_samples - 1) as f32;
        let v_step = (self.v_range.1 - self.v_range.0) / (self.v_samples - 1) as f32;

        for i in 0..self.u_samples {
            for j in 0..self.v_samples {
                let u = self.u_range.0 + i as f32 * u_step;
                let v = self.v_range.0 + j as f32 * v_step;
                pts.push(self.sample_position(u, v)?);
            }
        }

        Ok(pts)
    }

    /// Emit wireframe grid lines with optional color mapping
    fn emit_wireframe(&self, ctx: &mut ProjectionCtx) -> Result<(), ValidationError> {
        self.validate_sampling()?;
        if self.write_progress <= 0.0 {
            return Ok(());
        }

        let u_step = (self.u_range.1 - self.u_range.0) / (self.u_samples - 1) as f32;
        let v_step = (self.v_range.1 - self.v_range.0) / (self.v_samples - 1) as f32;

        let total_rows = self.u_samples;
        let rows_to_draw =
            ((self.write_progress * total_rows as f32).ceil() as usize).min(total_rows);

        // Generate grid points
        let mut grid_points = Vec::new();
        for i in 0..rows_to_draw {
            let mut row = Vec::new();
            for j in 0..self.v_samples {
                let u = self.u_range.0 + i as f32 * u_step;
                let v = self.v_range.0 + j as f32 * v_step;
                let pos = self.sample_position(u, v)?;
                row.push(pos);
            }
            grid_points.push(row);
        }

        // Calculate animation phase for line drawing
        // Phase 1 (0.0-0.5): Draw horizontal lines
        // Phase 2 (0.5-1.0): Draw vertical lines
        let line_progress = (self.write_progress * 2.0).min(1.0);
        let h_lines_progress = line_progress;
        let v_lines_progress = ((self.write_progress - 0.5) * 2.0).max(0.0);

        // Draw horizontal lines (u-direction) - drawn first
        let total_h_lines = grid_points.len();
        let h_lines_to_draw =
            ((h_lines_progress * total_h_lines as f32).ceil() as usize).min(total_h_lines);

        for row_idx in 0..h_lines_to_draw {
            let row = &grid_points[row_idx];
            for j in 0..row.len() - 1 {
                let start = row[j];
                let end = row[j + 1];
                let color = self.get_color_for_point(start);
                ctx.emit(RenderPrimitive::Line {
                    start,
                    end,
                    thickness: 0.02,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }

        // Draw vertical lines (v-direction) - drawn after horizontal
        if v_lines_progress > 0.0 {
            let total_v_lines = self.v_samples;
            let v_lines_to_draw =
                ((v_lines_progress * total_v_lines as f32).ceil() as usize).min(total_v_lines);

            for j in 0..v_lines_to_draw {
                for i in 0..grid_points.len() - 1 {
                    let start = grid_points[i][j];
                    let end = grid_points[i + 1][j];
                    let color = self.get_color_for_point(start);
                    ctx.emit(RenderPrimitive::Line {
                        start,
                        end,
                        thickness: 0.02,
                        color,
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
            }
        }
        Ok(())
    }

    /// Get color for a point based on color function or default color
    fn get_color_for_point(&self, point: Vec3) -> Vec4 {
        if let Some(color_fn) = &self.color_fn {
            // For surfaces, "height" is vertical displacement from the xz plane.
            color_fn(point.y)
        } else {
            self.color
        }
    }
}

impl Project for ParametricSurface {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let result = match self.render_mode {
            SurfaceRenderMode::Solid => self.emit_solid(ctx),
            SurfaceRenderMode::Wireframe => self.emit_wireframe(ctx),
            SurfaceRenderMode::SolidWithWireframe => {
                self.emit_solid(ctx).and_then(|()| self.emit_wireframe(ctx))
            }
        };
        if let Err(error) = result {
            ctx.report(error);
        }
    }
}

impl Bounded for ParametricSurface {
    fn local_bounds(&self) -> Bounds {
        let Ok(pts) = self.sample_points() else {
            return Bounds::default();
        };
        let mut min_xy = Vec2::splat(f32::INFINITY);
        let mut max_xy = Vec2::splat(f32::NEG_INFINITY);
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for p in pts {
            min_xy.x = min_xy.x.min(p.x);
            min_xy.y = min_xy.y.min(p.y);
            max_xy.x = max_xy.x.max(p.x);
            max_xy.y = max_xy.y.max(p.y);
            min_z = min_z.min(p.z);
            max_z = max_z.max(p.z);
        }

        let z_pad = max_z.abs().max(min_z.abs()) * 0.15;
        Bounds::new(min_xy - vec2(z_pad, z_pad), max_xy + vec2(z_pad, z_pad))
    }
}

fn validate_sample_count(field: &'static str, actual: usize) -> Result<(), ValidationError> {
    if actual < 2 {
        return Err(ValidationError::count_too_small(
            "ParametricSurface",
            field,
            2,
            actual,
        ));
    }
    Ok(())
}

fn validate_range(field: &'static str, range: (f32, f32)) -> Result<(), ValidationError> {
    if range.0.is_finite() && range.1.is_finite() && range.0 <= range.1 {
        return Ok(());
    }
    Err(ValidationError::InvalidRange {
        component: "ParametricSurface",
        field,
        start: range.0,
        end: range.1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directly_mutated_sample_counts_are_rejected_without_invalid_bounds() {
        let mut surface =
            ParametricSurface::new((0.0, 1.0), (0.0, 1.0), |u, v| Vec3::new(u, v, 0.0));
        surface.u_samples = 0;

        assert_eq!(surface.local_bounds(), Bounds::default());
        let mut ctx = ProjectionCtx::new(Default::default());
        surface.project(&mut ctx);
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::CountTooSmall {
                field: "u_samples",
                ..
            }
        ));
        assert!(ctx.primitives.is_empty());
    }

    #[test]
    fn non_finite_surface_output_is_reported() {
        let surface = ParametricSurface::new((0.0, 1.0), (0.0, 1.0), |_, _| Vec3::splat(f32::NAN));
        let mut ctx = ProjectionCtx::new(Default::default());
        surface.project(&mut ctx);
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::NonFiniteVector3 { field: "f", .. }
        ));
    }

    #[test]
    fn directly_mutated_non_finite_progress_is_reported() {
        let mut surface =
            ParametricSurface::new((0.0, 1.0), (0.0, 1.0), |u, v| Vec3::new(u, v, 0.0));
        surface.write_progress = f32::NAN;
        let mut ctx = ProjectionCtx::new(Default::default());
        surface.project(&mut ctx);
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::NonFinite {
                field: "write_progress",
                ..
            }
        ));
    }
}
