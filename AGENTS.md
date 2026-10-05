# Agent instructions

Wisdoverse Nexus is a Rust and TypeScript repository for human and AI collaboration.

## Read the required guidance

Read each document when its condition applies.

| Condition | Document |
| --- | --- |
| Any task | [Writing rules](.agents/writing.md) |
| Code, configuration, commands, or verification | [Development guide](.agents/development.md) |
| Frontend, backend, service, or deployment changes | [Architecture decision](docs/en/architecture/adr/008-fsd-ddd-cloud-native-services.md) |

Use Feature-Sliced Design (FSD) for Web and mobile frontends.
Use Domain-Driven Design (DDD) for backends.
Apply the cloud-native requirements in the architecture decision to service and deployment work.

## Do the work

- Preserve unrelated local changes.
- For nontrivial tasks, define the scope and acceptance checks before edits.
- Assign simple, bounded tasks to Luna when it is available.
- Keep architecture decisions and security-sensitive changes with the lead agent.
- Review delegated changes before you accept them.
- Use synthetic data and reserved example domains in public material.
- Never commit secrets.
- Remove credentials, personal details, private addresses, and sensitive payloads before publication.
- For vulnerability reports, use the private process in [SECURITY.md](SECURITY.md).

## Complete the task

- Run the checks for the changed areas in the development guide.
- Record the commands, results, and checks that you could not run.
- Use scoped Conventional Commits, as described in [CONTRIBUTING.md](CONTRIBUTING.md).
- For user-visible changes, update [CHANGELOG.md](CHANGELOG.md) under `Unreleased`.
- In the PR, explain the change, its reason, and the verification evidence.
- Include relevant issue links and evidence for UI, CLI, or deployment changes.
- Keep this file within 40 lines and 350 words.
- Put detailed rules in the linked documents.
