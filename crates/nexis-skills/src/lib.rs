//! Wisdoverse Nexus Skills - skill registry and execution primitives for AI agents.

pub mod error;
pub mod executor;
pub mod registry;
pub mod skill;

pub use error::{SkillError, SkillResult};
pub use executor::{ExecutionContext, ExecutionStatus, SkillExecution};
pub use registry::{InMemorySkillRegistry, SkillRegistry};
pub use skill::{Skill, SkillExecutor, SkillMetadata};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::error::{SkillError, SkillResult};
    pub use crate::executor::{ExecutionContext, ExecutionStatus, SkillExecution};
    pub use crate::registry::{InMemorySkillRegistry, SkillRegistry};
    pub use crate::skill::{Skill, SkillExecutor, SkillMetadata};
}
