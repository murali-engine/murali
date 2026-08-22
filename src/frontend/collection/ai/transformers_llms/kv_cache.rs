use glam::{Vec2, Vec3, Vec4, vec2, vec3};

use crate::frontend::collection::common::tensor::TensorSnapshot;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

const COMPONENT: &str = "KvCacheView";
const MAX_SLOTS: usize = 16;
const MAX_FEATURES: usize = 16;

/// Semantic key/value cache state for one transformer head or authored feature slice.
#[derive(Debug, Clone)]
pub struct KvCacheView {
    pub keys: TensorSnapshot,
    pub values: TensorSnapshot,
    pub token_axis_id: String,
    pub feature_axis_id: String,
    /// Continuous occupied-slot count. Integer values represent stable cache states.
    pub occupancy: f32,
    pub cell_size: Vec2,
    pub panel_gap: f32,
    pub padding: f32,
    pub text_color: Vec4,
    pub empty_color: Vec4,
    pub zero_color: Vec4,
    pub negative_color: Vec4,
    pub key_color: Vec4,
    pub value_color: Vec4,
    pub active_color: Vec4,
    pub grid_color: Vec4,
}

impl KvCacheView {
    pub fn try_new(
        keys: TensorSnapshot,
        values: TensorSnapshot,
        token_axis_id: impl Into<String>,
        feature_axis_id: impl Into<String>,
        occupied_tokens: usize,
    ) -> Result<Self, ValidationError> {
        let view = Self {
            keys,
            values,
            token_axis_id: token_axis_id.into(),
            feature_axis_id: feature_axis_id.into(),
            occupancy: occupied_tokens as f32,
            cell_size: vec2(0.48, 0.40),
            panel_gap: 0.72,
            padding: 0.34,
            text_color: Vec4::new(0.94, 0.97, 1.0, 1.0),
            empty_color: Vec4::new(0.105, 0.125, 0.15, 1.0),
            zero_color: Vec4::new(0.14, 0.17, 0.21, 1.0),
            negative_color: Vec4::new(0.90, 0.36, 0.48, 1.0),
            key_color: Vec4::new(0.33, 0.78, 0.72, 1.0),
            value_color: Vec4::new(0.36, 0.64, 0.91, 1.0),
            active_color: Vec4::new(0.96, 0.72, 0.35, 1.0),
            grid_color: Vec4::new(0.42, 0.47, 0.53, 1.0),
        };
        view.validate()?;
        Ok(view)
    }

    pub fn capacity(&self) -> usize {
        self.keys.shape[self.token_axis_index(&self.keys).unwrap_or(0)]
    }

    pub fn feature_count(&self) -> usize {
        self.keys.shape[self.feature_axis_index(&self.keys).unwrap_or(1)]
    }

    pub fn occupied_tokens(&self) -> usize {
        self.occupancy.round().clamp(0.0, self.capacity() as f32) as usize
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.keys.validate()?;
        self.values.validate()?;
        for snapshot in [&self.keys, &self.values] {
            if snapshot.rank() != 2 {
                return Err(ValidationError::RankMismatch {
                    component: COMPONENT,
                    field: "cache tensor",
                    expected: 2,
                    actual: snapshot.rank(),
                });
            }
        }
        if self.keys.id == self.values.id {
            return Err(ValidationError::DuplicateIdentifier {
                component: COMPONENT,
                field: "key and value tensors",
                value: self.keys.id.clone(),
            });
        }
        if self.token_axis_id == self.feature_axis_id {
            return Err(ValidationError::DuplicateIdentifier {
                component: COMPONENT,
                field: "cache axes",
                value: self.token_axis_id.clone(),
            });
        }
        let key_token = self.token_axis_index(&self.keys)?;
        let key_feature = self.feature_axis_index(&self.keys)?;
        let value_token = self.token_axis_index(&self.values)?;
        let value_feature = self.feature_axis_index(&self.values)?;
        let key_token_axis = &self.keys.axes[key_token];
        let value_token_axis = &self.values.axes[value_token];
        let key_feature_axis = &self.keys.axes[key_feature];
        let value_feature_axis = &self.values.axes[value_feature];
        if key_token_axis.element_ids != value_token_axis.element_ids
            || key_token_axis.element_labels != value_token_axis.element_labels
        {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "token axis",
                reason: "key and value caches must share token identity and order".to_string(),
            });
        }
        if key_feature_axis.element_ids != value_feature_axis.element_ids {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "feature axis",
                reason: "key and value caches must share feature identity and order".to_string(),
            });
        }
        let capacity = self.keys.shape[key_token];
        let features = self.keys.shape[key_feature];
        if capacity > MAX_SLOTS || features > MAX_FEATURES {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "focused cache size",
                reason: format!(
                    "cache view supports at most {MAX_SLOTS} token slots by {MAX_FEATURES} features; slice larger tensors explicitly"
                ),
            });
        }
        if self.values.shape[value_token] != capacity
            || self.values.shape[value_feature] != features
        {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "cache shape",
                reason: "key and value caches must have matching token and feature dimensions"
                    .to_string(),
            });
        }
        if !self.occupancy.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "occupancy",
                value: self.occupancy,
            });
        }
        if !(0.0..=capacity as f32).contains(&self.occupancy) {
            return Err(ValidationError::OutOfRange {
                component: COMPONENT,
                field: "occupancy",
                minimum: 0.0,
                maximum: capacity as f32,
                value: self.occupancy,
            });
        }
        for (field, value) in [
            ("cell width", self.cell_size.x),
            ("cell height", self.cell_size.y),
            ("panel gap", self.panel_gap),
            ("padding", self.padding),
        ] {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
            if value <= 0.0 {
                return Err(ValidationError::NonPositive {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
        }
        Ok(())
    }

    fn token_axis_index(&self, snapshot: &TensorSnapshot) -> Result<usize, ValidationError> {
        snapshot
            .axes
            .iter()
            .position(|axis| axis.id == self.token_axis_id)
            .ok_or_else(|| ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "token axis",
                value: self.token_axis_id.clone(),
            })
    }

    fn feature_axis_index(&self, snapshot: &TensorSnapshot) -> Result<usize, ValidationError> {
        snapshot
            .axes
            .iter()
            .position(|axis| axis.id == self.feature_axis_id)
            .ok_or_else(|| ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "feature axis",
                value: self.feature_axis_id.clone(),
            })
    }

    fn value(&self, snapshot: &TensorSnapshot, token: usize, feature: usize) -> f32 {
        let token_axis = self
            .token_axis_index(snapshot)
            .expect("validated token axis");
        if token_axis == 0 {
            snapshot
                .value(&[token, feature])
                .expect("validated cache index")
        } else {
            snapshot
                .value(&[feature, token])
                .expect("validated cache index")
        }
    }

    fn scale_limit(&self) -> f32 {
        self.keys
            .values
            .iter()
            .chain(&self.values.values)
            .map(|value| value.abs())
            .fold(0.0, f32::max)
            .max(f32::EPSILON)
    }

    fn dimensions(&self) -> Vec2 {
        let matrix_width = self.feature_count() as f32 * self.cell_size.x;
        let width = self.padding * 2.0 + 1.35 + matrix_width * 2.0 + self.panel_gap;
        let height = self.padding * 2.0 + 0.72 + self.capacity() as f32 * self.cell_size.y;
        vec2(width, height)
    }
}

impl Project for KvCacheView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }
        let size = self.dimensions();
        emit_rect(
            ctx,
            size.x,
            size.y,
            Vec4::new(0.055, 0.068, 0.085, 0.98),
            Vec3::ZERO,
        );
        let left = -size.x * 0.5 + self.padding;
        let top = size.y * 0.5 - self.padding;
        emit_text(
            ctx,
            "KV CACHE",
            0.22,
            self.text_color,
            vec3(left + 0.7, top - 0.13, 0.05),
        );
        emit_text(
            ctx,
            &format!("{} / {} SLOTS", self.occupied_tokens(), self.capacity()),
            0.15,
            Vec4::new(0.70, 0.75, 0.80, 1.0),
            vec3(size.x * 0.5 - self.padding - 0.7, top - 0.13, 0.05),
        );

        let label_width = 1.35;
        let matrix_width = self.feature_count() as f32 * self.cell_size.x;
        let key_left = left + label_width;
        let value_left = key_left + matrix_width + self.panel_gap;
        emit_text(
            ctx,
            "KEYS",
            0.16,
            self.key_color,
            vec3(key_left + matrix_width * 0.5, top - 0.48, 0.05),
        );
        emit_text(
            ctx,
            "VALUES",
            0.16,
            self.value_color,
            vec3(value_left + matrix_width * 0.5, top - 0.48, 0.05),
        );

        let row_top = top - 0.72;
        let scale = self.scale_limit();
        let token_axis = &self.keys.axes[self.token_axis_index(&self.keys).unwrap()];
        for token in 0..self.capacity() {
            let row_y = row_top - token as f32 * self.cell_size.y - self.cell_size.y * 0.5;
            let strength = (self.occupancy - token as f32).clamp(0.0, 1.0);
            let is_newest = strength > 0.0 && token == self.occupancy.ceil().max(1.0) as usize - 1;
            if is_newest {
                emit_outline(
                    ctx,
                    key_left - 0.06,
                    value_left + matrix_width + 0.06,
                    row_y,
                    self.cell_size.y * 0.94,
                    self.active_color,
                );
            }
            emit_text(
                ctx,
                &format!("{}  [{}]", token_axis.element_labels[token], token),
                0.13,
                if strength > 0.0 {
                    self.text_color
                } else {
                    self.grid_color
                },
                vec3(left + 0.58, row_y, 0.05),
            );
            for feature in 0..self.feature_count() {
                for (snapshot, panel_left, positive) in [
                    (&self.keys, key_left, self.key_color),
                    (&self.values, value_left, self.value_color),
                ] {
                    let value = self.value(snapshot, token, feature);
                    let color = if strength <= 0.0 {
                        self.empty_color
                    } else {
                        value_color(value, scale, self.negative_color, self.zero_color, positive)
                            .lerp(self.empty_color, 1.0 - strength)
                    };
                    let x = panel_left + feature as f32 * self.cell_size.x + self.cell_size.x * 0.5;
                    emit_rect(
                        ctx,
                        self.cell_size.x * 0.94,
                        self.cell_size.y * 0.88,
                        color,
                        vec3(x, row_y, 0.03),
                    );
                }
            }
        }
    }
}

impl Bounded for KvCacheView {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::from_center_size(Vec2::ZERO, Vec2::splat(0.1));
        }
        Bounds::from_center_size(Vec2::ZERO, self.dimensions())
    }
}

fn value_color(value: f32, limit: f32, negative: Vec4, zero: Vec4, positive: Vec4) -> Vec4 {
    let normalized = (value / limit).clamp(-1.0, 1.0);
    if normalized < 0.0 {
        zero.lerp(negative, -normalized)
    } else {
        zero.lerp(positive, normalized)
    }
}

fn emit_rect(ctx: &mut ProjectionCtx, width: f32, height: f32, color: Vec4, center: Vec3) {
    ctx.emit(RenderPrimitive::Mesh(
        Mesh::rectangle(width, height, color).translated(center),
    ));
}

fn emit_text(ctx: &mut ProjectionCtx, content: &str, height: f32, color: Vec4, offset: Vec3) {
    ctx.emit(RenderPrimitive::Text {
        content: content.to_string(),
        height,
        color,
        font_name: None,
        offset,
        rotation: 0.0,
    });
}

fn emit_outline(
    ctx: &mut ProjectionCtx,
    left: f32,
    right: f32,
    center_y: f32,
    height: f32,
    color: Vec4,
) {
    let top = center_y + height * 0.5;
    let bottom = center_y - height * 0.5;
    for (start, end) in [
        (vec3(left, bottom, 0.07), vec3(right, bottom, 0.07)),
        (vec3(right, bottom, 0.07), vec3(right, top, 0.07)),
        (vec3(right, top, 0.07), vec3(left, top, 0.07)),
        (vec3(left, top, 0.07), vec3(left, bottom, 0.07)),
    ] {
        ctx.emit(RenderPrimitive::Line {
            start,
            end,
            thickness: 0.018,
            color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::{Scene, SharedProps};
    use crate::engine::timeline::Timeline;
    use crate::frontend::animation::Ease;

    pub(crate) fn cache_tensor(id: &str, values: Vec<f32>) -> TensorSnapshot {
        TensorSnapshot::try_new(
            id,
            vec![4, 3],
            values,
            vec![
                crate::frontend::collection::common::tensor::TensorAxis::with_elements(
                    "token",
                    "Tokens",
                    [
                        ("token.the", "The"),
                        ("token.model", "model"),
                        ("token.learns", "learns"),
                        ("token.next", "next"),
                    ],
                ),
                crate::frontend::collection::common::tensor::TensorAxis::new(
                    "feature",
                    "Head features",
                    vec!["f0", "f1", "f2"],
                ),
            ],
        )
        .unwrap()
    }

    pub(crate) fn view(occupied: usize) -> KvCacheView {
        KvCacheView::try_new(
            cache_tensor(
                "layer.3.head.1.keys",
                vec![
                    0.2, -0.4, 0.8, 0.5, 0.1, -0.2, 0.7, 0.3, -0.6, 0.4, 0.9, 0.1,
                ],
            ),
            cache_tensor(
                "layer.3.head.1.values",
                vec![
                    -0.1, 0.6, 0.3, 0.8, -0.5, 0.2, 0.4, 0.7, -0.3, 0.2, -0.8, 0.5,
                ],
            ),
            "token",
            "feature",
            occupied,
        )
        .unwrap()
    }

    #[test]
    fn preserves_cache_shape_identity_and_occupancy() {
        let view = view(2);
        assert_eq!(view.capacity(), 4);
        assert_eq!(view.feature_count(), 3);
        assert_eq!(view.occupied_tokens(), 2);
    }

    #[test]
    fn rejects_mismatched_token_identity() {
        let keys = cache_tensor("keys", vec![0.0; 12]);
        let mut values = cache_tensor("values", vec![0.0; 12]);
        values.axes[0].element_ids[1] = "token.other".to_string();
        assert!(matches!(
            KvCacheView::try_new(keys, values, "token", "feature", 2),
            Err(ValidationError::Incompatible {
                field: "token axis",
                ..
            })
        ));
    }

    #[test]
    fn invalid_occupancy_emits_a_projection_diagnostic() {
        let mut view = view(2);
        view.occupancy = 5.0;
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        view.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::OutOfRange {
                field: "occupancy",
                ..
            }
        ));
    }

    #[test]
    fn cache_fill_reconstructs_across_repeated_timeline_seeks() {
        let mut scene = Scene::new();
        let cache_id = scene.add_tattva(view(0), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(cache_id)
            .at(0.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .kv_cache_fill_to(4)
            .spawn();
        scene.play(timeline).unwrap();

        for (time, expected) in [(0.5, 2.0), (0.0, 0.0), (0.5, 2.0), (1.0, 4.0)] {
            scene.seek_to(time).unwrap();
            let cache = scene.get_tattva_typed::<KvCacheView>(cache_id).unwrap();
            assert!((cache.state.occupancy - expected).abs() < f32::EPSILON);
        }
    }
}
