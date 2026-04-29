//! Skill definitions and execution interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::SkillResult;
use crate::executor::ExecutionContext;

/// Describes a skill's identity and discoverability metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Stable skill identifier.
    pub id: String,
    /// Human-readable skill name.
    pub name: String,
    /// Short description of what the skill does.
    pub description: String,
    /// Optional tags for categorization.
    pub tags: Vec<String>,
}

impl SkillMetadata {
    /// Construct metadata with no tags.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            tags: Vec::new(),
        }
    }
}

/// Skill definition that can be stored in a registry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    /// Metadata describing the skill.
    pub metadata: SkillMetadata,
    /// Input schema represented as JSON Schema-like value.
    pub input_schema: Value,
    /// Output schema represented as JSON Schema-like value.
    pub output_schema: Value,
    /// Indicates whether this skill is currently enabled.
    pub enabled: bool,
}

impl Skill {
    /// Create a skill with permissive schema defaults.
    #[must_use]
    pub fn new(metadata: SkillMetadata) -> Self {
        Self {
            metadata,
            input_schema: Value::Object(serde_json::Map::new()),
            output_schema: Value::Object(serde_json::Map::new()),
            enabled: true,
        }
    }
}

/// Trait implemented by concrete skill executors.
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    /// Execute a skill with input payload and context.
    async fn execute(&self, input: Value, ctx: ExecutionContext) -> SkillResult<Value>;
}
