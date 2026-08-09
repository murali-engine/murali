pub mod builder;
pub mod camera_animation;
pub mod camera_animation_builder;
pub mod indicate;

use parking_lot::Mutex;
use std::collections::HashMap;

use crate::engine::scene::Scene;
use crate::frontend::collection::ai::agentic_flow_chart::AgenticFlowChart;
use crate::frontend::collection::ai::signal_flow::SignalFlow;
use crate::frontend::collection::ai::tensor::{
    TensorSelectionFrame, TensorSelector, TensorSnapshot, TensorTransitionFrame, TensorView,
};
use crate::frontend::collection::ai::transformer_block_diagram::{
    TransformerBlockDiagram, TransformerStageFocusFrame,
};
use crate::frontend::collection::math::equation::{
    EquationLayout, EquationPart, EquationPartLayout,
};
use crate::frontend::collection::math::matrix::{Matrix, MatrixCellLayout};
use crate::frontend::collection::primitives::circle::Circle;
use crate::frontend::collection::primitives::ellipse::Ellipse;
use crate::frontend::collection::primitives::line::Line;
use crate::frontend::collection::primitives::noisy_circle::NoisyCircle;
use crate::frontend::collection::primitives::noisy_horizon::{LayeredPerlinField, NoisyHorizon};
use crate::frontend::collection::primitives::particle_belt::ParticleBelt;
use crate::frontend::collection::primitives::path::Path;
use crate::frontend::collection::primitives::polygon::Polygon;
use crate::frontend::collection::primitives::rectangle::Rectangle;
use crate::frontend::collection::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::collection::primitives::square::Square;
use crate::frontend::collection::primitives::to_path::ToPath;
use crate::frontend::collection::storytelling::stepwise::Stepwise;
use crate::frontend::layout::{Anchor, Bounded, Bounds};
use crate::frontend::props::DrawableProps;
use crate::frontend::tattva_trait::TattvaTrait;
use crate::frontend::{DirtyFlags, Tattva, TattvaId};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;
use glam::{Quat, Vec2, Vec3, Vec4};

/// Common easing curves for deterministic interpolation.
#[derive(Copy, Clone, Debug, Default)]
pub enum Ease {
    #[default]
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    /// Smoothstep: 3t² - 2t³. Continuous first derivative at 0 and 1.
    InOutSmooth,
}

impl Ease {
    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,
            Ease::InQuad => t * t,
            Ease::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::InOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Ease::InCubic => t * t * t,
            Ease::OutCubic => 1.0 - (1.0 - t).powi(3),
            Ease::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Ease::InOutSmooth => t * t * (3.0 - 2.0 * t),
        }
    }
}

/// The core trait for all Frontend logic changes over time.
/// Every implementation must be deterministic.
pub trait Animation: Send + Sync {
    fn validate(&self, _scene: &Scene) -> Result<(), crate::validation::ValidationError> {
        Ok(())
    }
    fn prepare(&mut self, _scene: &mut Scene) {}
    fn on_start(&mut self, scene: &mut Scene);
    fn apply_at(&mut self, scene: &mut Scene, t: f32);
    fn on_finish(&mut self, _scene: &mut Scene) {}
    fn reset(&mut self, _scene: &mut Scene) {}
    fn reapplies_terminal_state(&self) -> bool {
        false
    }
    fn seek_blocker(&self) -> Option<&'static str> {
        None
    }
}

pub struct TensorTransition {
    pub target_id: TattvaId,
    pub target: TensorSnapshot,
    pub ease: Ease,
    source: Option<TensorSnapshot>,
}

pub struct TensorSelectionTransition {
    pub target_id: TattvaId,
    pub target: Vec<TensorSelector>,
    pub ease: Ease,
    source: Option<Vec<TensorSelector>>,
}

pub struct TransformerStageFocus {
    pub target_id: TattvaId,
    pub target_stage_id: Option<String>,
    pub ease: Ease,
    source_stage_id: Option<Option<String>>,
}

impl TransformerStageFocus {
    pub fn new(target_id: TattvaId, target_stage_id: Option<String>, ease: Ease) -> Self {
        Self {
            target_id,
            target_stage_id,
            ease,
            source_stage_id: None,
        }
    }

    fn mark_changed(tattva: &mut Tattva<TransformerBlockDiagram>) {
        tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
    }
}

impl Animation for TransformerStageFocus {
    fn validate(&self, scene: &Scene) -> Result<(), crate::validation::ValidationError> {
        let Some(tattva) = scene.get_tattva_any(self.target_id) else {
            return Err(crate::validation::ValidationError::MissingTarget {
                component: "TransformerStageFocus",
                target_id: self.target_id,
            });
        };
        let Some(diagram) = tattva
            .as_any()
            .downcast_ref::<Tattva<TransformerBlockDiagram>>()
        else {
            return Err(crate::validation::ValidationError::TargetTypeMismatch {
                component: "TransformerStageFocus",
                target_id: self.target_id,
                expected: "TransformerBlockDiagram",
            });
        };
        let mut terminal = diagram.state.clone();
        terminal.active_stage_id = self.target_stage_id.clone();
        terminal.focus_transition = None;
        terminal.validate()
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(diagram) = scene.get_tattva_typed::<TransformerBlockDiagram>(self.target_id) {
            self.source_stage_id = Some(diagram.state.active_stage_id.clone());
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(source_stage_id) = &self.source_stage_id else {
            return;
        };
        let progress = self.ease.eval(t);
        if let Some(diagram) = scene.get_tattva_typed_mut::<TransformerBlockDiagram>(self.target_id)
        {
            diagram.state.active_stage_id = self.target_stage_id.clone();
            diagram.state.focus_transition =
                (progress < 1.0 - f32::EPSILON).then(|| TransformerStageFocusFrame {
                    source_stage_id: source_stage_id.clone(),
                    target_stage_id: self.target_stage_id.clone(),
                    progress,
                });
            Self::mark_changed(diagram);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        self.apply_at(scene, 1.0);
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(source_stage_id) = &self.source_stage_id {
            if let Some(diagram) =
                scene.get_tattva_typed_mut::<TransformerBlockDiagram>(self.target_id)
            {
                diagram.state.active_stage_id = source_stage_id.clone();
                diagram.state.focus_transition = None;
                Self::mark_changed(diagram);
            }
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

impl TensorSelectionTransition {
    pub fn new(target_id: TattvaId, target: Vec<TensorSelector>, ease: Ease) -> Self {
        Self {
            target_id,
            target,
            ease,
            source: None,
        }
    }
}

impl Animation for TensorSelectionTransition {
    fn validate(&self, scene: &Scene) -> Result<(), crate::validation::ValidationError> {
        let Some(tattva) = scene.get_tattva_any(self.target_id) else {
            return Err(crate::validation::ValidationError::MissingTarget {
                component: "TensorSelectionTransition",
                target_id: self.target_id,
            });
        };
        let Some(tensor) = tattva.as_any().downcast_ref::<Tattva<TensorView>>() else {
            return Err(crate::validation::ValidationError::TargetTypeMismatch {
                component: "TensorSelectionTransition",
                target_id: self.target_id,
                expected: "TensorView",
            });
        };
        TensorView::validate_selections(&self.target, &tensor.state.snapshot)
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tensor) = scene.get_tattva_typed::<TensorView>(self.target_id) {
            self.source = Some(tensor.state.selections.clone());
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(source) = &self.source else {
            return;
        };
        let progress = self.ease.eval(t);
        if let Some(tensor) = scene.get_tattva_typed_mut::<TensorView>(self.target_id) {
            tensor.state.selections = self.target.clone();
            tensor.state.selection_transition =
                (progress < 1.0 - f32::EPSILON).then(|| TensorSelectionFrame {
                    source: source.clone(),
                    target: self.target.clone(),
                    progress,
                });
            TensorTransition::mark_changed(tensor);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        self.apply_at(scene, 1.0);
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(source) = &self.source {
            if let Some(tensor) = scene.get_tattva_typed_mut::<TensorView>(self.target_id) {
                tensor.state.selections = source.clone();
                tensor.state.selection_transition = None;
                TensorTransition::mark_changed(tensor);
            }
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

impl TensorTransition {
    pub fn new(target_id: TattvaId, target: TensorSnapshot, ease: Ease) -> Self {
        Self {
            target_id,
            target,
            ease,
            source: None,
        }
    }

    fn mark_changed(tensor: &mut Tattva<TensorView>) {
        tensor.mark_dirty(
            DirtyFlags::GEOMETRY | DirtyFlags::STYLE | DirtyFlags::TEXT_LAYOUT | DirtyFlags::BOUNDS,
        );
    }
}

impl Animation for TensorTransition {
    fn validate(&self, scene: &Scene) -> Result<(), crate::validation::ValidationError> {
        let Some(tattva) = scene.get_tattva_any(self.target_id) else {
            return Err(crate::validation::ValidationError::MissingTarget {
                component: "TensorTransition",
                target_id: self.target_id,
            });
        };
        let Some(tensor) = tattva.as_any().downcast_ref::<Tattva<TensorView>>() else {
            return Err(crate::validation::ValidationError::TargetTypeMismatch {
                component: "TensorTransition",
                target_id: self.target_id,
                expected: "TensorView",
            });
        };
        tensor.state.snapshot.validate_transition_to(&self.target)?;
        let mut terminal_view = tensor.state.clone();
        terminal_view.snapshot = self.target.clone();
        terminal_view.transition = None;
        terminal_view.validate()
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tensor) = scene.get_tattva_typed::<TensorView>(self.target_id) {
            self.source = Some(tensor.state.snapshot.clone());
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(source) = &self.source else {
            return;
        };
        let progress = self.ease.eval(t);
        if let Some(tensor) = scene.get_tattva_typed_mut::<TensorView>(self.target_id) {
            if progress >= 1.0 - f32::EPSILON {
                tensor.state.snapshot = self.target.clone();
                tensor.state.transition = None;
            } else {
                tensor.state.snapshot = source.interpolated_target(&self.target, progress);
                tensor.state.transition = Some(TensorTransitionFrame {
                    source: source.clone(),
                    target: self.target.clone(),
                    progress,
                });
            }
            Self::mark_changed(tensor);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        self.apply_at(scene, 1.0);
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(source) = &self.source {
            if let Some(tensor) = scene.get_tattva_typed_mut::<TensorView>(self.target_id) {
                tensor.state.snapshot = source.clone();
                tensor.state.transition = None;
                Self::mark_changed(tensor);
            }
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

pub struct RunSceneCallback {
    callback: Mutex<Box<dyn FnMut(&mut Scene) + Send>>,
}

impl RunSceneCallback {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(&mut Scene) + Send + 'static,
    {
        Self {
            callback: Mutex::new(Box::new(callback)),
        }
    }
}

impl Animation for RunSceneCallback {
    fn on_start(&mut self, _scene: &mut Scene) {}

    fn apply_at(&mut self, _scene: &mut Scene, _t: f32) {}

    fn on_finish(&mut self, scene: &mut Scene) {
        (self.callback.lock())(scene);
    }

    fn seek_blocker(&self) -> Option<&'static str> {
        Some("call_at")
    }
}

pub struct RunSceneCallbackOverTime {
    callback: Mutex<Box<dyn FnMut(&mut Scene, f32) + Send>>,
}

impl RunSceneCallbackOverTime {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(&mut Scene, f32) + Send + 'static,
    {
        Self {
            callback: Mutex::new(Box::new(callback)),
        }
    }
}

impl Animation for RunSceneCallbackOverTime {
    fn on_start(&mut self, _scene: &mut Scene) {}

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        (self.callback.lock())(scene, t.clamp(0.0, 1.0));
    }

    fn seek_blocker(&self) -> Option<&'static str> {
        Some("call_during")
    }
}

pub struct RunReversibleSceneCallback {
    callback: Mutex<Box<dyn FnMut(&mut Scene) + Send>>,
    reset: Mutex<Box<dyn FnMut(&mut Scene) + Send>>,
}

impl RunReversibleSceneCallback {
    pub fn new<F, R>(callback: F, reset: R) -> Self
    where
        F: FnMut(&mut Scene) + Send + 'static,
        R: FnMut(&mut Scene) + Send + 'static,
    {
        Self {
            callback: Mutex::new(Box::new(callback)),
            reset: Mutex::new(Box::new(reset)),
        }
    }
}

impl Animation for RunReversibleSceneCallback {
    fn on_start(&mut self, _scene: &mut Scene) {}

    fn apply_at(&mut self, _scene: &mut Scene, _t: f32) {}

    fn on_finish(&mut self, scene: &mut Scene) {
        (self.callback.lock())(scene);
    }

    fn reset(&mut self, scene: &mut Scene) {
        (self.reset.lock())(scene);
    }
}

pub struct RunReversibleSceneCallbackOverTime {
    callback: Mutex<Box<dyn FnMut(&mut Scene, f32) + Send>>,
    reset: Mutex<Box<dyn FnMut(&mut Scene) + Send>>,
}

impl RunReversibleSceneCallbackOverTime {
    pub fn new<F, R>(callback: F, reset: R) -> Self
    where
        F: FnMut(&mut Scene, f32) + Send + 'static,
        R: FnMut(&mut Scene) + Send + 'static,
    {
        Self {
            callback: Mutex::new(Box::new(callback)),
            reset: Mutex::new(Box::new(reset)),
        }
    }
}

impl Animation for RunReversibleSceneCallbackOverTime {
    fn on_start(&mut self, _scene: &mut Scene) {}

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        (self.callback.lock())(scene, t.clamp(0.0, 1.0));
    }

    fn reset(&mut self, scene: &mut Scene) {
        (self.reset.lock())(scene);
    }
}

fn with_props_mut<F>(scene: &mut Scene, target_id: TattvaId, dirty: DirtyFlags, f: F)
where
    F: FnOnce(&mut DrawableProps),
{
    if let Some(tattva) = scene.get_tattva_any_mut(target_id) {
        let mut props = DrawableProps::write(tattva.props());
        f(&mut props);
        drop(props);
        tattva.mark_dirty(dirty);
    }
}

pub struct MoveTo {
    pub target_id: TattvaId,
    pub to: Vec3,
    pub ease: Ease,
    from: Option<Vec3>,
}

impl MoveTo {
    pub fn new(target_id: TattvaId, to: Vec3, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }

    pub fn with_from(mut self, from: Vec3) -> Self {
        self.from = Some(from);
        self
    }
}

impl Animation for MoveTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if self.from.is_none() {
            if let Some(tattva) = scene.get_tattva_any(self.target_id) {
                let props = DrawableProps::read(tattva.props());
                self.from = Some(props.position);
            }
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let new_pos = self
            .from
            .unwrap_or(self.to)
            .lerp(self.to, self.ease.eval(t));
        with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
            props.position = new_pos;
        });
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

pub struct RotateTo {
    pub target_id: TattvaId,
    pub to: Quat,
    pub ease: Ease,
    from: Option<Quat>,
}

impl RotateTo {
    pub fn new(target_id: TattvaId, to: Quat, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }

    pub fn with_from(mut self, from: Quat) -> Self {
        self.from = Some(from);
        self
    }
}

impl Animation for RotateTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if self.from.is_none() {
            if let Some(tattva) = scene.get_tattva_any(self.target_id) {
                let props = DrawableProps::read(tattva.props());
                self.from = Some(props.rotation);
            }
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(self.to);
        let rotation = from.slerp(self.to, self.ease.eval(t));
        with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
            props.rotation = rotation;
        });
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

pub struct ScaleTo {
    pub target_id: TattvaId,
    pub to: Vec3,
    pub ease: Ease,
    from: Option<Vec3>,
}

impl ScaleTo {
    pub fn new(target_id: TattvaId, to: Vec3, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }

    pub fn with_from(mut self, from: Vec3) -> Self {
        self.from = Some(from);
        self
    }
}

impl Animation for ScaleTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if self.from.is_none() {
            if let Some(tattva) = scene.get_tattva_any(self.target_id) {
                let props = DrawableProps::read(tattva.props());
                self.from = Some(props.scale);
            }
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let scale = self
            .from
            .unwrap_or(self.to)
            .lerp(self.to, self.ease.eval(t));
        with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
            props.scale = scale;
        });
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

pub struct FadeTo {
    pub target_id: TattvaId,
    pub to: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl FadeTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            to: to.clamp(0.0, 1.0),
            ease,
            from: None,
        }
    }

    pub fn with_from(mut self, from: f32) -> Self {
        self.from = Some(from);
        self
    }
}

impl Animation for FadeTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if self.from.is_none() {
            if let Some(tattva) = scene.get_tattva_any(self.target_id) {
                let props = DrawableProps::read(tattva.props());
                self.from = Some(props.opacity);
            }
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let opacity = self.from.unwrap_or(self.to)
            + (self.to - self.from.unwrap_or(self.to)) * self.ease.eval(t);
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = opacity > 0.001;
                props.opacity = opacity.clamp(0.0, 1.0);
            },
        );
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(from) = self.from {
            with_props_mut(
                scene,
                self.target_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    props.opacity = from.clamp(0.0, 1.0);
                    props.visible = from > 0.001;
                },
            );
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

/// IndicateText animation: a short pulse for Label and LaTeX tattvas.
pub struct IndicateText {
    pub target_id: TattvaId,
    pub ease: Ease,
}

impl IndicateText {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self { target_id, ease }
    }

    fn set_indicate_t(&self, scene: &mut Scene, t: f32) {
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.indicate_t = t.clamp(0.0, 1.0);
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.indicate_t = t.clamp(0.0, 1.0);
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

impl Animation for IndicateText {
    fn on_start(&mut self, scene: &mut Scene) {
        self.set_indicate_t(scene, 0.0);
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        self.set_indicate_t(scene, self.ease.eval(t));
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        self.set_indicate_t(scene, 0.0);
    }

    fn reset(&mut self, scene: &mut Scene) {
        self.set_indicate_t(scene, 0.0);
    }
}

pub struct Create {
    pub target_id: TattvaId,
    pub ease: Ease,
    target_opacity: Option<f32>,
}

impl Create {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            target_opacity: None,
        }
    }
}

impl Animation for Create {
    fn prepare(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
            let mut props = DrawableProps::write(tattva.props());
            self.target_opacity = Some(if props.opacity > 0.001 {
                props.opacity
            } else {
                1.0
            });
            props.visible = false;
            props.opacity = 0.0;
            drop(props);
            tattva.mark_dirty(DirtyFlags::STYLE | DirtyFlags::VISIBILITY);
        }
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
            let mut props = DrawableProps::write(tattva.props());
            // If the target was pre-hidden for staging, treat create() as revealing it to
            // normal visibility rather than preserving the staged zero opacity.
            if self.target_opacity.is_none() {
                self.target_opacity = Some(if props.opacity > 0.001 {
                    props.opacity
                } else {
                    1.0
                });
            }
            props.visible = true;
            props.opacity = 0.0;
            drop(props);
            tattva.mark_dirty(DirtyFlags::STYLE | DirtyFlags::VISIBILITY);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let target = self.target_opacity.unwrap_or(1.0);
        let opacity = target * self.ease.eval(t);
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.opacity = opacity.clamp(0.0, target);
            },
        );
    }

    fn reset(&mut self, scene: &mut Scene) {
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = false;
                props.opacity = 0.0;
            },
        );
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

pub struct FollowAnchor {
    pub follower_id: TattvaId,
    pub target_id: TattvaId,
    pub target_anchor: Anchor,
    pub follower_anchor: Anchor,
    pub offset: Vec3,
}

impl FollowAnchor {
    pub fn new(
        follower_id: TattvaId,
        target_id: TattvaId,
        target_anchor: Anchor,
        follower_anchor: Anchor,
        offset: Vec3,
    ) -> Self {
        Self {
            follower_id,
            target_id,
            target_anchor,
            follower_anchor,
            offset,
        }
    }

    fn sync_position(&self, scene: &mut Scene) {
        let Some(target_point) = scene.anchor_position(self.target_id, self.target_anchor) else {
            return;
        };
        let Some(local_bounds) = scene.local_bounds(self.follower_id) else {
            return;
        };
        let pos = target_point + self.offset.truncate() - local_bounds.anchor(self.follower_anchor);
        scene.set_position_2d(self.follower_id, pos);
    }
}

impl Animation for FollowAnchor {
    fn on_start(&mut self, scene: &mut Scene) {
        self.sync_position(scene);
    }

    fn apply_at(&mut self, scene: &mut Scene, _t: f32) {
        self.sync_position(scene);
    }
}

pub struct PropagateSignal {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
    to: f32,
}

impl PropagateSignal {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
            to: to.clamp(0.0, 1.0),
        }
    }
}

impl Animation for PropagateSignal {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed::<SignalFlow>(self.target_id) {
            self.from = Some(flow.state.progress);
        } else if let Some(flow) = scene.get_tattva_typed::<AgenticFlowChart>(self.target_id) {
            self.from = Some(flow.state.progress);
        } else if let Some(sw) = scene.get_tattva_typed::<Stepwise>(self.target_id) {
            self.from = Some(sw.state.progress);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let progress = from + (self.to - from) * self.ease.eval(t);
        if let Some(flow) = scene.get_tattva_typed_mut::<SignalFlow>(self.target_id) {
            flow.state.progress = progress.clamp(0.0, 1.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.progress = progress.clamp(0.0, 1.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.progress = progress.clamp(0.0, 1.0);
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed_mut::<SignalFlow>(self.target_id) {
            flow.state.progress = self.to;
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.progress = self.to;
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.progress = self.to;
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed_mut::<SignalFlow>(self.target_id) {
            flow.state.progress = self.from.unwrap_or(0.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.progress = self.from.unwrap_or(0.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        } else if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.progress = self.from.unwrap_or(0.0);
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

/// Animates `Stepwise::signal_progress` — drives the signal dot along the
/// sequence path independently of the build animation.
pub struct StepwiseSignal {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
    to: f32,
}

impl StepwiseSignal {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
            to: to.clamp(0.0, 1.0),
        }
    }
}

impl Animation for StepwiseSignal {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(sw) = scene.get_tattva_typed::<Stepwise>(self.target_id) {
            self.from = Some(sw.state.signal_progress);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let p = from + (self.to - from) * self.ease.eval(t);
        if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.signal_progress = p.clamp(0.0, 1.0);
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.signal_progress = self.to;
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(sw) = scene.get_tattva_typed_mut::<Stepwise>(self.target_id) {
            sw.state.signal_progress = self.from.unwrap_or(0.0);
            sw.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

pub struct RevealTo {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
    to: f32,
}

impl RevealTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
            to: to.clamp(0.0, 1.0),
        }
    }
}

impl Animation for RevealTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed::<AgenticFlowChart>(self.target_id) {
            self.from = Some(flow.state.reveal_progress);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let progress = from + (self.to - from) * self.ease.eval(t);
        if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.reveal_progress = progress.clamp(0.0, 1.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.reveal_progress = self.to;
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(flow) = scene.get_tattva_typed_mut::<AgenticFlowChart>(self.target_id) {
            flow.state.reveal_progress = self.from.unwrap_or(0.0);
            flow.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct NoisePhaseTo {
    pub target_id: TattvaId,
    pub to: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl NoisePhaseTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }
}

impl Animation for NoisePhaseTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed::<NoisyCircle>(self.target_id) {
            self.from = Some(circle.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(self.to);
        let phase = from + (self.to - from) * self.ease.eval(t);
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = phase;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = self.to;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = self.from.unwrap_or(0.0);
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct NoisePhaseBy {
    pub target_id: TattvaId,
    pub delta: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl NoisePhaseBy {
    pub fn new(target_id: TattvaId, delta: f32, ease: Ease) -> Self {
        Self {
            target_id,
            delta,
            ease,
            from: None,
        }
    }
}

impl Animation for NoisePhaseBy {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed::<NoisyCircle>(self.target_id) {
            self.from = Some(circle.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let phase = from + self.delta * self.ease.eval(t);
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = phase;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = from + self.delta;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = self.from.unwrap_or(0.0);
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct NoiseEvolve {
    pub target_id: TattvaId,
    pub duration: f32,
    pub speed_override: Option<f32>,
    pub ease: Ease,
    from: Option<f32>,
}

impl NoiseEvolve {
    pub fn new(
        target_id: TattvaId,
        duration: f32,
        speed_override: Option<f32>,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            duration,
            speed_override,
            ease,
            from: None,
        }
    }
}

impl Animation for NoiseEvolve {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed::<NoisyCircle>(self.target_id) {
            self.from = Some(circle.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            let speed = self.speed_override.unwrap_or(circle.state.morph_speed);
            let phase = from + speed * self.duration * self.ease.eval(t);
            circle.state.phase = phase;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            let speed = self.speed_override.unwrap_or(circle.state.morph_speed);
            circle.state.phase = from + speed * self.duration;
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(self.target_id) {
            circle.state.phase = self.from.unwrap_or(0.0);
            circle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct BeltPhaseTo {
    pub target_id: TattvaId,
    pub to: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl BeltPhaseTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }
}

impl Animation for BeltPhaseTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed::<ParticleBelt>(self.target_id) {
            self.from = Some(belt.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(self.to);
        let phase = from + (self.to - from) * self.ease.eval(t);
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = phase;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = self.to;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = self.from.unwrap_or(0.0);
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct BeltPhaseBy {
    pub target_id: TattvaId,
    pub delta: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl BeltPhaseBy {
    pub fn new(target_id: TattvaId, delta: f32, ease: Ease) -> Self {
        Self {
            target_id,
            delta,
            ease,
            from: None,
        }
    }
}

impl Animation for BeltPhaseBy {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed::<ParticleBelt>(self.target_id) {
            self.from = Some(belt.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let phase = from + self.delta * self.ease.eval(t);
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = phase;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = from + self.delta;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = self.from.unwrap_or(0.0);
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct BeltEvolve {
    pub target_id: TattvaId,
    pub duration: f32,
    pub speed_override: Option<f32>,
    pub ease: Ease,
    from: Option<f32>,
}

impl BeltEvolve {
    pub fn new(
        target_id: TattvaId,
        duration: f32,
        speed_override: Option<f32>,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            duration,
            speed_override,
            ease,
            from: None,
        }
    }
}

impl Animation for BeltEvolve {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed::<ParticleBelt>(self.target_id) {
            self.from = Some(belt.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            let speed = self.speed_override.unwrap_or(belt.state.orbit_speed);
            let phase = from + speed * self.duration * self.ease.eval(t);
            belt.state.phase = phase;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            let speed = self.speed_override.unwrap_or(belt.state.orbit_speed);
            belt.state.phase = from + speed * self.duration;
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(self.target_id) {
            belt.state.phase = self.from.unwrap_or(0.0);
            belt.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct HorizonPhaseTo {
    pub target_id: TattvaId,
    pub to: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl HorizonPhaseTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }
}

impl Animation for HorizonPhaseTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed::<NoisyHorizon>(self.target_id) {
            self.from = Some(horizon.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(self.to);
        let phase = from + (self.to - from) * self.ease.eval(t);
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = phase;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = self.to;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = self.from.unwrap_or(0.0);
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct HorizonPhaseBy {
    pub target_id: TattvaId,
    pub delta: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl HorizonPhaseBy {
    pub fn new(target_id: TattvaId, delta: f32, ease: Ease) -> Self {
        Self {
            target_id,
            delta,
            ease,
            from: None,
        }
    }
}

impl Animation for HorizonPhaseBy {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed::<NoisyHorizon>(self.target_id) {
            self.from = Some(horizon.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let phase = from + self.delta * self.ease.eval(t);
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = phase;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = from + self.delta;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = self.from.unwrap_or(0.0);
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct HorizonEvolve {
    pub target_id: TattvaId,
    pub duration: f32,
    pub speed_override: Option<f32>,
    pub ease: Ease,
    from: Option<f32>,
}

impl HorizonEvolve {
    pub fn new(
        target_id: TattvaId,
        duration: f32,
        speed_override: Option<f32>,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            duration,
            speed_override,
            ease,
            from: None,
        }
    }
}

impl Animation for HorizonEvolve {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed::<NoisyHorizon>(self.target_id) {
            self.from = Some(horizon.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            let speed = self.speed_override.unwrap_or(horizon.state.morph_speed);
            let phase = from + speed * self.duration * self.ease.eval(t);
            horizon.state.phase = phase;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            let speed = self.speed_override.unwrap_or(horizon.state.morph_speed);
            horizon.state.phase = from + speed * self.duration;
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(self.target_id) {
            horizon.state.phase = self.from.unwrap_or(0.0);
            horizon.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct PerlinFieldPhaseTo {
    pub target_id: TattvaId,
    pub to: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl PerlinFieldPhaseTo {
    pub fn new(target_id: TattvaId, to: f32, ease: Ease) -> Self {
        Self {
            target_id,
            to,
            ease,
            from: None,
        }
    }
}

impl Animation for PerlinFieldPhaseTo {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed::<LayeredPerlinField>(self.target_id) {
            self.from = Some(field.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(self.to);
        let phase = from + (self.to - from) * self.ease.eval(t);
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = phase;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = self.to;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = self.from.unwrap_or(0.0);
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct PerlinFieldPhaseBy {
    pub target_id: TattvaId,
    pub delta: f32,
    pub ease: Ease,
    from: Option<f32>,
}

impl PerlinFieldPhaseBy {
    pub fn new(target_id: TattvaId, delta: f32, ease: Ease) -> Self {
        Self {
            target_id,
            delta,
            ease,
            from: None,
        }
    }
}

impl Animation for PerlinFieldPhaseBy {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed::<LayeredPerlinField>(self.target_id) {
            self.from = Some(field.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        let phase = from + self.delta * self.ease.eval(t);
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = phase;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = from + self.delta;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = self.from.unwrap_or(0.0);
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

pub struct PerlinFieldEvolve {
    pub target_id: TattvaId,
    pub duration: f32,
    pub speed_override: Option<f32>,
    pub ease: Ease,
    from: Option<f32>,
}

impl PerlinFieldEvolve {
    pub fn new(
        target_id: TattvaId,
        duration: f32,
        speed_override: Option<f32>,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            duration,
            speed_override,
            ease,
            from: None,
        }
    }
}

impl Animation for PerlinFieldEvolve {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed::<LayeredPerlinField>(self.target_id) {
            self.from = Some(field.state.phase);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let from = self.from.unwrap_or(0.0);
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            let speed = self.speed_override.unwrap_or(field.state.morph_speed);
            let phase = from + speed * self.duration * self.ease.eval(t);
            field.state.phase = phase;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        let from = self.from.unwrap_or(0.0);
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            let speed = self.speed_override.unwrap_or(field.state.morph_speed);
            field.state.phase = from + speed * self.duration;
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(field) = scene.get_tattva_typed_mut::<LayeredPerlinField>(self.target_id) {
            field.state.phase = self.from.unwrap_or(0.0);
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

fn safe_div_vec2(numerator: Vec2, denominator: Vec2, fallback: Vec2) -> Vec2 {
    Vec2::new(
        if denominator.x.abs() > 1e-5 {
            numerator.x / denominator.x
        } else {
            fallback.x
        },
        if denominator.y.abs() > 1e-5 {
            numerator.y / denominator.y
        } else {
            fallback.y
        },
    )
}

#[derive(Clone)]
struct MatchSnapshot {
    source: DrawableProps,
    target: DrawableProps,
    matched_position: Vec3,
    matched_scale: Vec3,
}

fn build_match_snapshot(
    scene: &Scene,
    source_id: TattvaId,
    target_id: TattvaId,
) -> Option<MatchSnapshot> {
    let source = scene.get_tattva_any(source_id)?;
    let target = scene.get_tattva_any(target_id)?;
    let source_props = DrawableProps::read(source.props()).clone();
    let target_props = DrawableProps::read(target.props()).clone();
    let source_bounds = scene.world_bounds(source_id)?;
    let target_local = target.local_bounds();
    let target_local_size = target_local.size();
    let source_size = source_bounds.size();

    let matched_xy = safe_div_vec2(
        source_size,
        Vec2::new(target_local_size.x.max(1e-5), target_local_size.y.max(1e-5)),
        target_props.scale.truncate(),
    );

    Some(MatchSnapshot {
        source: source_props.clone(),
        target: target_props.clone(),
        matched_position: Vec3::new(
            source_bounds.center().x,
            source_bounds.center().y,
            source_props.position.z,
        ),
        matched_scale: Vec3::new(matched_xy.x, matched_xy.y, target_props.scale.z),
    })
}

pub struct MatchTransform {
    pub source_id: TattvaId,
    pub target_id: TattvaId,
    pub ease: Ease,
    snapshot: Option<MatchSnapshot>,
}

impl MatchTransform {
    pub fn new(source_id: TattvaId, target_id: TattvaId, ease: Ease) -> Self {
        Self {
            source_id,
            target_id,
            ease,
            snapshot: None,
        }
    }
}

impl Animation for MatchTransform {
    fn on_start(&mut self, scene: &mut Scene) {
        self.snapshot = build_match_snapshot(scene, self.source_id, self.target_id);
        let Some(snapshot) = &self.snapshot else {
            return;
        };

        with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
            props.visible = true;
            props.position = snapshot.matched_position;
            props.scale = snapshot.matched_scale;
            props.rotation = snapshot.source.rotation;
        });
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let eased = self.ease.eval(t);
        with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
            props.visible = true;
            props.position = snapshot
                .matched_position
                .lerp(snapshot.target.position, eased);
            props.scale = snapshot.matched_scale.lerp(snapshot.target.scale, eased);
            props.rotation = snapshot
                .source
                .rotation
                .slerp(snapshot.target.rotation, eased);
        });
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
                *props = snapshot.target.clone();
            });
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            with_props_mut(scene, self.target_id, DirtyFlags::TRANSFORM, |props| {
                *props = snapshot.target.clone();
            });
        }
    }
}

pub struct MorphObjects {
    pub source_id: TattvaId,
    pub target_id: TattvaId,
    pub ease: Ease,
    snapshot: Option<MatchSnapshot>,
}

impl MorphObjects {
    pub fn new(source_id: TattvaId, target_id: TattvaId, ease: Ease) -> Self {
        Self {
            source_id,
            target_id,
            ease,
            snapshot: None,
        }
    }
}

impl Animation for MorphObjects {
    fn on_start(&mut self, scene: &mut Scene) {
        self.snapshot = build_match_snapshot(scene, self.source_id, self.target_id);
        let Some(snapshot) = &self.snapshot else {
            return;
        };

        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::TRANSFORM | DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.position = snapshot.matched_position;
                props.scale = snapshot.matched_scale;
                props.rotation = snapshot.source.rotation;
                props.opacity = 0.0;
            },
        );
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let eased = self.ease.eval(t);
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::TRANSFORM | DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.position = snapshot
                    .matched_position
                    .lerp(snapshot.target.position, eased);
                props.scale = snapshot.matched_scale.lerp(snapshot.target.scale, eased);
                props.rotation = snapshot
                    .source
                    .rotation
                    .slerp(snapshot.target.rotation, eased);
                props.opacity = snapshot.target.opacity * eased;
            },
        );
        with_props_mut(
            scene,
            self.source_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.opacity = snapshot.source.opacity * (1.0 - eased);
                props.visible = props.opacity > 0.001;
            },
        );
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            with_props_mut(
                scene,
                self.target_id,
                DirtyFlags::TRANSFORM | DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.target.clone();
                },
            );
            with_props_mut(
                scene,
                self.source_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    props.opacity = 0.0;
                    props.visible = false;
                },
            );
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            with_props_mut(
                scene,
                self.target_id,
                DirtyFlags::TRANSFORM | DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.target.clone();
                },
            );
            with_props_mut(
                scene,
                self.source_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.source.clone();
                },
            );
        }
    }
}

#[derive(Clone)]
struct EquationContinuitySnapshot {
    source_props: DrawableProps,
    target_props: DrawableProps,
    final_target_opacity: f32,
    source_parts: Vec<EquationPartLayout>,
    target_parts: Vec<EquationPartLayout>,
    original_target_parts: Vec<EquationPart>,
}

pub struct EquationContinuity {
    pub source_id: TattvaId,
    pub target_id: TattvaId,
    pub ease: Ease,
    snapshot: Option<EquationContinuitySnapshot>,
}

impl EquationContinuity {
    pub fn new(source_id: TattvaId, target_id: TattvaId, ease: Ease) -> Self {
        Self {
            source_id,
            target_id,
            ease,
            snapshot: None,
        }
    }
}

impl Animation for EquationContinuity {
    fn on_start(&mut self, scene: &mut Scene) {
        let source_tattva = match scene.get_tattva_typed::<EquationLayout>(self.source_id) {
            Some(t) => t,
            None => return,
        };
        let target_tattva = match scene.get_tattva_typed::<EquationLayout>(self.target_id) {
            Some(t) => t,
            None => return,
        };

        let source_props = DrawableProps::read(&source_tattva.props).clone();
        let target_props = DrawableProps::read(&target_tattva.props).clone();
        let final_target_opacity = if target_props.opacity > 0.001 {
            target_props.opacity
        } else {
            source_props.opacity.max(0.001)
        };

        self.snapshot = Some(EquationContinuitySnapshot {
            source_props,
            target_props,
            final_target_opacity,
            source_parts: source_tattva.state.layout_snapshot(),
            target_parts: target_tattva.state.layout_snapshot(),
            original_target_parts: target_tattva.state.parts.clone(),
        });
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let eased = self.ease.eval(t);
        let source_by_key: HashMap<&str, &EquationPartLayout> = snapshot
            .source_parts
            .iter()
            .map(|part| (part.key.as_str(), part))
            .collect();

        let target_props = snapshot.target_props.clone();
        let source_props = snapshot.source_props.clone();

        if let Some(target) = scene.get_tattva_typed_mut::<EquationLayout>(self.target_id) {
            for (idx, part) in target.state.parts.iter_mut().enumerate() {
                let base = &snapshot.original_target_parts[idx];
                let target_layout = &snapshot.target_parts[idx];
                *part = base.clone();

                if let Some(source_layout) = source_by_key.get(target_layout.key.as_str()) {
                    let source_world = source_props.position.truncate()
                        + source_layout.center.truncate() * source_props.scale.truncate();
                    let target_local_start = safe_div_vec2(
                        source_world - target_props.position.truncate(),
                        target_props.scale.truncate(),
                        target_layout.center.truncate(),
                    );
                    let blended_center = Vec2::new(target_local_start.x, target_local_start.y)
                        .lerp(target_layout.center.truncate(), eased);
                    part.offset += Vec3::new(
                        blended_center.x - target_layout.center.x,
                        blended_center.y - target_layout.center.y,
                        0.0,
                    );
                    part.scale =
                        source_layout.scale + (target_layout.scale - source_layout.scale) * eased;
                    part.opacity = source_layout.opacity
                        + (target_layout.opacity - source_layout.opacity) * eased;
                } else {
                    part.offset += Vec3::new(0.0, (1.0 - eased) * target_layout.height * 0.35, 0.0);
                    part.opacity = target_layout.opacity * eased;
                    part.scale = 0.9 + 0.1 * eased;
                }
            }
            target.mark_dirty(DirtyFlags::TEXT_LAYOUT | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }

        with_props_mut(
            scene,
            self.source_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.opacity = snapshot.source_props.opacity * (1.0 - eased);
                props.visible = props.opacity > 0.001;
            },
        );
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.opacity = snapshot.final_target_opacity;
            },
        );
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            if let Some(target) = scene.get_tattva_typed_mut::<EquationLayout>(self.target_id) {
                target.state.parts = snapshot.original_target_parts.clone();
                target.mark_dirty(DirtyFlags::TEXT_LAYOUT | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            }
            with_props_mut(
                scene,
                self.source_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    props.opacity = 0.0;
                    props.visible = false;
                },
            );
            with_props_mut(
                scene,
                self.target_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.target_props.clone();
                    props.opacity = snapshot.final_target_opacity;
                    props.visible = true;
                },
            );
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            if let Some(target) = scene.get_tattva_typed_mut::<EquationLayout>(self.target_id) {
                target.state.parts = snapshot.original_target_parts.clone();
                target.mark_dirty(DirtyFlags::TEXT_LAYOUT | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            }
            with_props_mut(
                scene,
                self.source_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.source_props.clone();
                },
            );
            with_props_mut(
                scene,
                self.target_id,
                DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
                |props| {
                    *props = snapshot.target_props.clone();
                },
            );
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}

#[derive(Clone)]
enum MatrixSelection {
    Cells(Vec<(usize, usize)>),
    Row(usize),
    Column(usize),
}

#[derive(Clone)]
struct MatrixStepSnapshot {
    original: Matrix,
}

pub struct MatrixStep {
    pub target_id: TattvaId,
    pub ease: Ease,
    pub highlight: Vec4,
    pub dim_opacity: f32,
    selection: MatrixSelection,
    snapshot: Option<MatrixStepSnapshot>,
}

impl MatrixStep {
    pub fn cells(
        target_id: TattvaId,
        cells: Vec<(usize, usize)>,
        highlight: Vec4,
        dim_opacity: f32,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            ease,
            highlight,
            dim_opacity,
            selection: MatrixSelection::Cells(cells),
            snapshot: None,
        }
    }

    pub fn row(
        target_id: TattvaId,
        row: usize,
        highlight: Vec4,
        dim_opacity: f32,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            ease,
            highlight,
            dim_opacity,
            selection: MatrixSelection::Row(row),
            snapshot: None,
        }
    }

    pub fn column(
        target_id: TattvaId,
        col: usize,
        highlight: Vec4,
        dim_opacity: f32,
        ease: Ease,
    ) -> Self {
        Self {
            target_id,
            ease,
            highlight,
            dim_opacity,
            selection: MatrixSelection::Column(col),
            snapshot: None,
        }
    }

    fn is_selected(&self, layout: &MatrixCellLayout) -> bool {
        match &self.selection {
            MatrixSelection::Cells(cells) => cells.contains(&(layout.row, layout.col)),
            MatrixSelection::Row(row) => layout.row == *row,
            MatrixSelection::Column(col) => layout.col == *col,
        }
    }
}

impl Animation for MatrixStep {
    fn on_start(&mut self, scene: &mut Scene) {
        let Some(matrix) = scene.get_tattva_typed::<Matrix>(self.target_id) else {
            return;
        };
        self.snapshot = Some(MatrixStepSnapshot {
            original: matrix.state.clone(),
        });
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let eased = self.ease.eval(t);
        let layout = snapshot.original.layout_snapshot();

        if let Some(matrix) = scene.get_tattva_typed_mut::<Matrix>(self.target_id) {
            matrix.state = snapshot.original.clone();
            for cell_layout in &layout {
                let Some(cell) = matrix.state.cell_mut(cell_layout.row, cell_layout.col) else {
                    continue;
                };
                if self.is_selected(cell_layout) {
                    let mut highlight = self.highlight;
                    highlight.w *= eased;
                    cell.highlight = Some(highlight);
                    cell.scale = cell.scale + 0.12 * eased;
                    cell.opacity = cell.opacity + (1.0 - cell.opacity) * (eased * 0.35);
                } else {
                    cell.highlight = None;
                    cell.opacity =
                        cell.opacity + (self.dim_opacity.clamp(0.1, 1.0) - cell.opacity) * eased;
                }
            }
            matrix.mark_dirty(
                DirtyFlags::TEXT_LAYOUT
                    | DirtyFlags::BOUNDS
                    | DirtyFlags::STYLE
                    | DirtyFlags::GEOMETRY,
            );
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        self.apply_at(scene, 1.0);
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(snapshot) = &self.snapshot {
            if let Some(matrix) = scene.get_tattva_typed_mut::<Matrix>(self.target_id) {
                matrix.state = snapshot.original.clone();
                matrix.mark_dirty(
                    DirtyFlags::TEXT_LAYOUT
                        | DirtyFlags::BOUNDS
                        | DirtyFlags::STYLE
                        | DirtyFlags::GEOMETRY,
                );
            }
        }
    }
}

pub struct MorphGeometry {
    pub source_id: TattvaId,
    pub target_id: TattvaId,
    pub ease: Ease,
    source_path: Option<Path>,
    target_path: Option<Path>,
    original_target_tattva: Option<Box<dyn TattvaTrait>>,
}

impl MorphGeometry {
    pub fn new(source_id: TattvaId, target_id: TattvaId, ease: Ease) -> Self {
        Self {
            source_id,
            target_id,
            ease,
            source_path: None,
            target_path: None,
            original_target_tattva: None,
        }
    }

    fn morph_segment_budget(source: &Path, target: &Path) -> usize {
        let existing = source.segments.len().max(target.segments.len());

        // Geometry morphs look noticeably smoother when we interpolate between
        // denser path representations instead of the primitive authoring counts.
        // Closed shapes benefit the most because corners and arcs get more
        // intermediate control points, reducing the "chunky" feel in motion.
        let minimum = if source.closed || target.closed {
            48
        } else {
            24
        };
        existing.max(minimum)
    }

    fn try_get_path(scene: &Scene, id: TattvaId) -> Option<Path> {
        let tattva = scene.get_tattva_any(id)?;
        let any = tattva.as_any();

        // Check if it's already a Path
        if let Some(p) = any.downcast_ref::<Tattva<Path>>() {
            return Some(p.state.clone());
        }

        // Try common primitives
        if let Some(p) = any.downcast_ref::<Tattva<Rectangle>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<RoundedRectangle>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<Circle>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<Square>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<Ellipse>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<Polygon>>() {
            return Some(p.state.to_path());
        }
        if let Some(p) = any.downcast_ref::<Tattva<Line>>() {
            return Some(p.state.to_path());
        }

        None
    }
}

impl Animation for MorphGeometry {
    fn on_start(&mut self, scene: &mut Scene) {
        let mut source_path = match Self::try_get_path(scene, self.source_id) {
            Some(p) => p,
            None => {
                println!("Error: Could not get path for source {}", self.source_id);
                return;
            }
        };
        let mut target_path = match Self::try_get_path(scene, self.target_id) {
            Some(p) => p,
            None => {
                println!("Error: Could not get path for target {}", self.target_id);
                return;
            }
        };

        // Densify the path pair before interpolation so simultaneous morph +
        // translation feels continuous instead of stepping through a sparse mesh.
        let morph_segments = Self::morph_segment_budget(&source_path, &target_path);
        source_path.resample(morph_segments);
        target_path.resample(morph_segments);

        // Align starting points to minimize travel distance
        source_path = source_path.align_to(&target_path);

        self.source_path = Some(source_path.clone());
        self.target_path = Some(target_path);

        // Displace the current target with a Path Tattva
        if let Some(tattva_box) = scene.tattvas.remove(&self.target_id) {
            let shared_props = tattva_box.props().clone();

            // Ensure target is visible and opaque via the SHARED props
            {
                let mut props = shared_props.write();
                props.visible = true;
                props.opacity = 1.0;
            }

            // Create a new Tattva<Path> but REUSE the original SharedProps Arc
            let intermediate = Tattva {
                id: self.target_id,
                state: source_path,
                props: shared_props,
                dirty: DirtyFlags::ALL,
            };

            // Hide the source shape
            if let Some(source) = scene.get_tattva_any_mut(self.source_id) {
                let mut s_props = source.props().write();
                s_props.visible = false;
                s_props.opacity = 0.0;
            }

            // We have to store the original and replace it in the scene
            self.original_target_tattva = Some(tattva_box);
            scene.replace_tattva(self.target_id, Box::new(intermediate));
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let (Some(s_path), Some(t_path)) = (&self.source_path, &self.target_path) else {
            return;
        };

        let current_path = s_path.lerp(t_path, self.ease.eval(t));

        if let Some(tattva) = scene.get_tattva_typed_mut::<Path>(self.target_id) {
            tattva.state = current_path;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        // Swap back the original target Tattva
        if let Some(original) = self.original_target_tattva.take() {
            scene.replace_tattva(self.target_id, original);

            // Marks geometry as dirty since it's back to original type
            if let Some(t) = scene.get_tattva_any_mut(self.target_id) {
                {
                    let mut props = t.props().write();
                    props.visible = true;
                    props.opacity = 1.0;
                }
                t.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            }
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(original) = self.original_target_tattva.take() {
            scene.replace_tattva(self.target_id, original);
            if let Some(target) = scene.get_tattva_any_mut(self.target_id) {
                target.mark_dirty(DirtyFlags::ALL);
            }
        }
    }
}

/// WritePath animation: traces the path outline and fills the completed sector
/// The fill appears as a growing sector as the path is drawn (Manim-style)
pub struct WritePath {
    pub target_id: TattvaId,
    pub ease: Ease,
    /// Set when target is not a Path — shadow tattva owned by this animation
    shadow_id: Option<TattvaId>,
    /// Original visibility of the target, preserved so reset/finish can restore it
    original_visibility: Option<bool>,
}

impl WritePath {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            shadow_id: None,
            original_visibility: None,
        }
    }

    /// Returns the active tattva ID to drive trim on (shadow if present, else target).
    fn active_id(&self) -> TattvaId {
        self.shadow_id.unwrap_or(self.target_id)
    }

    /// Try to derive a Path from the target tattva via downcasting.
    /// Returns None if the type is not convertible.
    fn derive_path(scene: &Scene, id: TattvaId) -> Option<Path> {
        use crate::frontend::collection::primitives::to_path::ToPath;
        let tattva = scene.get_tattva_any(id)?;
        let any = tattva.as_any();
        if let Some(t) = any.downcast_ref::<Tattva<Circle>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<Rectangle>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<RoundedRectangle>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<Square>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<Ellipse>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<Polygon>>() {
            return Some(t.state.to_path());
        }
        if let Some(t) = any.downcast_ref::<Tattva<Line>>() {
            return Some(t.state.to_path());
        }
        None
    }

    fn cleanup(&mut self, scene: &mut Scene) {
        // Remove shadow tattva
        if let Some(sid) = self.shadow_id.take() {
            scene.remove_tattva(sid);
        }
        // Restore original visibility
        if let Some(vis) = self.original_visibility.take() {
            if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
                let mut props = DrawableProps::write(tattva.props());
                props.visible = vis;
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY);
            }
        }
    }
}

impl Animation for WritePath {
    fn prepare(&mut self, scene: &mut Scene) {
        self.reset(scene);
    }

    fn on_start(&mut self, scene: &mut Scene) {
        // If target is already a Path, drive it directly
        if scene.get_tattva_typed_mut::<Path>(self.target_id).is_some() {
            if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
                let mut props = DrawableProps::write(tattva.props());
                self.original_visibility = Some(props.visible);
                props.visible = true; // must be visible for trim animation to show
                props.opacity = 1.0; // Ensure it's not hidden via opacity
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY | DirtyFlags::STYLE);
            }
            if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.target_id) {
                path.state.trim_start = 0.0;
                path.state.trim_end = 0.0;
                path.state.fill_opacity = 0.0;
                path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            }
            return;
        }

        // Otherwise derive a shadow path, preserving original visibility
        if let Some(derived) = Self::derive_path(scene, self.target_id) {
            // Store and hide original
            let vis = {
                let tattva = scene.get_tattva_any_mut(self.target_id).unwrap();
                let mut props = DrawableProps::write(tattva.props());
                let v = props.visible;
                props.visible = false;
                props.opacity = 0.0; // Ensure it's completely out of the rendering pipeline
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY | DirtyFlags::STYLE);
                v
            };
            self.original_visibility = Some(vis);

            // Add shadow with trim at 0
            let mut shadow = derived;
            shadow.trim_start = 0.0;
            shadow.trim_end = 0.0;
            shadow.fill_opacity = 0.0;

            // Get position from original
            let pos = {
                let t = scene.get_tattva_any(self.target_id).unwrap();
                DrawableProps::read(t.props()).position
            };
            let sid = scene.add_tattva(shadow, pos);
            self.shadow_id = Some(sid);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.active_id()) {
            path.state.trim_end = eased_t;
            // Delay fill fade slightly for better visual establishing of the outline
            path.state.fill_opacity = eased_t.powi(2);
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.active_id()) {
            path.state.trim_end = 1.0;
            path.state.fill_opacity = 1.0;
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
        // Restore original and remove shadow (shadow path case)
        if self.shadow_id.is_some() {
            if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
                let mut props = DrawableProps::write(tattva.props());
                props.visible = true;
                props.opacity = 1.0;
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY | DirtyFlags::STYLE);
            }
            // Write should end with the original target visible, so don't let
            // cleanup restore the pre-animation hidden state from reset().
            self.original_visibility = None;
            self.cleanup(scene);
        } else {
            // Direct Path case — restore visibility
            if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
                let mut props = DrawableProps::write(tattva.props());
                props.visible = true;
                props.opacity = 1.0;
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY | DirtyFlags::STYLE);
            }
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        self.cleanup(scene);
        // Direct Path case reset
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.target_id) {
            path.state.trim_start = 0.0;
            path.state.trim_end = 0.0;
            path.state.fill_opacity = 0.0;
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }

        // Ensure the original object is hidden before the animation begins
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = false;
                props.opacity = 0.0;
            },
        );
    }
}

/// UnwritePath animation: reverses the write effect
/// Erases the outline from the end while keeping the fill in the remaining portion
pub struct UnwritePath {
    pub target_id: TattvaId,
    pub ease: Ease,
    shadow_id: Option<TattvaId>,
    original_visibility: Option<bool>,
}

impl UnwritePath {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            shadow_id: None,
            original_visibility: None,
        }
    }

    fn active_id(&self) -> TattvaId {
        self.shadow_id.unwrap_or(self.target_id)
    }

    fn cleanup(&mut self, scene: &mut Scene) {
        if let Some(sid) = self.shadow_id.take() {
            scene.remove_tattva(sid);
        }
        if let Some(vis) = self.original_visibility.take() {
            if let Some(tattva) = scene.get_tattva_any_mut(self.target_id) {
                let mut props = DrawableProps::write(tattva.props());
                props.visible = vis;
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY);
            }
        }
    }
}

impl Animation for UnwritePath {
    fn on_start(&mut self, scene: &mut Scene) {
        if scene.get_tattva_typed_mut::<Path>(self.target_id).is_some() {
            if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.target_id) {
                path.state.trim_start = 0.0;
                path.state.trim_end = 1.0;
                path.state.fill_opacity = 1.0;
                path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            }
            return;
        }

        if let Some(derived) = WritePath::derive_path(scene, self.target_id) {
            let vis = {
                let tattva = scene.get_tattva_any_mut(self.target_id).unwrap();
                let mut props = DrawableProps::write(tattva.props());
                let v = props.visible;
                props.visible = false;
                props.opacity = 0.0; // Ensure it's completely out of the rendering pipeline
                drop(props);
                tattva.mark_dirty(DirtyFlags::VISIBILITY | DirtyFlags::STYLE);
                v
            };
            self.original_visibility = Some(vis);

            let mut shadow = derived;
            shadow.trim_start = 0.0;
            shadow.trim_end = 1.0;
            shadow.fill_opacity = 1.0;

            let pos = {
                let t = scene.get_tattva_any(self.target_id).unwrap();
                DrawableProps::read(t.props()).position
            };
            let sid = scene.add_tattva(shadow, pos);
            self.shadow_id = Some(sid);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.active_id()) {
            path.state.trim_start = 0.0;
            path.state.trim_end = 1.0 - eased_t;
            // Keep the fill alpha stable and let the trimmed fill geometry
            // create the visual "erasing" effect for closed shapes.
            path.state.fill_opacity = 1.0;
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.active_id()) {
            path.state.trim_start = 0.0;
            path.state.trim_end = 0.0;
            path.state.fill_opacity = 1.0;
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }

        // IMPORTANT: Clear original visibility so cleanup doesn't restore it to TRUE
        self.original_visibility = None;

        // Ensure the target is hidden
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = false;
                props.opacity = 0.0;
            },
        );

        self.cleanup(scene);
    }

    fn reset(&mut self, scene: &mut Scene) {
        self.cleanup(scene);
        // Reset original visibility to visible (Unwrite goes from visible to invisible)
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.opacity = 1.0;
            },
        );
        if let Some(path) = scene.get_tattva_typed_mut::<Path>(self.target_id) {
            path.state.trim_start = 0.0;
            path.state.trim_end = 1.0;
            path.state.fill_opacity = 1.0;
            path.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

/// WriteText animation: reveals text character by character (typewriter effect)
pub struct WriteText {
    pub target_id: TattvaId,
    pub ease: Ease,
}

impl WriteText {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self { target_id, ease }
    }
}

impl Animation for WriteText {
    fn prepare(&mut self, scene: &mut Scene) {
        self.reset(scene);
    }

    fn on_start(&mut self, scene: &mut Scene) {
        with_props_mut(
            scene,
            self.target_id,
            DirtyFlags::STYLE | DirtyFlags::VISIBILITY,
            |props| {
                props.visible = true;
                props.opacity = 1.0;
            },
        );

        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try CodeBlock
        if let Some(cb) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::code_block::CodeBlock>(
                self.target_id,
            )
        {
            cb.state.char_reveal = 0.0;
            cb.state.typewriter_mode = true;
            cb.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);

        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = eased_t;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = eased_t;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try CodeBlock
        if let Some(cb) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::code_block::CodeBlock>(
                self.target_id,
            )
        {
            cb.state.char_reveal = eased_t;
            cb.state.typewriter_mode = true;
            cb.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try CodeBlock
        if let Some(cb) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::code_block::CodeBlock>(
                self.target_id,
            )
        {
            cb.state.char_reveal = 0.0;
            cb.state.typewriter_mode = true;
            cb.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

/// UnwriteText animation: hides text character by character (reverse typewriter effect)
pub struct UnwriteText {
    pub target_id: TattvaId,
    pub ease: Ease,
}

impl UnwriteText {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self { target_id, ease }
    }
}

impl Animation for UnwriteText {
    fn on_start(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);

        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0 - eased_t;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0 - eased_t;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try CodeBlock
        if let Some(cb) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::code_block::CodeBlock>(
                self.target_id,
            )
        {
            cb.state.char_reveal = 1.0 - eased_t;
            cb.state.typewriter_mode = true;
            cb.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try CodeBlock
        if let Some(cb) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::code_block::CodeBlock>(
                self.target_id,
            )
        {
            cb.state.char_reveal = 0.0;
            cb.state.typewriter_mode = true;
            cb.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = true;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = true;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

/// RevealText animation: reveals text character by character with shifting effect
/// Text grows from center, shifting as characters are revealed
pub struct RevealText {
    pub target_id: TattvaId,
    pub ease: Ease,
}

impl RevealText {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self { target_id, ease }
    }
}

impl Animation for RevealText {
    fn prepare(&mut self, scene: &mut Scene) {
        self.reset(scene);
    }

    fn on_start(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);

        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = eased_t;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = eased_t;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

/// UnrevealText animation: hides text character by character with shifting effect
/// Text shrinks to center, shifting as characters are hidden
pub struct UnrevealText {
    pub target_id: TattvaId,
    pub ease: Ease,
}

impl UnrevealText {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self { target_id, ease }
    }
}

impl Animation for UnrevealText {
    fn on_start(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        let eased_t = self.ease.eval(t);

        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0 - eased_t;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0 - eased_t;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 0.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 0.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        // Try Label first
        if let Some(label) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::label::Label>(self.target_id)
        {
            label.state.char_reveal = 1.0;
            label.state.typewriter_mode = false;
            label.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
            return;
        }

        // Try LaTeX
        if let Some(latex) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::text::latex::Latex>(self.target_id)
        {
            latex.state.char_reveal = 1.0;
            latex.state.typewriter_mode = false;
            latex.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::STYLE);
        }
    }
}

/// WriteTable animation: draws grid lines then writes text content
/// Grid lines are drawn first (horizontal then vertical), then text appears cell by cell
pub struct WriteTable {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
}

impl WriteTable {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
        }
    }
}

impl Animation for WriteTable {
    fn prepare(&mut self, scene: &mut Scene) {
        self.reset(scene);
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            self.from = Some(tattva.state.write_progress());
            tattva.state.set_write_progress(0.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            self.from = Some(tattva.state.write_progress());
            tattva.state.set_write_progress(0.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            let from = self.from.unwrap_or(0.0);
            let eased_t = self.ease.eval(t);
            tattva
                .state
                .set_write_progress(from + (1.0 - from) * eased_t);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            let from = self.from.unwrap_or(0.0);
            let eased_t = self.ease.eval(t);
            tattva
                .state
                .set_write_progress(from + (1.0 - from) * eased_t);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            tattva.state.set_write_progress(1.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            tattva.state.set_write_progress(1.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            tattva.state.set_write_progress(self.from.unwrap_or(0.0));
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            tattva.state.set_write_progress(self.from.unwrap_or(0.0));
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

/// UnwriteTable animation: reverses the write animation
pub struct UnwriteTable {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
}

impl UnwriteTable {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
        }
    }
}

impl Animation for UnwriteTable {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            self.from = Some(tattva.state.write_progress());
            tattva.state.set_write_progress(1.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            self.from = Some(tattva.state.write_progress());
            tattva.state.set_write_progress(1.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            let from = self.from.unwrap_or(1.0);
            let eased_t = self.ease.eval(t);
            tattva.state.set_write_progress(from - from * eased_t);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            let from = self.from.unwrap_or(1.0);
            let eased_t = self.ease.eval(t);
            tattva.state.set_write_progress(from - from * eased_t);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(tattva) =
            scene.get_tattva_typed_mut::<crate::frontend::collection::table::Table>(self.target_id)
        {
            tattva.state.set_write_progress(self.from.unwrap_or(1.0));
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            return;
        }
        if let Some(tattva) = scene
            .get_tattva_typed_mut::<crate::frontend::collection::table::TableV1>(self.target_id)
        {
            tattva.state.set_write_progress(self.from.unwrap_or(1.0));
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}
/// WriteSurface animation: progressively reveals a parametric surface
pub struct WriteSurface {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
}

impl WriteSurface {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
        }
    }
}

impl Animation for WriteSurface {
    fn prepare(&mut self, scene: &mut Scene) {
        self.reset(scene);
    }

    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            self.from = Some(tattva.state.write_progress);
            tattva.state.write_progress = 0.0;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            let from = self.from.unwrap_or(0.0);
            let eased_t = self.ease.eval(t);
            tattva.state.write_progress = from + (1.0 - from) * eased_t;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn on_finish(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            tattva.state.write_progress = 1.0;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            tattva.state.write_progress = self.from.unwrap_or(0.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}

/// UnwriteSurface animation: reverses the write animation
pub struct UnwriteSurface {
    pub target_id: TattvaId,
    pub ease: Ease,
    from: Option<f32>,
}

impl UnwriteSurface {
    pub fn new(target_id: TattvaId, ease: Ease) -> Self {
        Self {
            target_id,
            ease,
            from: None,
        }
    }
}

impl Animation for UnwriteSurface {
    fn on_start(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            self.from = Some(tattva.state.write_progress);
            tattva.state.write_progress = 1.0;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            let from = self.from.unwrap_or(1.0);
            let eased_t = self.ease.eval(t);
            tattva.state.write_progress = from - from * eased_t;
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }

    fn reset(&mut self, scene: &mut Scene) {
        if let Some(tattva) = scene.get_tattva_typed_mut::<crate::frontend::collection::graph::parametric_surface::ParametricSurface>(self.target_id) {
            tattva.state.write_progress = self.from.unwrap_or(1.0);
            tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    }
}
