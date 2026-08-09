// src/frontend/collection/graph/stream_lines.rs
//! Streamline visualization for vector fields
//! Shows flow paths that are tangent to the vector field at every point

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// Streamlines show the paths that particles would follow in a vector field
/// Each streamline is a curve that is tangent to the vector field at every point
pub struct StreamLines {
    /// Starting points for streamlines
    pub start_points: Vec<Vec2>,
    /// Function that returns a vector for a given position
    pub field_fn: Arc<dyn Fn(Vec2) -> Vec2 + Send + Sync>,
    /// Maximum number of steps to trace each streamline
    pub max_steps: usize,
    /// Step size for integration (smaller = more accurate but more points)
    pub step_size: f32,
    /// Color of the streamlines
    pub color: Vec4,
    /// Line thickness
    pub thickness: f32,
    /// Optional function to color streamlines based on position or magnitude
    pub color_fn: Option<Arc<dyn Fn(Vec2, f32) -> Vec4 + Send + Sync>>,
    /// Bounds to constrain streamlines
    pub bounds: Option<(Vec2, Vec2)>,
}

impl StreamLines {
    /// Create new streamlines from a set of starting points
    pub fn new<F>(start_points: Vec<Vec2>, field_fn: F) -> Self
    where
        F: Fn(Vec2) -> Vec2 + Send + Sync + 'static,
    {
        Self {
            start_points,
            field_fn: Arc::new(field_fn),
            max_steps: 1000,
            step_size: 0.05,
            color: Vec4::new(0.5, 0.7, 1.0, 0.8),
            thickness: 0.03,
            color_fn: None,
            bounds: None,
        }
    }

    /// Creates streamlines and validates authored seed and integration state.
    pub fn try_new<F>(start_points: Vec<Vec2>, field_fn: F) -> Result<Self, ValidationError>
    where
        F: Fn(Vec2) -> Vec2 + Send + Sync + 'static,
    {
        let streamlines = Self::new(start_points, field_fn);
        streamlines.validate()?;
        Ok(streamlines)
    }

    /// Create streamlines from a grid of starting points
    pub fn from_grid<F>(
        x_range: (f32, f32),
        y_range: (f32, f32),
        x_count: usize,
        y_count: usize,
        field_fn: F,
    ) -> Result<Self, ValidationError>
    where
        F: Fn(Vec2) -> Vec2 + Send + Sync + 'static,
    {
        validate_count("x_count", x_count, 2)?;
        validate_count("y_count", y_count, 2)?;
        validate_range("x_range", x_range)?;
        validate_range("y_range", y_range)?;

        let mut start_points = Vec::new();
        let dx = (x_range.1 - x_range.0) / (x_count - 1) as f32;
        let dy = (y_range.1 - y_range.0) / (y_count - 1) as f32;

        for i in 0..x_count {
            for j in 0..y_count {
                let x = x_range.0 + i as f32 * dx;
                let y = y_range.0 + j as f32 * dy;
                start_points.push(Vec2::new(x, y));
            }
        }

        Ok(Self::new(start_points, field_fn))
    }

    /// Set the color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Set a function to color streamlines based on position and magnitude
    pub fn with_color_fn<F>(mut self, color_fn: F) -> Self
    where
        F: Fn(Vec2, f32) -> Vec4 + Send + Sync + 'static,
    {
        self.color_fn = Some(Arc::new(color_fn));
        self
    }

    /// Set the line thickness
    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Set the step size for integration
    pub fn with_step_size(mut self, step_size: f32) -> Self {
        self.step_size = step_size;
        self
    }

    /// Set the maximum number of steps
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Set bounds to constrain streamlines
    pub fn with_bounds(mut self, min: Vec2, max: Vec2) -> Self {
        self.bounds = Some((min, max));
        self
    }

    /// Check if a point is within bounds
    fn in_bounds(&self, pos: Vec2) -> bool {
        if let Some((min, max)) = self.bounds {
            pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y
        } else {
            true
        }
    }

    /// Trace a single streamline using Euler integration
    fn validate(&self) -> Result<(), ValidationError> {
        if self.start_points.is_empty() {
            return Err(ValidationError::Empty {
                component: "StreamLines",
                field: "start_points",
            });
        }
        validate_count("max_steps", self.max_steps, 1)?;
        if !self.step_size.is_finite() {
            return Err(ValidationError::non_finite(
                "StreamLines",
                "step_size",
                self.step_size,
            ));
        }
        if self.step_size <= 0.0 {
            return Err(ValidationError::NonPositive {
                component: "StreamLines",
                field: "step_size",
                value: self.step_size,
            });
        }
        for point in &self.start_points {
            validate_vec2("start_points", *point)?;
        }
        if let Some((min, max)) = self.bounds {
            validate_bounds(min, max)?;
        }
        Ok(())
    }

    fn trace_streamline(&self, start: Vec2) -> Result<Vec<Vec2>, ValidationError> {
        let mut points = vec![start];
        let mut current = start;

        for _ in 0..self.max_steps {
            // Get the vector at the current position
            let vector = (self.field_fn)(current);
            if !vector.is_finite() {
                return Err(ValidationError::NonFiniteVector2 {
                    component: "StreamLines",
                    field: "field_fn",
                    x: vector.x,
                    y: vector.y,
                });
            }
            let magnitude = vector.length();

            // Stop if the vector is too small (stagnation point)
            if magnitude < 1e-6 {
                break;
            }

            // Normalize and scale by step size
            let step = vector.normalize() * self.step_size;
            let next = current + step;
            if !next.is_finite() {
                return Err(ValidationError::NonFiniteVector2 {
                    component: "StreamLines",
                    field: "integrated_position",
                    x: next.x,
                    y: next.y,
                });
            }

            // Stop if we go out of bounds
            if !self.in_bounds(next) {
                break;
            }

            points.push(next);
            current = next;
        }

        Ok(points)
    }
}

impl Bounded for StreamLines {
    fn local_bounds(&self) -> Bounds {
        if let Some((min, max)) = self.bounds {
            if validate_bounds(min, max).is_ok() {
                Bounds::new(min, max)
            } else {
                Bounds::default()
            }
        } else {
            let mut finite_points = self
                .start_points
                .iter()
                .copied()
                .filter(|point| point.is_finite());
            let Some(first) = finite_points.next() else {
                return Bounds::default();
            };
            finite_points.fold(Bounds::new(first, first), |bounds, point| {
                Bounds::new(bounds.min.min(point), bounds.max.max(point))
            })
        }
    }
}

impl Project for StreamLines {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }
        // Trace each streamline
        for start_point in &self.start_points {
            let points = match self.trace_streamline(*start_point) {
                Ok(points) => points,
                Err(error) => {
                    ctx.report(error);
                    return;
                }
            };

            // Draw the streamline as connected line segments
            for i in 0..points.len().saturating_sub(1) {
                let start = points[i];
                let end = points[i + 1];

                // Calculate color
                let color = if let Some(ref color_fn) = self.color_fn {
                    let vector = (self.field_fn)(start);
                    let magnitude = vector.length();
                    color_fn(start, magnitude)
                } else {
                    self.color
                };

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(start.x, start.y, 0.0),
                    end: Vec3::new(end.x, end.y, 0.0),
                    thickness: self.thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }
    }
}

/// Helper function to create evenly spaced starting points along a line
pub fn line_start_points(
    start: Vec2,
    end: Vec2,
    count: usize,
) -> Result<Vec<Vec2>, ValidationError> {
    validate_count("count", count, 2)?;
    validate_vec2("start", start)?;
    validate_vec2("end", end)?;
    let mut points = Vec::new();
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        points.push(start.lerp(end, t));
    }
    Ok(points)
}

/// Helper function to create starting points in a circle
pub fn circle_start_points(
    center: Vec2,
    radius: f32,
    count: usize,
) -> Result<Vec<Vec2>, ValidationError> {
    validate_count("count", count, 1)?;
    validate_vec2("center", center)?;
    if !radius.is_finite() {
        return Err(ValidationError::non_finite("StreamLines", "radius", radius));
    }
    if radius <= 0.0 {
        return Err(ValidationError::NonPositive {
            component: "StreamLines",
            field: "radius",
            value: radius,
        });
    }
    let mut points = Vec::new();
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        points.push(Vec2::new(x, y));
    }
    Ok(points)
}

fn validate_count(
    field: &'static str,
    actual: usize,
    minimum: usize,
) -> Result<(), ValidationError> {
    if actual < minimum {
        return Err(ValidationError::count_too_small(
            "StreamLines",
            field,
            minimum,
            actual,
        ));
    }
    Ok(())
}

fn validate_vec2(field: &'static str, value: Vec2) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::NonFiniteVector2 {
            component: "StreamLines",
            field,
            x: value.x,
            y: value.y,
        });
    }
    Ok(())
}

fn validate_range(field: &'static str, range: (f32, f32)) -> Result<(), ValidationError> {
    if !range.0.is_finite() {
        return Err(ValidationError::non_finite("StreamLines", field, range.0));
    }
    if !range.1.is_finite() {
        return Err(ValidationError::non_finite("StreamLines", field, range.1));
    }
    if range.0 > range.1 {
        return Err(ValidationError::InvalidRange {
            component: "StreamLines",
            field,
            start: range.0,
            end: range.1,
        });
    }
    Ok(())
}

fn validate_bounds(min: Vec2, max: Vec2) -> Result<(), ValidationError> {
    if min.is_finite() && max.is_finite() && min.cmple(max).all() {
        return Ok(());
    }
    Err(ValidationError::InvalidBounds {
        component: "StreamLines",
        field: "bounds",
        min_x: min.x,
        min_y: min.y,
        max_x: max.x,
        max_y: max.y,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_and_line_helpers_reject_counts_that_cannot_be_sampled() {
        assert!(StreamLines::from_grid((0.0, 1.0), (0.0, 1.0), 1, 2, |_| Vec2::X).is_err());
        assert!(line_start_points(Vec2::ZERO, Vec2::X, 1).is_err());
        assert!(circle_start_points(Vec2::ZERO, 1.0, 0).is_err());
    }

    #[test]
    fn empty_and_non_finite_state_has_finite_bounds_and_diagnostics() {
        let empty = StreamLines::new(Vec::new(), |_| Vec2::X);
        assert_eq!(empty.local_bounds(), Bounds::default());
        let mut ctx = ProjectionCtx::new(Default::default());
        empty.project(&mut ctx);
        assert!(matches!(ctx.diagnostics[0], ValidationError::Empty { .. }));

        let invalid = StreamLines::new(vec![Vec2::ZERO], |_| Vec2::splat(f32::NAN));
        let mut ctx = ProjectionCtx::new(Default::default());
        invalid.project(&mut ctx);
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::NonFiniteVector2 {
                field: "field_fn",
                ..
            }
        ));
    }
}
