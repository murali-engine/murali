use crate::engine::camera::Camera;
use crate::engine::scene::Scene;
use crate::frontend::DirtyFlags;
use crate::frontend::TattvaId;
use crate::frontend::animation::{
    Animation, Ease, RunReversibleSceneCallback, RunReversibleSceneCallbackOverTime,
    RunSceneCallback, RunSceneCallbackOverTime, builder::AnimationBuilder,
    camera_animation_builder::CameraAnimationBuilder,
};
use crate::frontend::collection::math::equation::VectorEquationHandle;
use crate::frontend::props::DrawableProps;
use crate::validation::ValidationError;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SeekError {
    #[error(transparent)]
    InvalidTimeline(#[from] ValidationError),
    #[error("cannot seek across non-reversible timeline effect `{effect}` at {start_time:.3}s")]
    NonReversibleTimelineEffect {
        effect: &'static str,
        start_time: f32,
    },
    #[error("cannot seek a scene with {count} frame-dependent updater(s)")]
    FrameDependentUpdaters { count: usize },
    #[error("cannot seek a scene containing {count} history-dependent traced path(s)")]
    HistoryDependentTracedPaths { count: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimState {
    Pending,
    Running,
    Done,
}

pub struct ScheduledAnimation {
    pub order: usize,
    pub start_time: f32,
    pub duration: f32,
    pub anim: Box<dyn Animation>,
    pub state: AnimState,
    initialized: bool,
}

impl ScheduledAnimation {
    pub fn new(order: usize, start_time: f32, duration: f32, anim: Box<dyn Animation>) -> Self {
        Self {
            order,
            start_time,
            duration: duration.max(0.0),
            anim,
            state: AnimState::Pending,
            initialized: false,
        }
    }
}

struct TimelineBaseline {
    drawable_props: HashMap<TattvaId, DrawableProps>,
    camera: Camera,
}

pub struct Timeline {
    pub scheduled: Vec<ScheduledAnimation>,
    next_order: usize,
    baseline: Option<TimelineBaseline>,
    current_time: Option<f32>,
    hold_until: f32,
    composition_cursor: f32,
    overlay_origin: Option<f32>,
    authoring_errors: Vec<ValidationError>,
}

/// A reusable animation schedule authored in its own local time frame.
///
/// Every clip begins at local time `0.0`. Adding it to a [`Timeline`] converts
/// its local animation times into absolute scene times before playback.
pub struct Clip {
    timeline: Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalPlaybackMode {
    Once,
    RoundTrip,
    Loop { repeats: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct SignalPlayback {
    pub start_time: f32,
    pub duration: f32,
    pub ease: Ease,
    pub mode: SignalPlaybackMode,
}

impl SignalPlayback {
    pub fn once(start_time: f32, duration: f32, ease: Ease) -> Self {
        Self {
            start_time,
            duration,
            ease,
            mode: SignalPlaybackMode::Once,
        }
    }

    pub fn round_trip(start_time: f32, duration: f32, ease: Ease) -> Self {
        Self {
            start_time,
            duration,
            ease,
            mode: SignalPlaybackMode::RoundTrip,
        }
    }

    pub fn looped(start_time: f32, duration: f32, repeats: usize, ease: Ease) -> Self {
        Self {
            start_time,
            duration,
            ease,
            mode: SignalPlaybackMode::Loop {
                repeats: repeats.max(1),
            },
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
            next_order: 0,
            baseline: None,
            current_time: None,
            hold_until: 0.0,
            composition_cursor: 0.0,
            overlay_origin: None,
            authoring_errors: Vec::new(),
        }
    }

    pub fn add_animation(&mut self, start_time: f32, duration: f32, anim: Box<dyn Animation>) {
        if let Err(error) = self.try_add_animation(start_time, duration, anim) {
            self.authoring_errors.push(error);
        }
    }

    /// Adds an animation or returns the invalid authored value immediately.
    pub fn try_add_animation(
        &mut self,
        start_time: f32,
        duration: f32,
        anim: Box<dyn Animation>,
    ) -> Result<(), ValidationError> {
        Self::validate_schedule(start_time, duration)?;
        let order = self.next_order;
        self.next_order += 1;
        self.scheduled
            .push(ScheduledAnimation::new(order, start_time, duration, anim));
        self.scheduled.sort_by(|a, b| {
            a.start_time
                .total_cmp(&b.start_time)
                .then(a.order.cmp(&b.order))
        });
        self.composition_cursor = self.composition_cursor.max(start_time + duration.max(0.0));
        self.overlay_origin = None;
        Ok(())
    }

    fn validate_schedule(start_time: f32, duration: f32) -> Result<(), ValidationError> {
        if !start_time.is_finite() {
            return Err(ValidationError::non_finite(
                "Timeline",
                "start_time",
                start_time,
            ));
        }
        if !duration.is_finite() {
            return Err(ValidationError::non_finite(
                "Timeline", "duration", duration,
            ));
        }
        let end_time = start_time + duration.max(0.0);
        if !end_time.is_finite() {
            return Err(ValidationError::non_finite(
                "Timeline", "end_time", end_time,
            ));
        }
        Ok(())
    }

    /// Returns the first rejected authoring value, if any.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(error) = self.authoring_errors.first() {
            return Err(error.clone());
        }
        for scheduled in &self.scheduled {
            Self::validate_schedule(scheduled.start_time, scheduled.duration)?;
        }
        Ok(())
    }

    pub(crate) fn validate_for_scene(&self, scene: &Scene) -> Result<(), ValidationError> {
        for scheduled in &self.scheduled {
            scheduled.anim.validate(scene)?;
        }
        Ok(())
    }

    /// Values rejected while using the infallible fluent authoring helpers.
    pub fn authoring_errors(&self) -> &[ValidationError] {
        &self.authoring_errors
    }

    /// Places a clip at the end of the current composition.
    pub fn append(&mut self, clip: Clip) -> &mut Self {
        let start_time = self.composition_cursor;
        let duration = clip.duration();
        self.merge_clip(start_time, clip);
        self.overlay_origin = Some(start_time);
        self.composition_cursor = self.composition_cursor.max(start_time + duration);
        self
    }

    /// Places a clip alongside the current composition group.
    ///
    /// After an [`append`](Self::append), this uses the same start time as the
    /// appended clip. Its duration extends the cursor when necessary.
    pub fn overlay(&mut self, clip: Clip) -> &mut Self {
        let start_time = self.overlay_origin.unwrap_or(self.composition_cursor);
        let duration = clip.duration();
        self.merge_clip(start_time, clip);
        self.overlay_origin = Some(start_time);
        self.composition_cursor = self.composition_cursor.max(start_time + duration);
        self
    }

    /// Places a clip at an explicit absolute scene time without moving the
    /// sequential composition cursor.
    pub fn place_at(&mut self, start_time: f32, clip: Clip) -> &mut Self {
        if !start_time.is_finite() {
            self.authoring_errors.push(ValidationError::non_finite(
                "Timeline",
                "clip_start_time",
                start_time,
            ));
            return self;
        }
        self.merge_clip(start_time, clip);
        self
    }

    /// Returns the absolute time where the next appended clip will begin.
    pub fn cursor(&self) -> f32 {
        self.composition_cursor
    }

    fn merge_clip(&mut self, start_time: f32, mut clip: Clip) {
        self.authoring_errors
            .append(&mut clip.timeline.authoring_errors);
        for mut scheduled in clip.timeline.scheduled.drain(..) {
            let absolute_start = scheduled.start_time + start_time;
            if let Err(error) = Self::validate_schedule(absolute_start, scheduled.duration) {
                self.authoring_errors.push(error);
                continue;
            }
            scheduled.order = self.next_order;
            scheduled.start_time = absolute_start;
            scheduled.state = AnimState::Pending;
            scheduled.initialized = false;
            self.next_order += 1;
            self.scheduled.push(scheduled);
        }

        let absolute_hold = start_time + clip.timeline.hold_until.max(0.0);
        if absolute_hold.is_finite() {
            self.hold_until = self.hold_until.max(absolute_hold);
        } else {
            self.authoring_errors.push(ValidationError::non_finite(
                "Timeline",
                "clip_end_time",
                absolute_hold,
            ));
        }
        self.scheduled.sort_by(|a, b| {
            a.start_time
                .total_cmp(&b.start_time)
                .then(a.order.cmp(&b.order))
        });
    }

    fn capture_baseline(&mut self, scene: &Scene) {
        if self.baseline.is_some() {
            return;
        }
        let drawable_props = scene
            .tattvas
            .iter()
            .map(|(id, tattva)| (*id, DrawableProps::read(tattva.props()).clone()))
            .collect();
        self.baseline = Some(TimelineBaseline {
            drawable_props,
            camera: scene.camera,
        });
    }

    pub(crate) fn prepare(&mut self, scene: &mut Scene) {
        for scheduled in &mut self.scheduled {
            scheduled.anim.prepare(scene);
        }
    }

    fn evaluate_at(&mut self, scene_time: f32, scene: &mut Scene) {
        for sa in &mut self.scheduled {
            let elapsed = scene_time - sa.start_time;

            if elapsed < 0.0 {
                sa.state = AnimState::Pending;
                continue;
            }

            // Trigger initialization if this is the first time we hit this animation
            if !sa.initialized {
                sa.anim.on_start(scene);
                sa.initialized = true;
            }

            if sa.duration <= f32::EPSILON || elapsed >= sa.duration {
                if sa.state != AnimState::Done {
                    sa.anim.apply_at(scene, 1.0); // Ensure it finishes exactly at 1.0
                    sa.anim.on_finish(scene);
                    sa.state = AnimState::Done;
                } else if sa.anim.reapplies_terminal_state() {
                    sa.anim.apply_at(scene, 1.0);
                }
            } else {
                sa.state = AnimState::Running;
                let t = (elapsed / sa.duration).clamp(0.0, 1.0);
                sa.anim.apply_at(scene, t); // Pass normalized 0.0 -> 1.0
            }
        }
    }

    fn evaluate_forward(&mut self, after_time: Option<f32>, target_time: f32, scene: &mut Scene) {
        let mut crossed_starts = self
            .scheduled
            .iter()
            .map(|scheduled| scheduled.start_time)
            .filter(|start_time| {
                *start_time <= target_time
                    && after_time.is_none_or(|after_time| *start_time > after_time)
            })
            .collect::<Vec<_>>();
        crossed_starts.sort_by(f32::total_cmp);
        crossed_starts.dedup_by(|a, b| a.total_cmp(b).is_eq());

        for start_time in crossed_starts.iter().copied() {
            scene.scene_time = start_time;
            self.evaluate_at(start_time, scene);
        }
        scene.scene_time = target_time;
        if crossed_starts
            .last()
            .is_none_or(|start_time| start_time.total_cmp(&target_time).is_ne())
        {
            self.evaluate_at(target_time, scene);
        }
    }

    fn restore_baseline(&mut self, scene: &mut Scene) {
        for scheduled in self.scheduled.iter_mut().rev() {
            scheduled.anim.reset(scene);
            scheduled.initialized = false;
            scheduled.state = AnimState::Pending;
        }
        self.current_time = None;

        let Some(baseline) = &self.baseline else {
            return;
        };
        scene.camera = baseline.camera;
        for (id, original_props) in &baseline.drawable_props {
            if let Some(tattva) = scene.get_tattva_any_mut(*id) {
                *DrawableProps::write(tattva.props()) = original_props.clone();
                tattva.mark_dirty(DirtyFlags::ALL);
            }
        }
    }

    fn validate_seek(&self, scene_time: f32) -> Result<(), SeekError> {
        let traversed_until = self.current_time.unwrap_or(scene_time).max(scene_time);
        if let Some((scheduled, effect)) = self.scheduled.iter().find_map(|scheduled| {
            (scheduled.start_time <= traversed_until)
                .then(|| {
                    scheduled
                        .anim
                        .seek_blocker()
                        .map(|effect| (scheduled, effect))
                })
                .flatten()
        }) {
            return Err(SeekError::NonReversibleTimelineEffect {
                effect,
                start_time: scheduled.start_time,
            });
        }
        Ok(())
    }

    /// Advances the timeline. Moving to an earlier time reconstructs the
    /// scene from its captured baseline before evaluating the target time.
    pub fn update(&mut self, scene_time: f32, scene: &mut Scene) -> Result<(), SeekError> {
        self.validate()?;
        self.capture_baseline(scene);
        let moving_backward = self
            .current_time
            .is_some_and(|current_time| scene_time < current_time);
        if moving_backward {
            self.validate_seek(scene_time)?;
            self.restore_baseline(scene);
        }
        let after_time = if moving_backward {
            None
        } else {
            self.current_time
        };
        self.evaluate_forward(after_time, scene_time, scene);
        self.current_time = Some(scene_time);
        Ok(())
    }

    /// Reconstructs the authored scene at an absolute timeline time.
    pub fn seek_to(&mut self, scene_time: f32, scene: &mut Scene) -> Result<(), SeekError> {
        self.validate()?;
        self.validate_seek(scene_time)?;
        self.capture_baseline(scene);
        self.restore_baseline(scene);
        self.evaluate_forward(None, scene_time, scene);
        self.current_time = Some(scene_time);
        Ok(())
    }

    // --- Fluent API Helpers ---

    pub fn animate(&mut self, id: TattvaId) -> AnimationBuilder<'_> {
        AnimationBuilder::new(self, id)
    }

    pub fn animate_camera(&mut self) -> CameraAnimationBuilder<'_> {
        CameraAnimationBuilder::new(self)
    }

    pub fn call_at<F>(&mut self, time: f32, callback: F)
    where
        F: FnMut(&mut Scene) + Send + 'static,
    {
        self.add_animation(time, 0.0, Box::new(RunSceneCallback::new(callback)));
    }

    pub fn call_at_reversible<F, R>(&mut self, time: f32, callback: F, reset: R)
    where
        F: FnMut(&mut Scene) + Send + 'static,
        R: FnMut(&mut Scene) + Send + 'static,
    {
        self.add_animation(
            time,
            0.0,
            Box::new(RunReversibleSceneCallback::new(callback, reset)),
        );
    }

    pub fn call_during<F>(&mut self, start_time: f32, duration: f32, callback: F)
    where
        F: FnMut(&mut Scene, f32) + Send + 'static,
    {
        self.add_animation(
            start_time,
            duration.max(0.0),
            Box::new(RunSceneCallbackOverTime::new(callback)),
        );
    }

    pub fn call_during_reversible<F, R>(
        &mut self,
        start_time: f32,
        duration: f32,
        callback: F,
        reset: R,
    ) where
        F: FnMut(&mut Scene, f32) + Send + 'static,
        R: FnMut(&mut Scene) + Send + 'static,
    {
        self.add_animation(
            start_time,
            duration.max(0.0),
            Box::new(RunReversibleSceneCallbackOverTime::new(callback, reset)),
        );
    }

    pub fn play_signal(&mut self, id: TattvaId, playback: SignalPlayback) {
        match playback.mode {
            SignalPlaybackMode::Once => {
                self.animate(id)
                    .at(playback.start_time)
                    .for_duration(playback.duration)
                    .ease(playback.ease)
                    .propagate()
                    .spawn();
            }
            SignalPlaybackMode::RoundTrip => {
                self.animate(id)
                    .at(playback.start_time)
                    .for_duration(playback.duration)
                    .ease(playback.ease)
                    .propagate()
                    .spawn();

                self.animate(id)
                    .at(playback.start_time + playback.duration)
                    .for_duration(playback.duration)
                    .ease(playback.ease)
                    .propagate_to(0.0)
                    .spawn();
            }
            SignalPlaybackMode::Loop { repeats } => {
                for i in 0..repeats {
                    let cycle_start = playback.start_time + i as f32 * playback.duration;
                    self.animate(id)
                        .at(cycle_start)
                        .for_duration(playback.duration)
                        .ease(playback.ease)
                        .propagate()
                        .spawn();

                    if i + 1 < repeats {
                        self.animate(id)
                            .at(cycle_start + playback.duration)
                            .for_duration(0.0)
                            .ease(playback.ease)
                            .propagate_to(0.0)
                            .spawn();
                    }
                }
            }
        }
    }

    pub fn morph_matching(
        &mut self,
        sources: Vec<TattvaId>,
        targets: Vec<TattvaId>,
        scene: &crate::engine::scene::Scene,
        start_time: f32,
        duration: f32,
        ease: crate::frontend::animation::Ease,
    ) {
        use crate::frontend::props::DrawableProps;
        use std::collections::HashMap;

        let mut unmatched_sources = sources.clone();
        let mut unmatched_targets = targets.clone();
        let mut pairs = Vec::new();

        // 1. Match by Tag (Identity)
        let mut source_tags: HashMap<String, Vec<TattvaId>> = HashMap::new();
        for &id in &sources {
            if let Some(t) = scene.get_tattva_any(id) {
                if let Some(tag) = &DrawableProps::read(t.props()).tag {
                    source_tags.entry(tag.clone()).or_default().push(id);
                }
            }
        }

        let mut still_unmatched_targets = Vec::new();
        for &id in &targets {
            let mut matched = false;
            if let Some(t) = scene.get_tattva_any(id) {
                if let Some(tag) = &DrawableProps::read(t.props()).tag {
                    if let Some(ids) = source_tags.get_mut(tag) {
                        if let Some(source_id) = ids.pop() {
                            pairs.push((source_id, id));
                            unmatched_sources.retain(|&x| x != source_id);
                            matched = true;
                        }
                    }
                }
            }
            if !matched {
                still_unmatched_targets.push(id);
            }
        }
        unmatched_targets = still_unmatched_targets;

        // 2. Match by spatial proximity for the remainder
        let mut final_unmatched_targets = Vec::new();
        for &target_id in &unmatched_targets {
            let target_pos = scene
                .get_tattva_any(target_id)
                .map(|t| DrawableProps::read(t.props()).position)
                .unwrap_or_default();

            let mut best_source = None;
            let mut min_dist = f32::MAX;

            for (idx, &source_id) in unmatched_sources.iter().enumerate() {
                let source_pos = scene
                    .get_tattva_any(source_id)
                    .map(|t| DrawableProps::read(t.props()).position)
                    .unwrap_or_default();

                let dist = (target_pos - source_pos).length_squared();
                if dist < min_dist {
                    min_dist = dist;
                    best_source = Some(idx);
                }
            }

            if let Some(idx) = best_source {
                let source_id = unmatched_sources.remove(idx);
                pairs.push((source_id, target_id));
            } else {
                final_unmatched_targets.push(target_id);
            }
        }

        // 3. Bake animations into the timeline
        for (src, tgt) in pairs {
            // Morph geometry
            self.animate(tgt)
                .at(start_time)
                .for_duration(duration)
                .ease(ease)
                .morph_from(src)
                .spawn();

            // Move position to target
            let source_pos = scene
                .get_tattva_any(src)
                .map(|t| DrawableProps::read(t.props()).position)
                .unwrap_or_default();
            let target_pos = scene
                .get_tattva_any(tgt)
                .map(|t| DrawableProps::read(t.props()).position)
                .unwrap_or_default();

            self.animate(tgt)
                .at(start_time)
                .for_duration(duration)
                .ease(ease)
                .move_to(target_pos)
                .from_vec3(source_pos)
                .spawn();
        }

        // Fade out unmatched sources
        for src in unmatched_sources {
            self.animate(src)
                .at(start_time)
                .for_duration(duration * 0.5)
                .fade_to(0.0)
                .spawn();
        }

        // Fade in unmatched targets (new symbols)
        for tgt in final_unmatched_targets {
            self.animate(tgt)
                .at(start_time + duration * 0.5)
                .for_duration(duration * 0.5)
                .appear()
                .spawn();
        }
    }

    fn stage_targets(scene: &mut Scene, targets: &[TattvaId]) {
        for &id in targets {
            scene.hide_tattva(id);
        }
    }

    pub fn morph_matching_staged(
        &mut self,
        sources: Vec<TattvaId>,
        targets: Vec<TattvaId>,
        scene: &mut Scene,
        start_time: f32,
        duration: f32,
        ease: crate::frontend::animation::Ease,
    ) {
        Self::stage_targets(scene, &targets);
        self.morph_matching(sources, targets, scene, start_time, duration, ease);
    }

    pub fn morph_vector_equations(
        &mut self,
        source: &VectorEquationHandle,
        target: &VectorEquationHandle,
        scene: &mut Scene,
        start_time: f32,
        duration: f32,
        ease: crate::frontend::animation::Ease,
    ) {
        self.morph_matching_staged(
            source.ids().to_vec(),
            target.ids().to_vec(),
            scene,
            start_time,
            duration,
            ease,
        );
    }

    pub fn morph_vector_formulas(
        &mut self,
        source: &VectorEquationHandle,
        target: &VectorEquationHandle,
        scene: &mut Scene,
        start_time: f32,
        duration: f32,
        ease: crate::frontend::animation::Ease,
    ) {
        self.morph_vector_equations(source, target, scene, start_time, duration, ease);
    }

    fn ordered_tattvas(ids: &[TattvaId], scene: &Scene) -> Vec<TattvaId> {
        use crate::frontend::props::DrawableProps;

        let mut ordered = ids.to_vec();
        ordered.sort_by(|a, b| {
            let a_pos = scene
                .get_tattva_any(*a)
                .map(|t| DrawableProps::read(t.props()).position)
                .unwrap_or_default();
            let b_pos = scene
                .get_tattva_any(*b)
                .map(|t| DrawableProps::read(t.props()).position)
                .unwrap_or_default();

            a_pos
                .x
                .partial_cmp(&b_pos.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b_pos
                        .y
                        .partial_cmp(&a_pos.y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        ordered
    }

    pub fn write_vector_equation(
        &mut self,
        ids: Vec<TattvaId>,
        scene: &Scene,
        start_time: f32,
        duration: f32,
        ease: Ease,
    ) {
        let ordered = Self::ordered_tattvas(&ids, scene);
        if ordered.is_empty() {
            return;
        }

        let window = 1.8_f32;
        let step = duration.max(0.0) / ((ordered.len().saturating_sub(1)) as f32 + window);
        let item_duration = step * window;

        for (idx, id) in ordered.into_iter().enumerate() {
            self.animate(id)
                .at(start_time + idx as f32 * step)
                .for_duration(item_duration)
                .ease(ease)
                .draw()
                .spawn();
        }
    }

    pub fn unwrite_vector_equation(
        &mut self,
        ids: Vec<TattvaId>,
        scene: &Scene,
        start_time: f32,
        duration: f32,
        ease: Ease,
    ) {
        let mut ordered = Self::ordered_tattvas(&ids, scene);
        if ordered.is_empty() {
            return;
        }
        ordered.reverse();

        let window = 1.8_f32;
        let step = duration.max(0.0) / ((ordered.len().saturating_sub(1)) as f32 + window);
        let item_duration = step * window;

        for (idx, id) in ordered.into_iter().enumerate() {
            self.animate(id)
                .at(start_time + idx as f32 * step)
                .for_duration(item_duration)
                .ease(ease)
                .undraw()
                .spawn();
        }
    }

    pub fn end_time(&self) -> f32 {
        let anim_end = self
            .scheduled
            .iter()
            .map(|sa| sa.start_time + sa.duration.max(0.0))
            .fold(0.0, f32::max);
        anim_end.max(self.hold_until)
    }

    /// Ensures the scene runs at least until `timestamp`, even if all animations
    /// finish earlier. Useful for adding a pause at the end of a scene.
    pub fn wait_until(&mut self, timestamp: f32) {
        if !timestamp.is_finite() {
            self.authoring_errors.push(ValidationError::non_finite(
                "Timeline",
                "wait_until",
                timestamp,
            ));
            return;
        }
        self.hold_until = self.hold_until.max(timestamp);
        self.composition_cursor = self.composition_cursor.max(timestamp);
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Clip {
    pub fn new() -> Self {
        Self {
            timeline: Timeline::new(),
        }
    }

    /// Returns the clip's duration in local seconds.
    pub fn duration(&self) -> f32 {
        self.timeline.end_time()
    }

    pub fn is_empty(&self) -> bool {
        self.timeline.scheduled.is_empty() && self.timeline.hold_until <= 0.0
    }
}

impl Default for Clip {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Clip {
    type Target = Timeline;

    fn deref(&self) -> &Self::Target {
        &self.timeline
    }
}

impl DerefMut for Clip {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::camera::Projection;
    use crate::frontend::collection::math::equation::{EquationLayout, EquationPart};
    use crate::frontend::collection::primitives::circle::Circle;
    use crate::frontend::collection::primitives::path::Path;
    use crate::frontend::collection::primitives::square::Square;
    use crate::frontend::collection::text::label::Label;
    use glam::{Vec2, Vec3, Vec4};
    use std::sync::{Arc, Mutex};

    struct NoopAnimation;

    impl Animation for NoopAnimation {
        fn on_start(&mut self, _scene: &mut Scene) {}

        fn apply_at(&mut self, _scene: &mut Scene, _t: f32) {}
    }

    fn clip_with_animation(start_time: f32, duration: f32) -> Clip {
        let mut clip = Clip::new();
        clip.add_animation(start_time, duration, Box::new(NoopAnimation));
        clip
    }

    #[test]
    fn invalid_schedule_values_are_rejected_without_sorting_panics() {
        let mut timeline = Timeline::new();
        timeline.add_animation(f32::NAN, 1.0, Box::new(NoopAnimation));

        assert!(timeline.scheduled.is_empty());
        assert!(matches!(
            timeline.validate(),
            Err(ValidationError::NonFinite {
                field: "start_time",
                ..
            })
        ));

        let mut scene = Scene::new();
        assert!(scene.play(timeline).is_err());
        assert!(scene.timeline.is_none());
    }

    #[test]
    fn direct_schedule_mutation_is_validated_before_playback() {
        let mut timeline = Timeline::new();
        timeline
            .try_add_animation(0.0, 1.0, Box::new(NoopAnimation))
            .unwrap();
        timeline.scheduled[0].duration = f32::INFINITY;

        assert!(matches!(
            timeline.validate(),
            Err(ValidationError::NonFinite {
                field: "duration",
                ..
            })
        ));
    }

    #[test]
    fn invalid_clip_placement_and_wait_are_reported() {
        let mut timeline = Timeline::new();
        timeline.place_at(f32::INFINITY, Clip::new());
        timeline.wait_until(f32::NAN);

        assert_eq!(timeline.authoring_errors().len(), 2);
        assert!(timeline.validate().is_err());
    }

    #[test]
    fn clip_supports_the_standard_animation_builder() {
        let mut clip = Clip::new();
        clip.animate(42)
            .at(1.25)
            .for_duration(0.75)
            .fade_to(0.0)
            .spawn();

        assert_eq!(clip.scheduled.len(), 1);
        assert_eq!(clip.scheduled[0].start_time, 1.25);
        assert_eq!(clip.duration(), 2.0);
    }

    #[test]
    fn delayed_appear_is_hidden_when_the_timeline_is_installed() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(2.0)
            .for_duration(1.0)
            .appear()
            .spawn();

        scene.play(timeline).unwrap();

        assert!(!is_visible(&scene, id));
        assert_eq!(opacity(&scene, id), 0.0);
        scene.seek_to(2.5).unwrap();
        assert!(is_visible(&scene, id));
        assert_eq!(opacity(&scene, id), 0.5);
    }

    #[test]
    fn delayed_draw_targets_are_staged_before_the_first_frame() {
        let mut scene = Scene::new();
        let shape_id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let path_id =
            scene.add_tattva(Path::new().move_to(Vec2::ZERO).line_to(Vec2::X), Vec3::ZERO);
        let mut timeline = Timeline::new();
        for id in [shape_id, path_id] {
            timeline
                .animate(id)
                .at(2.0)
                .for_duration(1.0)
                .draw()
                .spawn();
        }

        scene.play(timeline).unwrap();

        assert!(!is_visible(&scene, shape_id));
        assert!(!is_visible(&scene, path_id));
        assert_eq!(
            scene
                .get_tattva_typed::<Path>(path_id)
                .unwrap()
                .state
                .trim_end,
            0.0
        );
    }

    #[test]
    fn delayed_typewriter_text_is_empty_when_the_timeline_is_installed() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Label::new("delayed", 0.2), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(2.0)
            .for_duration(1.0)
            .typewrite_text()
            .spawn();

        scene.play(timeline).unwrap();

        let label = scene.get_tattva_typed::<Label>(id).unwrap();
        assert_eq!(label.state.char_reveal, 0.0);
        assert!(label.state.typewriter_mode);
    }

    #[test]
    fn appended_clips_convert_local_times_to_global_times() {
        let mut timeline = Timeline::new();
        timeline.append(clip_with_animation(1.0, 2.0));
        timeline.append(clip_with_animation(0.5, 1.0));

        let starts = timeline
            .scheduled
            .iter()
            .map(|animation| animation.start_time)
            .collect::<Vec<_>>();

        assert_eq!(starts, vec![1.0, 3.5]);
        assert_eq!(timeline.end_time(), 4.5);
        assert_eq!(timeline.cursor(), 4.5);
    }

    #[test]
    fn overlay_uses_the_previous_append_origin_and_extends_the_cursor() {
        let mut timeline = Timeline::new();
        timeline.append(clip_with_animation(0.0, 2.0));
        timeline.overlay(clip_with_animation(0.5, 3.0));
        timeline.append(clip_with_animation(0.0, 1.0));

        let starts = timeline
            .scheduled
            .iter()
            .map(|animation| animation.start_time)
            .collect::<Vec<_>>();

        assert_eq!(starts, vec![0.0, 0.5, 3.5]);
        assert_eq!(timeline.end_time(), 4.5);
    }

    #[test]
    fn explicit_placement_does_not_move_the_composition_cursor() {
        let mut timeline = Timeline::new();
        timeline.place_at(10.0, clip_with_animation(1.0, 2.0));
        timeline.append(clip_with_animation(0.0, 1.0));

        let starts = timeline
            .scheduled
            .iter()
            .map(|animation| animation.start_time)
            .collect::<Vec<_>>();

        assert_eq!(starts, vec![0.0, 11.0]);
        assert_eq!(timeline.cursor(), 1.0);
        assert_eq!(timeline.end_time(), 13.0);
    }

    #[test]
    fn clip_wait_until_contributes_to_composed_duration() {
        let mut clip = Clip::new();
        clip.wait_until(5.0);

        let mut timeline = Timeline::new();
        timeline.append(clip);

        assert_eq!(timeline.cursor(), 5.0);
        assert_eq!(timeline.end_time(), 5.0);
    }

    fn position(scene: &Scene, id: TattvaId) -> Vec3 {
        let tattva = scene.get_tattva_any(id).expect("missing test tattva");
        DrawableProps::read(tattva.props()).position
    }

    fn opacity(scene: &Scene, id: TattvaId) -> f32 {
        let tattva = scene.get_tattva_any(id).expect("missing test tattva");
        DrawableProps::read(tattva.props()).opacity
    }

    fn is_visible(scene: &Scene, id: TattvaId) -> bool {
        let tattva = scene.get_tattva_any(id).expect("missing test tattva");
        DrawableProps::read(tattva.props()).visible
    }

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert!(
            actual.abs_diff_eq(expected, 1e-5),
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn appended_clips_handoff_at_the_shared_absolute_boundary() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);

        let mut first = Clip::new();
        first
            .animate(id)
            .at(0.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(4.0, 0.0, 0.0))
            .spawn();
        let mut second = Clip::new();
        second
            .animate(id)
            .at(0.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(8.0, 0.0, 0.0))
            .spawn();

        let mut timeline = Timeline::new();
        timeline.append(first).append(second);
        scene.play(timeline).unwrap();

        scene.seek_to(2.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(4.0, 0.0, 0.0));
        scene.seek_to(3.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(6.0, 0.0, 0.0));
        scene.seek_to(4.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(8.0, 0.0, 0.0));
    }

    #[test]
    fn explicit_clip_placement_evaluates_local_boundaries_in_absolute_time() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut clip = Clip::new();
        clip.animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(6.0, 0.0, 0.0))
            .spawn();

        let mut timeline = Timeline::new();
        timeline.place_at(4.0, clip);
        scene.play(timeline).unwrap();

        for (scene_time, expected_x) in [(4.999, 0.0), (5.0, 0.0), (6.0, 3.0), (7.0, 6.0)] {
            scene.seek_to(scene_time).unwrap();
            assert_vec3_close(position(&scene, id), Vec3::new(expected_x, 0.0, 0.0));
        }
    }

    #[test]
    fn zero_duration_animation_applies_at_its_boundary_and_rewinds() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(2.0)
            .for_duration(0.0)
            .fade_to(0.0)
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(1.999).unwrap();
        assert_eq!(opacity(&scene, id), 1.0);
        scene.seek_to(2.0).unwrap();
        assert_eq!(opacity(&scene, id), 0.0);
        scene.seek_to(1.0).unwrap();
        assert_eq!(opacity(&scene, id), 1.0);
        scene.seek_to(3.0).unwrap();
        assert_eq!(opacity(&scene, id), 0.0);
    }

    #[test]
    fn overlayed_clip_ownership_is_stable_across_repeated_seeks() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);

        let mut primary = Clip::new();
        primary
            .animate(id)
            .at(0.0)
            .for_duration(4.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(8.0, 0.0, 0.0))
            .spawn();
        let mut override_clip = Clip::new();
        override_clip
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(10.0, 0.0, 0.0))
            .spawn();

        let mut timeline = Timeline::new();
        timeline.append(primary).overlay(override_clip);
        scene.play(timeline).unwrap();

        for (scene_time, expected_x) in [(2.0, 6.0), (0.5, 1.0), (2.0, 6.0), (3.5, 10.0)] {
            scene.seek_to(scene_time).unwrap();
            assert_vec3_close(position(&scene, id), Vec3::new(expected_x, 0.0, 0.0));
        }
    }

    #[test]
    fn callback_in_appended_clip_observes_absolute_time_and_runs_once() {
        let mut lead = Clip::new();
        lead.wait_until(2.0);

        let observed = Arc::new(Mutex::new(Vec::new()));
        let callback_observed = observed.clone();
        let mut callback_clip = Clip::new();
        callback_clip.call_at(1.0, move |scene| {
            callback_observed.lock().unwrap().push(scene.scene_time);
        });

        let mut timeline = Timeline::new();
        timeline.append(lead).append(callback_clip);
        let mut scene = Scene::new();
        scene.play(timeline).unwrap();

        scene.update(5.0).unwrap();
        scene.update(1.0).unwrap();

        assert_eq!(*observed.lock().unwrap(), vec![3.0]);
    }

    #[test]
    fn scene_seek_restores_and_replays_drawable_properties() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(6.0, 2.0, 0.0))
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(2.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(3.0, 1.0, 0.0));

        scene.seek_to(0.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::ZERO);

        scene.seek_to(2.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(3.0, 1.0, 0.0));
        scene.seek_to(4.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(6.0, 2.0, 0.0));
    }

    #[test]
    fn direct_seek_at_zero_evaluates_animations_starting_at_zero() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(0.0)
            .fade_to(0.0)
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(0.0).unwrap();

        assert_eq!(
            DrawableProps::read(scene.get_tattva_any(id).unwrap().props()).opacity,
            0.0
        );
    }

    #[test]
    fn repeated_seek_reconstructs_chained_property_animations() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(10.0, 0.0, 0.0))
            .spawn();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(20.0, 0.0, 0.0))
            .spawn();
        scene.play(timeline).unwrap();

        for target_time in [1.5, 0.5, 1.5] {
            scene.seek_to(target_time).unwrap();
            let expected_x = if target_time < 1.0 {
                target_time * 10.0
            } else {
                10.0 + (target_time - 1.0) * 10.0
            };
            assert_vec3_close(position(&scene, id), Vec3::new(expected_x, 0.0, 0.0));
        }
    }

    #[test]
    fn scene_seek_restores_and_replays_camera_state() {
        let mut scene = Scene::new();
        scene.camera.position = Vec3::new(0.0, 0.0, 10.0);
        scene.camera.target = Vec3::ZERO;
        scene.camera.projection = Projection::Perspective {
            fov_y_rad: 1.0,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        };

        let mut timeline = Timeline::new();
        timeline
            .animate_camera()
            .at(0.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .frame_to(Vec3::new(4.0, 2.0, 8.0), Vec3::new(2.0, 0.0, 0.0))
            .spawn();
        timeline
            .animate_camera()
            .at(2.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .fov_to(0.5)
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(3.0).unwrap();
        assert_vec3_close(scene.camera.position, Vec3::new(4.0, 2.0, 8.0));
        assert_vec3_close(scene.camera.target, Vec3::new(2.0, 0.0, 0.0));
        let Projection::Perspective { fov_y_rad, .. } = scene.camera.projection else {
            panic!("expected perspective camera");
        };
        assert!((fov_y_rad - 0.75).abs() < 1e-5);

        scene.seek_to(0.0).unwrap();
        assert_vec3_close(scene.camera.position, Vec3::new(0.0, 0.0, 10.0));
        assert_vec3_close(scene.camera.target, Vec3::ZERO);
        let Projection::Perspective { fov_y_rad, .. } = scene.camera.projection else {
            panic!("expected perspective camera");
        };
        assert!((fov_y_rad - 1.0).abs() < 1e-5);
    }

    #[test]
    fn negative_scene_update_uses_seek_reconstruction() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(4.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(8.0, 0.0, 0.0))
            .spawn();
        scene.play(timeline).unwrap();

        scene.update(3.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(6.0, 0.0, 0.0));
        scene.update(-2.0).unwrap();
        assert_eq!(scene.scene_time, 1.0);
        assert_vec3_close(position(&scene, id), Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn non_reversible_callback_rejects_seek_without_changing_scene() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline.call_at(1.0, move |scene| scene.hide(id));
        scene.play(timeline).unwrap();

        let error = scene.seek_to(2.0).unwrap_err();
        assert_eq!(
            error,
            SeekError::NonReversibleTimelineEffect {
                effect: "call_at",
                start_time: 1.0,
            }
        );
        assert_eq!(scene.scene_time, 0.0);
        assert_eq!(
            DrawableProps::read(scene.get_tattva_any(id).unwrap().props()).opacity,
            1.0
        );
    }

    #[test]
    fn reversible_callback_resets_before_seek_replay() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline.call_at_reversible(
            1.0,
            move |scene| scene.hide(id),
            move |scene| scene.show(id),
        );
        scene.play(timeline).unwrap();

        scene.seek_to(2.0).unwrap();
        assert_eq!(
            DrawableProps::read(scene.get_tattva_any(id).unwrap().props()).opacity,
            0.0
        );
        scene.seek_to(0.0).unwrap();
        assert_eq!(
            DrawableProps::read(scene.get_tattva_any(id).unwrap().props()).opacity,
            1.0
        );
        scene.seek_to(2.0).unwrap();
        assert_eq!(
            DrawableProps::read(scene.get_tattva_any(id).unwrap().props()).opacity,
            0.0
        );
    }

    #[test]
    fn seek_restores_tattva_replaced_by_an_in_progress_morph() {
        let mut scene = Scene::new();
        let source_id = scene.add_tattva(Circle::new(1.0, 32, Vec4::ONE), Vec3::ZERO);
        let target_id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::new(2.0, 0.0, 0.0));
        let mut timeline = Timeline::new();
        timeline
            .animate(target_id)
            .at(1.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .morph_from(source_id)
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(1.5).unwrap();
        assert!(scene.get_tattva_typed::<Path>(target_id).is_some());
        scene.seek_to(0.0).unwrap();
        assert!(scene.get_tattva_typed::<Square>(target_id).is_some());
        scene.seek_to(1.5).unwrap();
        assert!(scene.get_tattva_typed::<Path>(target_id).is_some());
    }

    fn scene_with_overlapping_moves() -> (Scene, TattvaId) {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(4.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(8.0, 0.0, 0.0))
            .spawn();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .move_to(Vec3::new(10.0, 0.0, 0.0))
            .spawn();
        scene.play(timeline).unwrap();
        (scene, id)
    }

    #[test]
    fn later_starting_property_animation_wins_during_and_after_overlap() {
        let (mut direct_scene, direct_id) = scene_with_overlapping_moves();
        direct_scene.seek_to(2.0).unwrap();
        assert_vec3_close(position(&direct_scene, direct_id), Vec3::new(6.0, 0.0, 0.0));
        direct_scene.seek_to(3.5).unwrap();
        assert_vec3_close(
            position(&direct_scene, direct_id),
            Vec3::new(10.0, 0.0, 0.0),
        );

        let (mut stepped_scene, stepped_id) = scene_with_overlapping_moves();
        for _ in 0..14 {
            stepped_scene.update(0.25).unwrap();
        }
        assert_vec3_close(
            position(&stepped_scene, stepped_id),
            Vec3::new(10.0, 0.0, 0.0),
        );
    }

    fn scene_with_equation_continuity() -> (Scene, TattvaId, TattvaId) {
        let mut scene = Scene::new();
        let source_id = scene.add_tattva(
            EquationLayout::new(
                vec![
                    EquationPart::new("x").with_key("x"),
                    EquationPart::new("+").with_key("plus"),
                    EquationPart::new("2").with_key("two"),
                ],
                0.4,
            ),
            Vec3::ZERO,
        );
        let target_id = scene.add_tattva(
            EquationLayout::new(
                vec![
                    EquationPart::new("x").with_key("x"),
                    EquationPart::new("=").with_key("equals"),
                    EquationPart::new("2").with_key("two"),
                ],
                0.4,
            ),
            Vec3::ZERO,
        );
        scene.hide(target_id);

        let mut timeline = Timeline::new();
        timeline
            .animate(source_id)
            .at(0.0)
            .for_duration(0.5)
            .appear()
            .spawn();
        timeline
            .animate(target_id)
            .at(1.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .equation_continuity_from(source_id)
            .spawn();
        scene.play(timeline).unwrap();

        (scene, source_id, target_id)
    }

    #[test]
    fn equation_continuity_owns_visibility_after_an_earlier_appear() {
        let (mut scene, source_id, target_id) = scene_with_equation_continuity();

        for _ in 0..12 {
            scene.update(0.25).unwrap();
        }

        assert!(!is_visible(&scene, source_id));
        assert_eq!(opacity(&scene, source_id), 0.0);
        assert!(is_visible(&scene, target_id));
        assert_eq!(opacity(&scene, target_id), 1.0);
    }

    #[test]
    fn equation_continuity_terminal_visibility_is_stable_across_seeks() {
        let (mut scene, source_id, target_id) = scene_with_equation_continuity();

        for time in [3.0, 0.5, 3.0] {
            scene.seek_to(time).unwrap();
            if time > 2.0 {
                assert!(!is_visible(&scene, source_id));
                assert!(is_visible(&scene, target_id));
                assert_eq!(opacity(&scene, target_id), 1.0);
            }
        }
    }

    #[test]
    fn insertion_order_breaks_ties_for_same_start_property_animations() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        for target_x in [10.0, 20.0] {
            timeline
                .animate(id)
                .at(0.0)
                .for_duration(2.0)
                .ease(Ease::Linear)
                .move_to(Vec3::new(target_x, 0.0, 0.0))
                .spawn();
        }
        scene.play(timeline).unwrap();

        scene.seek_to(1.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(10.0, 0.0, 0.0));
        scene.seek_to(3.0).unwrap();
        assert_vec3_close(position(&scene, id), Vec3::new(20.0, 0.0, 0.0));
    }

    #[test]
    fn call_during_invokes_each_normalized_endpoint_once() {
        let mut scene = Scene::new();
        let samples = Arc::new(Mutex::new(Vec::new()));
        let captured_samples = samples.clone();
        let mut timeline = Timeline::new();
        timeline.call_during(0.0, 1.0, move |_scene, t| {
            captured_samples.lock().unwrap().push(t);
        });
        scene.play(timeline).unwrap();

        scene.update(0.0).unwrap();
        scene.update(1.0).unwrap();

        assert_eq!(*samples.lock().unwrap(), vec![0.0, 1.0]);
    }

    #[test]
    fn callback_observes_its_exact_crossed_start_time() {
        let mut scene = Scene::new();
        let observed_time = Arc::new(Mutex::new(None));
        let callback_time = observed_time.clone();
        let mut timeline = Timeline::new();
        timeline.call_at(1.0, move |scene| {
            *callback_time.lock().unwrap() = Some(scene.scene_time);
        });
        scene.play(timeline).unwrap();

        scene.update(2.0).unwrap();

        assert_eq!(*observed_time.lock().unwrap(), Some(1.0));
        assert_eq!(scene.scene_time, 2.0);
    }
}
