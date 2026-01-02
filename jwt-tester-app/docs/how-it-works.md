# How jwt-tester-app works

This document explains how the `jwt-tester-app` binary is structured and how
data flows through the CLI, vault, and UI layers.

## High-level flow

- `main` uses `clap` to parse flags and subcommands, then dispatches to:
  - `commands/` for CLI workflows (`encode`, `verify`, `decode`, `inspect`, `split`),
  - `ui/` for the local web UI,
  - `vault/` for persistence operations (projects/keys/tokens/export/import).
- Output is normalized through `CommandOutput` + `emit_ok` / `emit_err` so JSON
  output is consistent across commands.

## Input handling (CLI)

All secret-bearing flags and token inputs use shared helpers:

- `read_input` (string inputs)
- `read_input_bytes` (key/secret bytes)

Supported formats:

- `prompt` / `prompt:LABEL` (TTY-only, no-echo input)
- `-` (read from stdin)
- `@file` (read file contents)
- `env:NAME` (read from environment variable)
- `b64:BASE64` (bytes-only inputs)
- any other value is treated as a literal

`prompt` requires a TTY; if stdin is not a terminal, an error explains how to
use `-`, `@file`, or `env:NAME` instead.

## Vault and persistence

`VaultConfig` selects between in-memory and persistent modes:

- `--no-persist` → in-memory only (no disk writes)
- default → SQLite metadata + keychain-backed secret material

### Data directory

The data directory is resolved via `ProjectDirs::from("dev", "jwt-tester", "jwt-tester")`.
Override with `--data-dir`. The SQLite database lives at:

```
<data-dir>/vault.sqlite3
```

### Metadata vs secret storage

Metadata lives in SQLite:

- Projects: id, name, created_at, default key, description, tags
- Keys: id, project_id, name, kind, created_at, kid, description, tags
- Tokens: id, project_id, name, created_at

Secret material is stored in a keychain backend:

- OS keychain (default)
- File-backed encrypted keychain (Docker-only)

### Key generation (UI + CLI)

Both the UI and CLI can generate key material when you do not already have one:

- UI: `POST /api/vault/keys/generate` returns the material for reveal/download.
- CLI: `vault key generate` uses the same keygen module; `--reveal` includes the
  material in output and `--out` writes it to a file.

Generated HMAC secrets are base64url strings. RSA/EC/EdDSA keys are PKCS8 PEM.

### Keychain backends

OS keychain backend:

- Default outside Docker.
- Uses `JWT_TESTER_KEYCHAIN_SERVICE` (default: `jwt-tester`).
- Accounts are stored as `key:<uuid>` and `token:<uuid>`.

File backend (Docker UI):

- Enable with `JWT_TESTER_KEYCHAIN_BACKEND=file` and `JWT_TESTER_KEYCHAIN_PASSPHRASE`.
- Optional `JWT_TESTER_KEYCHAIN_DIR` (defaults to `<data-dir>/keychain`).
- Requires Docker context: `JWT_TESTER_DOCKER=1` and `/.dockerenv`.
- Stores encrypted JSON per entry (one file per key/token).
- Test runs can set `JWT_TESTER_DOCKER_TEST=1` in debug builds to bypass the
  Docker marker (used only by integration tests).

## Key resolution and verification

`encode`, `verify`, and `decode` share key resolution via `key_resolver`:

1. Direct input:
   - `--secret` (HMAC)
   - `--key` (PEM/DER)
   - `--jwks` (JSON Web Key Set)
2. Vault project selection:
   - explicit `--key-id` / `--key-name`
   - token `kid` match
   - project default key
   - single-key fallback

If none of the above yields a single key, the command returns a user-facing
error with guidance (e.g., set a default key or specify a key id).

## UI server

`jwt-tester ui` starts a local Axum server:

- Binds to `127.0.0.1` by default (`--allow-remote` required for non-loopback)
- Serves static assets at `/` and `/assets/*`
- Exposes JSON APIs under `/api/*`
- The UI module is behind the `ui` Cargo feature (enabled by default).
- The `jwt-tester-cli` binary excludes the UI command entirely.
- Vault Manager can generate HMAC secrets and RSA/EC/Ed25519 key material directly in the UI.

Frontend build:

- The UI is a standalone React app built with Vite (`jwt-tester-app/ui/`).
- `jwt-tester ui` expects a prebuilt `ui/dist` unless you pass `--build` or `--dev`.
- `jwt-tester ui --build` runs `npm install` + `npm run build` (disabled if
  `JWT_TESTER_UI_ASSETS_DIR` is set).
- If npm is not on PATH, set `JWT_TESTER_NPM` or pass `--npm` with the npm
  executable path (on Windows prefer `npm.cmd`).
- `npm run build` outputs static files to `jwt-tester-app/ui/dist`.
- The Rust server serves `index.html` and `/assets/*` from that build output.
- Override the assets directory with `JWT_TESTER_UI_ASSETS_DIR` (useful for
  packaged builds or custom locations).

Dev mode:

- `jwt-tester ui --dev` starts the API server and launches the Vite dev server
  (hot reload) at `http://127.0.0.1:5173`.
- The Vite dev server proxies `/api` to the Rust backend using
  `JWT_TESTER_API_URL` (set automatically by the Rust command).
- The frontend fetches the CSRF token from `/api/csrf` when the meta tag is not
  present (dev server case).

Security behavior:

- A per-process CSRF token is generated and embedded in the served HTML.
- All mutating endpoints (POST/DELETE) require the `x-csrf-token` header.
- Basic security headers are set (CSP, XFO, nosniff, referrer policy).
- Cross-origin modifying requests are blocked if `Origin` is not localhost.

The UI uses the same `Vault` implementation and respects `--no-persist` and
`--data-dir` just like the CLI.

## Docker UI mode

The Docker image runs the UI by default with file-backed keychain persistence:

- `CMD ["--data-dir","/data","ui","--host","0.0.0.0","--port","3000","--allow-remote"]`
- `JWT_TESTER_KEYCHAIN_BACKEND=file`
- `JWT_TESTER_DOCKER=1`
- `VOLUME /data`

## CLI-only build

To build without the UI dependencies, use the helper script:

- Windows: `.\jwt-tester-app\build.ps1 --cli-only`
- macOS/Linux: `./jwt-tester-app/build.sh --cli-only`

This produces the `jwt-tester-cli` binary. Under the hood it maps to
`cargo build --manifest-path jwt-tester-app/Cargo.toml --no-default-features --features cli-only`.

## Testing notes

- Unit tests cover command logic, vault operations, and IO helpers.
- Integration tests (in `tests/`) run the CLI with a temp data dir and file
  keychain backend, so the OS keychain is never touched.
