use crate::engine::timeline::Timeline;
use crate::frontend::TattvaId;
use crate::frontend::animation::{
    Animation, BeltEvolve, BeltPhaseBy, BeltPhaseTo, Create, Ease, EquationContinuity, FadeTo,
    FollowAnchor, HorizonEvolve, HorizonPhaseBy, HorizonPhaseTo, IndicateText, KvCacheFill,
    LetterParticleScatterTo, MatchTransform, MatrixStep, MorphGeometry, MorphObjects, MoveTo,
    NoiseEvolve, NoisePhaseBy, NoisePhaseTo, PerlinFieldEvolve, PerlinFieldPhaseBy,
    PerlinFieldPhaseTo, PropagateSignal, RevealText, RevealTo, RotateTo, ScaleTo, StepwiseSignal,
    TensorSelectionTransition, TensorTransition, TransformerStageFocus, UnrevealText, UnwritePath,
    UnwriteSurface, UnwriteTable, UnwriteText, WritePath, WriteSurface, WriteTable, WriteText,
};
use crate::frontend::layout::Anchor;
use crate::frontend::sangrah::common::tensor::{TensorSelector, TensorSnapshot};
use glam::{Quat, Vec3, Vec4};

#[derive(Debug, Clone)]
pub enum AnimKind {
    MoveTo {
        to: Vec3,
        from: Option<Vec3>,
    },
    RotateTo {
        to: Quat,
        from: Option<Quat>,
    },
    ScaleTo {
        to: Vec3,
        from: Option<Vec3>,
    },
    FadeTo {
        opacity: f32,
        from: Option<f32>,
    },
    Create,
    MatchTransform {
        source_id: TattvaId,
    },
    MorphFrom {
        source_id: TattvaId,
    },
    EquationContinuityFrom {
        source_id: TattvaId,
    },
    MatrixStepCells {
        cells: Vec<(usize, usize)>,
        highlight: Vec4,
        dim_opacity: f32,
    },
    MatrixStepRow {
        row: usize,
        highlight: Vec4,
        dim_opacity: f32,
    },
    MatrixStepColumn {
        col: usize,
        highlight: Vec4,
        dim_opacity: f32,
    },
    TensorTo {
        snapshot: TensorSnapshot,
    },
    TensorSelect {
        selections: Vec<TensorSelector>,
    },
    TransformerFocus {
        stage_id: Option<String>,
    },
    KvCacheFill {
        occupied_tokens: usize,
    },
    FollowAnchor {
        target_id: TattvaId,
        target_anchor: Anchor,
        follower_anchor: Anchor,
        offset: Vec3,
    },
    NoisePhaseTo {
        to: f32,
    },
    NoisePhaseBy {
        delta: f32,
    },
    NoiseEvolve {
        speed: Option<f32>,
    },
    BeltPhaseTo {
        to: f32,
    },
    BeltPhaseBy {
        delta: f32,
    },
    BeltEvolve {
        speed: Option<f32>,
    },
    LetterParticleScatterTo {
        to: f32,
    },
    HorizonPhaseTo {
        to: f32,
    },
    HorizonPhaseBy {
        delta: f32,
    },
    HorizonEvolve {
        speed: Option<f32>,
    },
    PerlinFieldPhaseTo {
        to: f32,
    },
    PerlinFieldPhaseBy {
        delta: f32,
    },
    PerlinFieldEvolve {
        speed: Option<f32>,
    },
    Propagate {
        to: f32,
    },
    Reveal {
        to: f32,
    },
    Signal {
        to: f32,
    },
    WritePath,
    UnwritePath,
    WriteText,
    UnwriteText,
    RevealText,
    UnrevealText,
    IndicateText,
    WriteTable,
    UnwriteTable,
    WriteSurface,
    UnwriteSurface,
}

#[derive(Debug, Clone)]
pub struct AnimationSpec {
    pub target_id: TattvaId,
    pub start_time: f32,
    pub duration: f32,
    pub ease: Option<Ease>,
    pub kind: Option<AnimKind>,
}

pub struct AnimationBuilder<'a> {
    timeline: &'a mut Timeline,
    target_id: TattvaId,
    spec: AnimationSpec,
}

impl<'a> AnimationBuilder<'a> {
    pub fn new(timeline: &'a mut Timeline, target_id: TattvaId) -> Self {
        Self {
            timeline,
            target_id,
            spec: AnimationSpec {
                target_id,
                start_time: 0.0,
                duration: 1.0,
                ease: None,
                kind: None,
            },
        }
    }

    pub fn at(mut self, start_time: f32) -> Self {
        self.spec.start_time = start_time;
        self
    }

    pub fn for_duration(mut self, duration: f32) -> Self {
        self.spec.duration = duration;
        self
    }

    pub fn ease(mut self, ease: Ease) -> Self {
        self.spec.ease = Some(ease);
        self
    }

    pub fn move_to(mut self, to: Vec3) -> Self {
        self.spec.kind = Some(AnimKind::MoveTo { to, from: None });
        self
    }

    pub fn rotate_to(mut self, to: Quat) -> Self {
        self.spec.kind = Some(AnimKind::RotateTo { to, from: None });
        self
    }

    pub fn scale_to(mut self, to: Vec3) -> Self {
        self.spec.kind = Some(AnimKind::ScaleTo { to, from: None });
        self
    }

    pub fn fade_to(mut self, opacity: f32) -> Self {
        self.spec.kind = Some(AnimKind::FadeTo {
            opacity,
            from: None,
        });
        self
    }

    pub fn from(mut self, value: impl Into<f32>) -> Self {
        let val = value.into();
        match self.spec.kind.take() {
            Some(AnimKind::FadeTo { opacity, .. }) => {
                self.spec.kind = Some(AnimKind::FadeTo {
                    opacity,
                    from: Some(val),
                });
            }
            Some(k) => self.spec.kind = Some(k),
            None => {}
        }
        self
    }

    pub fn from_vec3(mut self, value: Vec3) -> Self {
        match self.spec.kind.take() {
            Some(AnimKind::MoveTo { to, .. }) => {
                self.spec.kind = Some(AnimKind::MoveTo {
                    to,
                    from: Some(value),
                });
            }
            Some(AnimKind::ScaleTo { to, .. }) => {
                self.spec.kind = Some(AnimKind::ScaleTo {
                    to,
                    from: Some(value),
                });
            }
            Some(k) => self.spec.kind = Some(k),
            None => {}
        }
        self
    }

    /// Animate a tattva from hidden to visible by fading its staged opacity in.
    pub fn appear(mut self) -> Self {
        self.spec.kind = Some(AnimKind::Create);
        self
    }

    pub fn match_transform(mut self, source_id: TattvaId) -> Self {
        self.spec.kind = Some(AnimKind::MatchTransform { source_id });
        self
    }

    pub fn morph_from(mut self, source_id: TattvaId) -> Self {
        self.spec.kind = Some(AnimKind::MorphFrom { source_id });
        self
    }

    pub fn equation_continuity_from(mut self, source_id: TattvaId) -> Self {
        self.spec.kind = Some(AnimKind::EquationContinuityFrom { source_id });
        self
    }

    pub fn matrix_step_cells(
        mut self,
        cells: Vec<(usize, usize)>,
        highlight: Vec4,
        dim_opacity: f32,
    ) -> Self {
        self.spec.kind = Some(AnimKind::MatrixStepCells {
            cells,
            highlight,
            dim_opacity,
        });
        self
    }

    pub fn matrix_step_row(mut self, row: usize, highlight: Vec4, dim_opacity: f32) -> Self {
        self.spec.kind = Some(AnimKind::MatrixStepRow {
            row,
            highlight,
            dim_opacity,
        });
        self
    }

    pub fn matrix_step_column(mut self, col: usize, highlight: Vec4, dim_opacity: f32) -> Self {
        self.spec.kind = Some(AnimKind::MatrixStepColumn {
            col,
            highlight,
            dim_opacity,
        });
        self
    }

    /// Interpolate a semantic tensor view to another compatible snapshot.
    pub fn tensor_to(mut self, snapshot: TensorSnapshot) -> Self {
        self.spec.kind = Some(AnimKind::TensorTo { snapshot });
        self
    }

    /// Crossfade a tensor view to a new semantic selection.
    pub fn tensor_select(mut self, selections: Vec<TensorSelector>) -> Self {
        self.spec.kind = Some(AnimKind::TensorSelect { selections });
        self
    }

    /// Crossfades focus to one semantic stage in a transformer composition.
    pub fn transformer_focus(mut self, stage_id: impl Into<String>) -> Self {
        self.spec.kind = Some(AnimKind::TransformerFocus {
            stage_id: Some(stage_id.into()),
        });
        self
    }

    /// Restores equal emphasis across all transformer stages.
    pub fn transformer_clear_focus(mut self) -> Self {
        self.spec.kind = Some(AnimKind::TransformerFocus { stage_id: None });
        self
    }

    /// Reveals key/value cache slots through the requested occupied-token count.
    pub fn kv_cache_fill_to(mut self, occupied_tokens: usize) -> Self {
        self.spec.kind = Some(AnimKind::KvCacheFill { occupied_tokens });
        self
    }

    pub fn follow_anchor(
        mut self,
        target_id: TattvaId,
        target_anchor: Anchor,
        follower_anchor: Anchor,
        offset: Vec3,
    ) -> Self {
        self.spec.kind = Some(AnimKind::FollowAnchor {
            target_id,
            target_anchor,
            follower_anchor,
            offset,
        });
        self
    }

    pub fn propagate(mut self) -> Self {
        self.spec.kind = Some(AnimKind::Propagate { to: 1.0 });
        self
    }

    pub fn noise_phase_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::NoisePhaseTo { to });
        self
    }

    pub fn noise_phase_by(mut self, delta: f32) -> Self {
        self.spec.kind = Some(AnimKind::NoisePhaseBy { delta });
        self
    }

    pub fn noise_evolve(mut self) -> Self {
        self.spec.kind = Some(AnimKind::NoiseEvolve { speed: None });
        self
    }

    pub fn noise_evolve_with_speed(mut self, speed: f32) -> Self {
        self.spec.kind = Some(AnimKind::NoiseEvolve { speed: Some(speed) });
        self
    }

    pub fn belt_phase_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::BeltPhaseTo { to });
        self
    }

    pub fn belt_phase_by(mut self, delta: f32) -> Self {
        self.spec.kind = Some(AnimKind::BeltPhaseBy { delta });
        self
    }

    pub fn belt_evolve(mut self) -> Self {
        self.spec.kind = Some(AnimKind::BeltEvolve { speed: None });
        self
    }

    pub fn belt_evolve_with_speed(mut self, speed: f32) -> Self {
        self.spec.kind = Some(AnimKind::BeltEvolve { speed: Some(speed) });
        self
    }

    /// Scatter a glyph-derived `LetterParticles3D` cloud to its full extent.
    pub fn scatter_letter_particles(mut self) -> Self {
        self.spec.kind = Some(AnimKind::LetterParticleScatterTo { to: 1.0 });
        self
    }

    /// Interpolate a `LetterParticles3D` cloud to a specific scatter amount.
    pub fn letter_particle_scatter_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::LetterParticleScatterTo {
            to: to.clamp(0.0, 1.0),
        });
        self
    }

    pub fn horizon_phase_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::HorizonPhaseTo { to });
        self
    }

    pub fn horizon_phase_by(mut self, delta: f32) -> Self {
        self.spec.kind = Some(AnimKind::HorizonPhaseBy { delta });
        self
    }

    pub fn horizon_evolve(mut self) -> Self {
        self.spec.kind = Some(AnimKind::HorizonEvolve { speed: None });
        self
    }

    pub fn horizon_evolve_with_speed(mut self, speed: f32) -> Self {
        self.spec.kind = Some(AnimKind::HorizonEvolve { speed: Some(speed) });
        self
    }

    pub fn perlin_field_phase_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::PerlinFieldPhaseTo { to });
        self
    }

    pub fn perlin_field_phase_by(mut self, delta: f32) -> Self {
        self.spec.kind = Some(AnimKind::PerlinFieldPhaseBy { delta });
        self
    }

    pub fn perlin_field_evolve(mut self) -> Self {
        self.spec.kind = Some(AnimKind::PerlinFieldEvolve { speed: None });
        self
    }

    pub fn perlin_field_evolve_with_speed(mut self, speed: f32) -> Self {
        self.spec.kind = Some(AnimKind::PerlinFieldEvolve { speed: Some(speed) });
        self
    }

    pub fn propagate_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::Propagate {
            to: to.clamp(0.0, 1.0),
        });
        self
    }

    pub fn signal_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::Signal {
            to: to.clamp(0.0, 1.0),
        });
        self
    }

    pub fn reveal_to(mut self, to: f32) -> Self {
        self.spec.kind = Some(AnimKind::Reveal {
            to: to.clamp(0.0, 1.0),
        });
        self
    }

    /// Draw a path-like tattva on progressively.
    pub fn draw(mut self) -> Self {
        self.spec.kind = Some(AnimKind::WritePath);
        self
    }

    /// Reverse a draw-on animation for path-like tattvas.
    pub fn undraw(mut self) -> Self {
        self.spec.kind = Some(AnimKind::UnwritePath);
        self
    }

    /// Reveal text character by character while keeping its anchor stable.
    pub fn typewrite_text(mut self) -> Self {
        self.spec.kind = Some(AnimKind::WriteText);
        self
    }

    /// Reverse a fixed-anchor typewriter animation.
    pub fn untypewrite_text(mut self) -> Self {
        self.spec.kind = Some(AnimKind::UnwriteText);
        self
    }

    /// Reveal text with the centered shifting/growing effect.
    pub fn reveal_text(mut self) -> Self {
        self.spec.kind = Some(AnimKind::RevealText);
        self
    }

    /// Hide text using the reverse of the centered reveal effect.
    pub fn hide_text(mut self) -> Self {
        self.spec.kind = Some(AnimKind::UnrevealText);
        self
    }

    pub fn indicate(mut self) -> Self {
        self.spec.kind = Some(AnimKind::IndicateText);
        self
    }

    pub fn write_table(mut self) -> Self {
        self.spec.kind = Some(AnimKind::WriteTable);
        self
    }

    pub fn unwrite_table(mut self) -> Self {
        self.spec.kind = Some(AnimKind::UnwriteTable);
        self
    }

    pub fn write_surface(mut self) -> Self {
        self.spec.kind = Some(AnimKind::WriteSurface);
        self
    }

    pub fn unwrite_surface(mut self) -> Self {
        self.spec.kind = Some(AnimKind::UnwriteSurface);
        self
    }

    pub fn spawn(self) {
        let spec = self.spec;
        let ease = spec.ease.unwrap_or_default();

        let anim: Box<dyn Animation> = match spec.kind {
            Some(AnimKind::MoveTo { to, from }) => {
                let mut m = MoveTo::new(spec.target_id, to, ease);
                if let Some(f) = from {
                    m = m.with_from(f);
                }
                Box::new(m)
            }
            Some(AnimKind::RotateTo { to, from }) => {
                let mut m = RotateTo::new(spec.target_id, to, ease);
                if let Some(f) = from {
                    m = m.with_from(f);
                }
                Box::new(m)
            }
            Some(AnimKind::ScaleTo { to, from }) => {
                let mut m = ScaleTo::new(spec.target_id, to, ease);
                if let Some(f) = from {
                    m = m.with_from(f);
                }
                Box::new(m)
            }
            Some(AnimKind::FadeTo { opacity, from }) => {
                let mut m = FadeTo::new(spec.target_id, opacity, ease);
                if let Some(f) = from {
                    m = m.with_from(f);
                }
                Box::new(m)
            }
            Some(AnimKind::Create) => Box::new(Create::new(spec.target_id, ease)),
            Some(AnimKind::MatchTransform { source_id }) => {
                Box::new(MatchTransform::new(spec.target_id, source_id, ease))
            }
            Some(AnimKind::MorphFrom { source_id }) => {
                Box::new(MorphGeometry::new(source_id, spec.target_id, ease))
            }
            Some(AnimKind::EquationContinuityFrom { source_id }) => {
                Box::new(EquationContinuity::new(source_id, spec.target_id, ease))
            }
            Some(AnimKind::MatrixStepCells {
                cells,
                highlight,
                dim_opacity,
            }) => Box::new(MatrixStep::cells(
                spec.target_id,
                cells,
                highlight,
                dim_opacity,
                ease,
            )),
            Some(AnimKind::MatrixStepRow {
                row,
                highlight,
                dim_opacity,
            }) => Box::new(MatrixStep::row(
                spec.target_id,
                row,
                highlight,
                dim_opacity,
                ease,
            )),
            Some(AnimKind::MatrixStepColumn {
                col,
                highlight,
                dim_opacity,
            }) => Box::new(MatrixStep::column(
                spec.target_id,
                col,
                highlight,
                dim_opacity,
                ease,
            )),
            Some(AnimKind::TensorTo { snapshot }) => {
                Box::new(TensorTransition::new(spec.target_id, snapshot, ease))
            }
            Some(AnimKind::TensorSelect { selections }) => Box::new(
                TensorSelectionTransition::new(spec.target_id, selections, ease),
            ),
            Some(AnimKind::TransformerFocus { stage_id }) => {
                Box::new(TransformerStageFocus::new(spec.target_id, stage_id, ease))
            }
            Some(AnimKind::KvCacheFill { occupied_tokens }) => {
                Box::new(KvCacheFill::new(spec.target_id, occupied_tokens, ease))
            }
            Some(AnimKind::FollowAnchor {
                target_id,
                target_anchor,
                follower_anchor,
                offset,
            }) => Box::new(FollowAnchor::new(
                spec.target_id,
                target_id,
                target_anchor,
                follower_anchor,
                offset,
            )),
            Some(AnimKind::NoisePhaseTo { to }) => {
                Box::new(NoisePhaseTo::new(spec.target_id, to, ease))
            }
            Some(AnimKind::NoisePhaseBy { delta }) => {
                Box::new(NoisePhaseBy::new(spec.target_id, delta, ease))
            }
            Some(AnimKind::NoiseEvolve { speed }) => {
                Box::new(NoiseEvolve::new(spec.target_id, spec.duration, speed, ease))
            }
            Some(AnimKind::BeltPhaseTo { to }) => {
                Box::new(BeltPhaseTo::new(spec.target_id, to, ease))
            }
            Some(AnimKind::BeltPhaseBy { delta }) => {
                Box::new(BeltPhaseBy::new(spec.target_id, delta, ease))
            }
            Some(AnimKind::BeltEvolve { speed }) => {
                Box::new(BeltEvolve::new(spec.target_id, spec.duration, speed, ease))
            }
            Some(AnimKind::LetterParticleScatterTo { to }) => Box::new(
                LetterParticleScatterTo::new(spec.target_id, to, spec.duration, ease),
            ),
            Some(AnimKind::HorizonPhaseTo { to }) => {
                Box::new(HorizonPhaseTo::new(spec.target_id, to, ease))
            }
            Some(AnimKind::HorizonPhaseBy { delta }) => {
                Box::new(HorizonPhaseBy::new(spec.target_id, delta, ease))
            }
            Some(AnimKind::HorizonEvolve { speed }) => Box::new(HorizonEvolve::new(
                spec.target_id,
                spec.duration,
                speed,
                ease,
            )),
            Some(AnimKind::PerlinFieldPhaseTo { to }) => {
                Box::new(PerlinFieldPhaseTo::new(spec.target_id, to, ease))
            }
            Some(AnimKind::PerlinFieldPhaseBy { delta }) => {
                Box::new(PerlinFieldPhaseBy::new(spec.target_id, delta, ease))
            }
            Some(AnimKind::PerlinFieldEvolve { speed }) => Box::new(PerlinFieldEvolve::new(
                spec.target_id,
                spec.duration,
                speed,
                ease,
            )),
            Some(AnimKind::Propagate { to }) => {
                Box::new(PropagateSignal::new(spec.target_id, to, ease))
            }
            Some(AnimKind::Signal { to }) => {
                Box::new(StepwiseSignal::new(spec.target_id, to, ease))
            }
            Some(AnimKind::Reveal { to }) => Box::new(RevealTo::new(spec.target_id, to, ease)),
            Some(AnimKind::WritePath) => Box::new(WritePath::new(spec.target_id, ease)),
            Some(AnimKind::UnwritePath) => Box::new(UnwritePath::new(spec.target_id, ease)),
            Some(AnimKind::WriteText) => Box::new(WriteText::new(spec.target_id, ease)),
            Some(AnimKind::UnwriteText) => Box::new(UnwriteText::new(spec.target_id, ease)),
            Some(AnimKind::RevealText) => Box::new(RevealText::new(spec.target_id, ease)),
            Some(AnimKind::UnrevealText) => Box::new(UnrevealText::new(spec.target_id, ease)),
            Some(AnimKind::IndicateText) => Box::new(IndicateText::new(spec.target_id, ease)),
            Some(AnimKind::WriteTable) => Box::new(WriteTable::new(spec.target_id, ease)),
            Some(AnimKind::UnwriteTable) => Box::new(UnwriteTable::new(spec.target_id, ease)),
            Some(AnimKind::WriteSurface) => Box::new(WriteSurface::new(spec.target_id, ease)),
            Some(AnimKind::UnwriteSurface) => Box::new(UnwriteSurface::new(spec.target_id, ease)),
            None => panic!("Murali Error: AnimationBuilder requires a kind before .spawn()"),
        };

        self.timeline
            .add_animation(spec.start_time, spec.duration, anim);
    }
}
