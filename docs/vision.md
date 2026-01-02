# Vision and differentiators

## Goal

Build a CLI tool that makes working with JWTs **fast**, **scriptable**, and **safe by default**.

Success means:

- It is easy to generate test tokens for development.
- It is easy to inspect tokens in pipelines.
- It is hard to accidentally “verify” a token insecurely.
- Output is consistent and machine-readable when requested.
- Errors are actionable and the process exits with predictable codes.

## Non-goals (initially)

- Full JWE (encrypted JWT) support.
- Full JWK management suite (rotations, HSM integrations).
- Being a full OpenID Connect client (we can optionally add discovery later).
- A full “JWT debugger UI” is optional, but a **local-only UI mode** (`jwt-tester ui`) is in-scope if it stays offline and secure-by-default.

## Product principles

- **No surprises**: avoid silent inference of security-sensitive settings; be explicit in docs and output.
- **Secure by default**: verification requires explicit intent and safe validation rules.
- **Pipeline-first**: design for `stdin`/`stdout`, JSON output, and stable exit codes.
- **Errors are data**: structured errors for JSON output; human-friendly otherwise.
- **No panics on user input**: invalid JSON, invalid keys, missing files → clean error + non-zero exit.

## How to be better than the reference CLI

The reference CLI in this workspace is a good baseline. Here’s how we can do better.

### 1) Safer defaults for verification

- **Prefer explicit `alg` when verifying**. The header is attacker-controlled.
  - Allow inference only when `--alg` is omitted and document the trade-offs clearly.
- Default to **audience validation enabled** when `--aud` is provided, and clearly document behavior.
- Default leeway to something small (e.g. `30s`) and make it configurable.

### 2) Clear separation of “decode” vs “verify”

- Provide:
  - `decode`: parses and prints contents without claiming trust.
  - `verify`: cryptographically verifies signature and validates claims.
  - `inspect`: prints helpful summaries (claims + timestamp conversions) without implying trust.

### 3) Better key and JWKS ergonomics

- `verify --jwks <file|url|-> --kid <kid>` with caching for URLs (opt-in).
- Support selecting keys by:
  - `kid`,
  - JWK thumbprint (`x5t` / RFC 7638 thumbprint),
  - “only one key in JWKS” fallback (explicit flag).
- Support reading secrets/keys from:
  - `@file` (like the reference),
  - `env:NAME`,
  - `stdin` (e.g. `--key -`),
  - OS keychain integration (optional future).

### 4) Better UX for JSON payload input

- Accept payloads from:
  - inline JSON,
  - `@file.json`,
  - stdin (`-`),
  - `--claim key=value` with automatic JSON typing,
  - `--claim-file @claims.json` to merge.
- Provide deterministic merging rules and conflict behavior.

### 5) Better output contracts

- Consistent JSON schema for `--json` across commands:
  - `ok: boolean`, `error: { code, message, details? }`, `data: ...`.
- `--quiet` and `--no-color` for scripts.
- “pretty text” for humans and “compact text” for pipes.

### 6) Better security messaging

- When outputting decoded-but-unverified tokens, clearly label them as **UNVERIFIED** in text mode.
- Provide `--explain` on `verify` to output what validations were applied (alg, key selection, claim checks).

### 7) Higher engineering quality

- No `panic!/unwrap/expect` on user-driven paths; use typed errors.
- Structured error codes for stable scripting.
- Property tests / fuzzing for parsers (optional but recommended).

## “Better than others” features (optional but compelling)

- `verify --oidc <issuer-url>`: fetch OIDC discovery + JWKS (cache, pinning options).
- `encode --template <name>`: token templates (common claim sets) for internal tooling.
- `repl` mode: interactive JWT workshop.
- `bench` mode: measure verification speed (useful for services).
