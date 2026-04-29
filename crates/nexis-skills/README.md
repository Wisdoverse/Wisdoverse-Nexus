# nexis-skills

Skill registry and execution primitives for Wisdoverse Nexus AI agents.

## What It Provides

- `Skill` and `SkillMetadata` domain types
- `SkillExecutor` trait for runtime skill handlers
- `SkillRegistry` trait for registry backends
- `InMemorySkillRegistry` for basic runtime/testing use
- `SkillExecution` and `ExecutionContext` execution models
- `SkillError` and `SkillResult` for unified error handling

## Notes

- Execution orchestration is intentionally lightweight.
- Extend the registry trait with persistent implementations as needed.
