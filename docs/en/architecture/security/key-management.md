# Key Management

This page documents key-management expectations for the source-available project. It
does not claim built-in integration with a specific KMS or Vault provider.

## Runtime Inputs

The gateway authentication path currently reads `JWT_SECRET`. Deployment
wrappers may expose higher-level `NEXIS_*` variables, but operator docs and
examples must state which value reaches the process environment.

Common secret inputs:

| Secret | Purpose | Notes |
| --- | --- | --- |
| `JWT_SECRET` | JWT signing and validation | Required for stable auth; required in production mode. |
| `DATABASE_URL` | Database connection | Inject through deployment secrets when persistence is enabled. |
| `REDIS_URL` | Redis connection | Inject through deployment secrets when Redis is enabled. |
| `OPENAI_API_KEY` | OpenAI provider access | Used only when OpenAI-backed AI features are configured. |

## Principles

- Inject secrets through the runtime environment or an operator-owned secret
  manager.
- Never store plaintext keys in source code, images, logs, telemetry labels, or
  exported dashboards.
- Apply least privilege to CI tokens, package publishing tokens, cloud
  credentials, and provider API keys.
- Rotate keys through auditable change records.
- Include lockfile and audit evidence when dependency updates are part of a
  security fix.

## Recommended Rotation Windows

| Secret type | Target window |
| --- | --- |
| Provider API keys | 90 days or stricter operator policy |
| Webhook and HMAC secrets | 90 days |
| JWT signing secrets | 90 days, plus immediate rotation after suspected exposure |
| Encryption keys | 180 days or stricter operator policy |

## Rotation Checklist

1. Generate the replacement secret with a cryptographically secure generator.
2. Store it in the operator-owned secret manager.
3. Deploy the new value to staging and verify login/token behavior.
4. Roll the production deployment.
5. Revoke or delete the old value after the migration window.
6. Record the commit, deployment, and verification evidence.

## Leak Response

If a secret is exposed:

1. Revoke the exposed value immediately.
2. Generate and deploy a replacement.
3. Review logs and audit records for use of the exposed value.
4. Rotate adjacent credentials if the blast radius is unclear.
5. Publish a private advisory or security issue when users need to act.

Use [SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md)
for vulnerability reporting and disclosure flow.
