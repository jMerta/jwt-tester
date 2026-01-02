# Roadmap (optional enhancements)

These are “nice” features that can differentiate your tool once the core is solid.

## Verification experience

- `verify --explain`: prints the exact checks performed and why it failed.
- `verify --policy @policy.json`: load a validation policy (allowed algs, required claims, issuer/aud lists).

## Key discovery and caching

- `--jwks https://...` with:
  - caching (TTL),
  - pinning (expected issuer or expected key thumbprints),
  - offline mode.
- `--oidc <issuer>`:
  - fetch `/.well-known/openid-configuration`,
  - use `jwks_uri`,
  - optionally validate issuer.

## Developer quality of life

- `jwt repl`: interactive mode to build/inspect tokens.
- `jwt sample`: generate sample payloads/templates.
- `jwt bench`: benchmark encode/verify speed.

## CLI ergonomics

- `doctor`/`config` command to show resolved settings (data dir, keychain backend, UI bind) with redacted secrets.
- Vault rename/update commands (project/key/token) plus filtering and sorting for list outputs.
- Vault summary/status view (counts, default key per project).

## Output and integration

- `--jq <expr>`: apply a jq-like filter to JSON output (or document piping to `jq`).
- `--format yaml`: optional YAML output (if needed).

## Local UI enhancements

- `ui --profile <name>`: multiple vault profiles (work/personal) with separate encryption.
- “policy presets” for verify (allowed algs, issuer/audience sets).
- OIDC helper in UI:
  - paste issuer URL
  - fetch discovery + JWKS (opt-in; clearly labeled network access)
  - pin expected issuer and key thumbprints.
