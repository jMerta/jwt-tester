# Implementation architecture (recommended)

This is a language-agnostic architecture that maps well to Rust, Go, Node, etc.

## Modules / layers

1. `cli` (argument parsing)
2. `io` (read stdin, files, env)
3. `model` (types: TokenParts, Claims, Header, KeyMaterial)
4. `jwt` (encode/decode/verify primitives)
5. `commands` (decode/encode/verify/inspect orchestration)
6. `output` (text/json renderers + exit code mapping)
7. `ui` (optional): local HTTP server + embedded frontend
8. `store` (optional): local vault (projects, keys, sample tokens)

Keep cryptographic operations behind a small interface so you can swap libraries later.

## Error model

Define a small set of stable error categories:

- Invalid input token
- Invalid JSON (header/claims)
- Invalid key material / unsupported key format
- Signature invalid
- Claim validation failed
- Internal error

Every command should map errors to:

- stable `error.code` (JSON)
- stable exit code
- a human-friendly message

## Decode/verify flow

Recommended:

- `decode`:
  - parse segments
  - decode header/payload bytes
  - parse JSON
  - output

- `verify`:
  - parse segments
  - decode header/payload bytes
  - parse JSON
  - choose algorithm (from CLI flags, not header)
  - load key material
  - verify signature
  - validate claims
  - output success + optional `--explain`

## Key loading

Centralize key loading so behavior is uniform across commands.

Support:

- `@file`
- stdin (`-`)
- env (`env:NAME`) (optional)

## UI architecture (optional)

Keep the UI as a thin wrapper around the same “core” that powers the CLI commands.

Suggested split:

- `core`:
  - token parsing, encoding, verifying
  - claim merging and time rendering
  - key parsing and JWKS selection
- `ui-server`:
  - serves static frontend assets
  - exposes a local JSON API that calls into `core`
  - enforces security headers + CSRF protections
- `vault`:
  - encrypted persistence layer (SQLite or file-based)
  - stores metadata for projects + keys + tokens; secret bytes live in OS keychain by default

Avoid “frontend holds the keys” unless you intentionally choose an in-browser-only vault design (and document tradeoffs).

## Determinism and reproducibility

For `encode`:

- define whether claim key order matters
- if you offer “preserve order”, document it clearly (JSON objects are unordered by spec)
