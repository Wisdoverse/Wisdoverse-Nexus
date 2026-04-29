# Security Posture

This page summarizes the public security posture for the source-available repository.
It is not a third-party audit, certification report, or compliance attestation.

## Scope

Covered areas:

- Rust workspace dependency and compile checks
- Node workspace dependency checks
- GitHub Actions security scanning
- Gateway security controls documented in the repository
- Public vulnerability reporting process

Not covered:

- Third-party penetration testing
- SOC 2, ISO 27001, HIPAA, or GDPR certification
- Production environment controls outside this repository
- Cloud infrastructure operated by downstream users

## Repository Controls

| Control | Status | Evidence |
| --- | --- | --- |
| Private vulnerability reporting | Enabled | `SECURITY.md` and GitHub Security Advisories |
| Rust lint gate | Enabled | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust dependency audit | Enabled in CI | `cargo audit` |
| Node dependency audit | Enabled in CI | `pnpm audit` plus package-local `npm audit` |
| Secret scanning | Enabled in CI | TruffleHog OSS workflow |
| Container scanning | Enabled in CI | Trivy workflow |
| Dependabot | Enabled | Cargo, npm, Docker, and GitHub Actions ecosystems |

## Local Verification

```bash
cargo check --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm audit --audit-level moderate
npm --prefix apps/mobile audit --audit-level=moderate
npm --prefix docs audit --audit-level=moderate
npm --prefix apps/web/e2e audit --audit-level=moderate
```

For Rust advisory checks, install `cargo-audit` and run:

```bash
cargo audit
```

## Application Security Baselines

Security-sensitive behavior is documented in:

- [Security architecture overview](../architecture/security/overview.md)
- [Baseline profile](../architecture/security/baseline-profile.md)
- [Enterprise profile](../architecture/security/enterprise-profile.md)
- [Key management](../architecture/security/key-management.md)

Production operators are responsible for configuring TLS termination, secret
management, CORS origins, rate limits, database access, log retention, and
network controls for their own environment.

## Vulnerability Reporting

Do not report vulnerabilities through public GitHub issues. Use the private
GitHub Security Advisory channel documented in
[SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md).

Reports should include:

- Affected component and version or commit SHA
- Reproduction steps
- Expected impact
- Logs, request samples, or proof of concept when safe to share
- Suggested remediation if available

## Disclosure Policy

Maintainers triage security reports privately, prepare a fix, publish a release
or patch guidance, and then publish advisory details when coordinated disclosure
is appropriate.
