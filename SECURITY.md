# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.x | Yes |

## Reporting a Vulnerability

Please do **not** report security vulnerabilities through public GitHub issues.

Use the private GitHub Security Advisory channel:

- <https://github.com/Wisdoverse/Wisdoverse-Nexus/security/advisories/new>

Recommended report content:

- Affected component and version
- Reproduction steps
- Impact assessment
- Suggested remediation (if available)

## Response Targets

On business days:

- Acknowledgement within 24 hours
- Initial triage within 72 hours
- Coordinated remediation timeline based on severity

## Disclosure Process

1. Report received and tracked with an internal incident ID.
2. Severity assessment (CVSS plus business impact).
3. Mitigation and patch implementation.
4. Coordinated release and advisory publication.
5. Incident closure with postmortem actions.

## Security Best Practices

- Run production workloads with least privilege (non-root containers, scoped tokens, and short-lived credentials).
- Enforce TLS termination at the edge and enable `NEXIS_HTTPS_REDIRECT_ENABLED=true` and `NEXIS_HSTS_ENABLED=true` in production.
- Restrict CORS origins with `NEXIS_CORS_ALLOW_ORIGINS` to trusted domains only.
- Keep secrets out of source control and inject them at runtime through your secrets manager.
- Enable centralized logging and metrics with retention policies that meet your compliance requirements.
- Apply database, host, and network hardening baselines alongside application controls.

## Dependency Update Policy

- Security patches for direct dependencies are prioritized and merged after validation is complete.
- Critical vulnerabilities (CVSS 9.0-10.0) target patch release within 48 hours when practical.
- High vulnerabilities (CVSS 7.0-8.9) target patch release within 7 calendar days when practical.
- Medium vulnerabilities (CVSS 4.0-6.9) are reviewed in the next scheduled maintenance window.
- Dependency scanning is expected in CI, and lockfiles should be updated in the same change as dependency upgrades.
- Any exception to upgrade timelines must be documented with compensating controls and expiry date.

## Disclosure Timeline

After a valid report is received:

1. `T+0-1 day`: Acknowledge reporter and begin triage.
2. `T+1-3 days`: Confirm impact, define remediation plan, and assign owner.
3. `T+3-14 days`: Prepare, test, and stage fixes (timeline varies by severity and blast radius).
4. `T+14-30 days`: Publish patched release and advisory notes, unless active exploitation requires accelerated release.
5. `T+30-90 days`: Complete follow-up hardening and close post-incident actions.

## Security Baselines

Detailed controls are documented in:

- [Baseline Profile](docs/en/architecture/security/baseline-profile.md)
- [Enterprise Profile](docs/en/architecture/security/enterprise-profile.md)
- [Key Management](docs/en/architecture/security/key-management.md)

## Safe Harbor

We support good-faith security research. If you avoid privacy violations,
service disruption, and data destruction, and comply with applicable laws,
we will treat your report as responsible disclosure.
