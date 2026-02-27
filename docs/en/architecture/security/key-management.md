# Key Management

Principles:

- Inject secrets only through environment variables or KMS/Vault.
- Never store plaintext keys in source code, images, logs, or telemetry tags.
- Apply least-privilege key access.
- Track key rotation through auditable change records.

Typical rotation windows:

- Provider API keys: 90 days
- Webhook and HMAC secrets: 90 days
- Encryption keys: 180 days (or stricter policy)
