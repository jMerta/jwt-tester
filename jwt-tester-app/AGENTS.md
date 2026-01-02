# Agent instructions (scope: jwt-tester-app)

## Scope
- Applies to: `jwt-tester-app/` and subdirectories
- Languages/tooling: Rust (cargo), SQLite (rusqlite), Axum (UI)

## Architecture (high-level)
- Style: clean
- Boundaries:
  - `cli/` defines argument models only; no side effects or I/O.
  - `commands/` orchestrates app flows; may call `vault/`, `key_resolver/`, `jwt_ops/`, `claims/`, `output/`, `io_utils/`.
  - `vault/` owns storage/keychain; must not depend on `commands/` or `ui/`.
  - `ui/` uses shared logic (`vault/`, `jwt_ops/`, `claims/`, `key_resolver/`) but must not depend on `commands/` or `cli/`.
  - `key_resolver/` may depend on `vault/`, `jwks/`, `io_utils/`, `jwt_ops/`; not on `commands/` or `ui/`.
  - Core utilities (`error/`, `jwt_ops/`, `claims/`, `date_utils/`, `io_utils/`, `jwks/`, `output/`) should not depend on `commands/` or `ui/`.

## Conventions
- Formatting: `cargo fmt` (auto-fix enabled)
- Linting: `cargo clippy -- -D warnings`
- Tests: `cargo test`
- Use `AppError`/`AppResult` for user-facing errors; avoid `unwrap()`/`expect()` outside tests.
- Never log secrets; redact key/token material in errors/logs/output.
- Tests must not touch the OS keychain; use `TempDir` + `MemoryKeychain` for sqlite tests.
- Keep files reasonably sized; split by domain (e.g., `vault/project`, `vault/key`, `vault/token`, `vault/export`).

## Commands
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Test: `cargo test`

## Verifiable config (used by `coding-guidelines-verify`)
```codex-guidelines
{
  "version": 1,
  "format": {
    "autofix": true,
    "commands": ["cargo fmt"],
    "windows": [],
    "posix": []
  },
  "lint": { "commands": ["cargo clippy -- -D warnings"], "windows": [], "posix": [] },
  "test": { "commands": ["cargo test"], "optional": false, "windows": [], "posix": [] },
  "rules": {
    "forbid_globs": [],
    "forbid_regex": []
  }
}
```
