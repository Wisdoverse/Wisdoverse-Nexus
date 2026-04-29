//! Skill registry abstraction and in-memory implementation.

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::error::{SkillError, SkillResult};
use crate::skill::Skill;

/// Registry operations for storing and retrieving skills.
#[async_trait]
pub trait SkillRegistry: Send + Sync {
    /// Register a new skill.
    async fn register(&self, skill: Skill) -> SkillResult<()>;

    /// Retrieve a skill by id.
    async fn get(&self, id: &str) -> SkillResult<Option<Skill>>;

    /// List all registered skills.
    async fn list(&self) -> SkillResult<Vec<Skill>>;

    /// Remove a skill by id.
    async fn remove(&self, id: &str) -> SkillResult<bool>;
}

/// In-memory skill registry suitable for local runtime use and tests.
#[derive(Default)]
pub struct InMemorySkillRegistry {
    skills: RwLock<HashMap<String, Skill>>,
}

impl InMemorySkillRegistry {
    /// Create an empty in-memory registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SkillRegistry for InMemorySkillRegistry {
    async fn register(&self, skill: Skill) -> SkillResult<()> {
        let mut skills = self
            .skills
            .write()
            .map_err(|err| SkillError::Execution(err.to_string()))?;
        let id = skill.metadata.id.clone();

        if skills.contains_key(&id) {
            return Err(SkillError::AlreadyExists(id));
        }

        skills.insert(id, skill);
        Ok(())
    }

    async fn get(&self, id: &str) -> SkillResult<Option<Skill>> {
        let skills = self
            .skills
            .read()
            .map_err(|err| SkillError::Execution(err.to_string()))?;
        Ok(skills.get(id).cloned())
    }

    async fn list(&self) -> SkillResult<Vec<Skill>> {
        let skills = self
            .skills
            .read()
            .map_err(|err| SkillError::Execution(err.to_string()))?;
        Ok(skills.values().cloned().collect())
    }

    async fn remove(&self, id: &str) -> SkillResult<bool> {
        let mut skills = self
            .skills
            .write()
            .map_err(|err| SkillError::Execution(err.to_string()))?;
        Ok(skills.remove(id).is_some())
    }
}
