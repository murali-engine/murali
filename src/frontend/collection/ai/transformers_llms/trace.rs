use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::frontend::collection::common::tensor::{TensorAxis, TensorSnapshot};
use crate::validation::ValidationError;

pub const AI_TRACE_SCHEMA_VERSION: u32 = 1;
const COMPONENT: &str = "AiTrace";

/// Framework-neutral metadata describing the source of a recorded AI trace.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiModelMetadata {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub head: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceTokenRole {
    #[default]
    Text,
    Special,
    Generated,
}

/// One tokenizer output with stable identity and optional source-text coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceToken {
    pub id: String,
    pub text: String,
    pub index: usize,
    #[serde(default)]
    pub byte_span: Option<[usize; 2]>,
    #[serde(default)]
    pub word_id: Option<String>,
    #[serde(default)]
    pub role: TraceTokenRole,
}

/// A timestamped, typed event referring to tokens and tensors stored in the trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiTraceEvent {
    pub id: String,
    pub timestamp: f32,
    #[serde(default)]
    pub cue: Option<String>,
    #[serde(flatten)]
    pub kind: AiTraceEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum AiTraceEventKind {
    Tokens {
        token_ids: Vec<String>,
    },
    Tensor {
        tensor_id: String,
    },
    Operation {
        operation: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
    },
    Generation {
        token_id: String,
        probability: f32,
    },
    Metric {
        name: String,
        value: f32,
    },
    Cue,
}

/// Versioned model snapshots and events consumed by Murali's AI teaching views.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiTrace {
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub metadata: AiModelMetadata,
    #[serde(default)]
    pub tokens: Vec<TraceToken>,
    #[serde(default)]
    pub tensors: Vec<TensorSnapshot>,
    #[serde(default)]
    pub events: Vec<AiTraceEvent>,
}

#[derive(Debug, thiserror::Error)]
pub enum AiTraceError {
    #[error("failed to read AI trace '{}': {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid AI trace JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

impl AiTrace {
    pub fn from_json_str(json: &str) -> Result<Self, AiTraceError> {
        let trace: Self = serde_json::from_str(json)?;
        trace.validate()?;
        Ok(trace)
    }

    pub fn from_json_path(path: impl AsRef<Path>) -> Result<Self, AiTraceError> {
        let path = path.as_ref();
        let json = fs::read_to_string(path).map_err(|source| AiTraceError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_str(&json)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != AI_TRACE_SCHEMA_VERSION {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "schema version",
                reason: format!(
                    "expected version {AI_TRACE_SCHEMA_VERSION}, got {}",
                    self.schema_version
                ),
            });
        }
        validate_nonempty("id", &self.id)?;

        let mut token_ids = HashSet::with_capacity(self.tokens.len());
        let mut token_indices = HashSet::with_capacity(self.tokens.len());
        for token in &self.tokens {
            validate_nonempty("token id", &token.id)?;
            validate_nonempty("token text", &token.text)?;
            if !token_ids.insert(token.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "tokens",
                    value: token.id.clone(),
                });
            }
            if !token_indices.insert(token.index) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "token indices",
                    value: token.index.to_string(),
                });
            }
            if let Some([start, end]) = token.byte_span {
                if start > end {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "token byte span",
                        reason: format!("token '{}' has start {start} after end {end}", token.id),
                    });
                }
            }
        }

        let mut tensor_ids = HashSet::with_capacity(self.tensors.len());
        for tensor in &self.tensors {
            tensor.validate()?;
            if !tensor_ids.insert(tensor.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "tensors",
                    value: tensor.id.clone(),
                });
            }
        }

        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut previous_timestamp = 0.0;
        for (index, event) in self.events.iter().enumerate() {
            validate_nonempty("event id", &event.id)?;
            if !event_ids.insert(event.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "events",
                    value: event.id.clone(),
                });
            }
            if !event.timestamp.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field: "event timestamp",
                    value: event.timestamp,
                });
            }
            if event.timestamp < 0.0 {
                return Err(ValidationError::OutOfRange {
                    component: COMPONENT,
                    field: "event timestamp",
                    minimum: 0.0,
                    maximum: f32::MAX,
                    value: event.timestamp,
                });
            }
            if index > 0 && event.timestamp < previous_timestamp {
                return Err(ValidationError::Incompatible {
                    component: COMPONENT,
                    field: "event order",
                    reason: format!(
                        "event '{}' at {} precedes the previous timestamp {}",
                        event.id, event.timestamp, previous_timestamp
                    ),
                });
            }
            previous_timestamp = event.timestamp;
            if let Some(cue) = &event.cue {
                validate_nonempty("event cue", cue)?;
            }
            event.kind.validate(&token_ids, &tensor_ids)?;
        }
        Ok(())
    }

    pub fn tensor(&self, tensor_id: &str) -> Option<&TensorSnapshot> {
        self.tensors.iter().find(|tensor| tensor.id == tensor_id)
    }

    pub fn require_tensor(&self, tensor_id: &str) -> Result<&TensorSnapshot, ValidationError> {
        self.tensor(tensor_id)
            .ok_or_else(|| ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "tensor",
                value: tensor_id.to_string(),
            })
    }

    pub fn token_axis(&self, axis_id: impl Into<String>, label: impl Into<String>) -> TensorAxis {
        let mut tokens: Vec<_> = self.tokens.iter().collect();
        tokens.sort_by_key(|token| token.index);
        TensorAxis::with_elements(
            axis_id,
            label,
            tokens
                .into_iter()
                .map(|token| (token.id.clone(), token.text.clone())),
        )
    }
}

impl AiTraceEventKind {
    fn validate(
        &self,
        token_ids: &HashSet<&str>,
        tensor_ids: &HashSet<&str>,
    ) -> Result<(), ValidationError> {
        match self {
            Self::Tokens { token_ids: ids } => {
                if ids.is_empty() {
                    return Err(ValidationError::Empty {
                        component: COMPONENT,
                        field: "event token ids",
                    });
                }
                for id in ids {
                    require_reference("event token", id, token_ids)?;
                }
            }
            Self::Tensor { tensor_id } => {
                require_reference("event tensor", tensor_id, tensor_ids)?;
            }
            Self::Operation {
                operation,
                inputs,
                outputs,
            } => {
                validate_nonempty("operation", operation)?;
                if outputs.is_empty() {
                    return Err(ValidationError::Empty {
                        component: COMPONENT,
                        field: "operation outputs",
                    });
                }
                for id in inputs.iter().chain(outputs) {
                    require_reference("operation tensor", id, tensor_ids)?;
                }
            }
            Self::Generation {
                token_id,
                probability,
            } => {
                require_reference("generated token", token_id, token_ids)?;
                validate_probability("generation probability", *probability)?;
            }
            Self::Metric { name, value } => {
                validate_nonempty("metric name", name)?;
                if !value.is_finite() {
                    return Err(ValidationError::NonFinite {
                        component: COMPONENT,
                        field: "metric value",
                        value: *value,
                    });
                }
            }
            Self::Cue => {}
        }
        Ok(())
    }
}

fn validate_nonempty(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::Empty {
            component: COMPONENT,
            field,
        });
    }
    Ok(())
}

fn validate_probability(field: &'static str, value: f32) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::NonFinite {
            component: COMPONENT,
            field,
            value,
        });
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(ValidationError::OutOfRange {
            component: COMPONENT,
            field,
            minimum: 0.0,
            maximum: 1.0,
            value,
        });
    }
    Ok(())
}

fn require_reference<'a>(
    field: &'static str,
    value: &str,
    known: &HashSet<&'a str>,
) -> Result<(), ValidationError> {
    if !known.contains(value) {
        return Err(ValidationError::UnknownIdentifier {
            component: COMPONENT,
            field,
            value: value.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TRACE: &str = r#"
    {
      "schema_version": 1,
      "id": "lesson.attention",
      "metadata": { "model": "tiny-transformer", "layer": "0", "head": "0" },
      "tokens": [
        { "id": "token.0", "text": "AI", "index": 0, "byte_span": [0, 2] },
        { "id": "token.1", "text": "learns", "index": 1, "byte_span": [3, 9] }
      ],
      "tensors": [{
        "id": "embedding",
        "shape": [2, 1],
        "values": [0.25, 0.75],
        "axes": [
          { "id": "query", "label": "Tokens", "element_ids": ["token.0", "token.1"], "element_labels": ["AI", "learns"] },
          { "id": "feature", "label": "Features", "element_ids": ["f0"], "element_labels": ["x"] }
        ]
      }],
      "events": [
        { "id": "tokens", "timestamp": 0.0, "cue": "input", "type": "tokens", "token_ids": ["token.0", "token.1"] },
        { "id": "embedding", "timestamp": 0.5, "type": "tensor", "tensor_id": "embedding" }
      ]
    }
    "#;

    #[test]
    fn valid_json_loads_typed_tokens_tensors_and_events() {
        let trace = AiTrace::from_json_str(VALID_TRACE).unwrap();
        assert_eq!(trace.metadata.model.as_deref(), Some("tiny-transformer"));
        assert_eq!(trace.require_tensor("embedding").unwrap().shape, vec![2, 1]);
        let axis = trace.token_axis("query", "Tokens");
        assert_eq!(axis.element_ids, vec!["token.0", "token.1"]);
    }

    #[test]
    fn unknown_schema_versions_and_references_are_rejected() {
        let unsupported = VALID_TRACE.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1);
        assert!(matches!(
            AiTrace::from_json_str(&unsupported),
            Err(AiTraceError::Validation(ValidationError::Incompatible {
                field: "schema version",
                ..
            }))
        ));

        let unknown = VALID_TRACE.replacen(
            "\"tensor_id\": \"embedding\"",
            "\"tensor_id\": \"missing\"",
            1,
        );
        assert!(matches!(
            AiTrace::from_json_str(&unknown),
            Err(AiTraceError::Validation(
                ValidationError::UnknownIdentifier {
                    field: "event tensor",
                    ..
                }
            ))
        ));

        let unknown_event_field = VALID_TRACE.replace(
            "\"token_ids\": [\"token.0\", \"token.1\"]",
            "\"token_ids\": [\"token.0\", \"token.1\"], \"tensor_ids\": []",
        );
        assert!(matches!(
            AiTrace::from_json_str(&unknown_event_field),
            Err(AiTraceError::Json(_))
        ));
    }

    #[test]
    fn malformed_json_and_out_of_order_events_have_distinct_errors() {
        assert!(matches!(
            AiTrace::from_json_str("{"),
            Err(AiTraceError::Json(_))
        ));

        let out_of_order = VALID_TRACE
            .replacen("\"timestamp\": 0.5", "\"timestamp\": 0.0", 1)
            .replacen("\"timestamp\": 0.0", "\"timestamp\": 1.0", 1);
        assert!(matches!(
            AiTrace::from_json_str(&out_of_order),
            Err(AiTraceError::Validation(ValidationError::Incompatible {
                field: "event order",
                ..
            }))
        ));
    }

    #[test]
    fn checked_in_trace_fixture_loads_from_a_file() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/data/self_attention_trace.json");
        let trace = AiTrace::from_json_path(path).unwrap();

        assert_eq!(trace.metadata.layer.as_deref(), Some("encoder.layer.0"));
        assert_eq!(
            trace.require_tensor("embedding.query").unwrap().shape,
            [3, 3]
        );
        assert_eq!(
            trace.token_axis("query", "Tokens").element_labels,
            ["AI", "learns", "by"]
        );
    }
}
