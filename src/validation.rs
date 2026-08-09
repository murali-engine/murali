/// Structured validation failures produced while authoring or projecting a scene.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ValidationError {
    #[error("{component}.{field} must be finite, got {value}")]
    NonFinite {
        component: &'static str,
        field: &'static str,
        value: f32,
    },
    #[error("{component}.{field} must be at least {minimum}, got {actual}")]
    CountTooSmall {
        component: &'static str,
        field: &'static str,
        minimum: usize,
        actual: usize,
    },
    #[error("{component}.{field} must not be empty")]
    Empty {
        component: &'static str,
        field: &'static str,
    },
    #[error("{component}.{field} produced a non-finite vector ({x}, {y})")]
    NonFiniteVector2 {
        component: &'static str,
        field: &'static str,
        x: f32,
        y: f32,
    },
    #[error("{component}.{field} produced a non-finite vector ({x}, {y}, {z})")]
    NonFiniteVector3 {
        component: &'static str,
        field: &'static str,
        x: f32,
        y: f32,
        z: f32,
    },
    #[error("{component}.{field} must be greater than zero, got {value}")]
    NonPositive {
        component: &'static str,
        field: &'static str,
        value: f32,
    },
    #[error(
        "{component}.{field} has invalid bounds: min=({min_x}, {min_y}), max=({max_x}, {max_y})"
    )]
    InvalidBounds {
        component: &'static str,
        field: &'static str,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    },
    #[error("{component}.{field} must be an ordered finite range, got ({start}, {end})")]
    InvalidRange {
        component: &'static str,
        field: &'static str,
        start: f32,
        end: f32,
    },
    #[error("{component}.{field} must be between {minimum} and {maximum}, got {value}")]
    OutOfRange {
        component: &'static str,
        field: &'static str,
        minimum: f32,
        maximum: f32,
        value: f32,
    },
    #[error("{component}.{field} must have rank {expected}, got {actual}")]
    RankMismatch {
        component: &'static str,
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("{component}.{field} must contain {expected} items, got {actual}")]
    LengthMismatch {
        component: &'static str,
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("{component}.{field} contains duplicate identifier '{value}'")]
    DuplicateIdentifier {
        component: &'static str,
        field: &'static str,
        value: String,
    },
    #[error("{component}.{field} references unknown identifier '{value}'")]
    UnknownIdentifier {
        component: &'static str,
        field: &'static str,
        value: String,
    },
    #[error("{component}.{field} index {index} is outside 0..{length}")]
    IndexOutOfBounds {
        component: &'static str,
        field: &'static str,
        index: usize,
        length: usize,
    },
    #[error("{component}.{field} dimensions overflow addressable storage")]
    ShapeOverflow {
        component: &'static str,
        field: &'static str,
    },
    #[error("{component}.{field} is incompatible: {reason}")]
    Incompatible {
        component: &'static str,
        field: &'static str,
        reason: String,
    },
    #[error("{component} target {target_id} does not exist")]
    MissingTarget {
        component: &'static str,
        target_id: usize,
    },
    #[error("{component} target {target_id} must be {expected}")]
    TargetTypeMismatch {
        component: &'static str,
        target_id: usize,
        expected: &'static str,
    },
}

impl ValidationError {
    pub(crate) fn non_finite(component: &'static str, field: &'static str, value: f32) -> Self {
        Self::NonFinite {
            component,
            field,
            value,
        }
    }

    pub(crate) fn count_too_small(
        component: &'static str,
        field: &'static str,
        minimum: usize,
        actual: usize,
    ) -> Self {
        Self::CountTooSmall {
            component,
            field,
            minimum,
            actual,
        }
    }
}
