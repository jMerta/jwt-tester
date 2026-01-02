# CLI UX and conventions

## Command model
Binary name: `jwt-tester`.
General structure:
```
jwt-tester <command> [options] [args]
```

Global flags must appear before the subcommand (e.g. `jwt-tester --json decode <TOKEN>`).

Design for both:
- interactive use (humans),
- automation (pipelines, CI).

## Global flags (current)
- `--json`: machine-readable output (see `output.md`)
- `--no-color`: disable ANSI color even on TTY
- `--quiet`: suppress non-essential output (still prints primary result on success)
- `--verbose` / `-v`: include debug context (not secrets)
- `--no-persist`: keep vault metadata in memory only (no SQLite)
- `--data-dir <PATH>`: override the data directory used for persistence
- `--version` / `-V`: print version
- `--help` / `-h`: print help
## STDIN / STDOUT rules

- Any argument that accepts a “string payload” should accept `-` as “read from stdin”.
- Prefer reading *all* stdin (`read_to_end` / `read_to_string`) instead of `read_line`.
  - Many pipelines produce data without a trailing newline or across multiple lines.
- In text mode:
  - If stdout is a TTY, include formatting and helpful labels.
  - If stdout is not a TTY, output only the raw token or raw JSON (so it composes).

## Configuration (current)

The current CLI does not load a config file. It only honors specific environment variables plus CLI flags.

Precedence (lowest → highest):

1. Built-in defaults
2. Environment variables (only those documented below)
3. Command-line flags

Secrets should not be stored in config unless explicitly enabled and clearly documented.

### Vault/keychain configuration (jwt-tester)

- `JWT_TESTER_KEYCHAIN_SERVICE`: overrides the OS keychain “service” name used to store secret material.
  - default: `jwt-tester`
- `JWT_TESTER_KEYCHAIN_BACKEND`: `os` (default) or `file` (Docker-only).
- `JWT_TESTER_KEYCHAIN_PASSPHRASE`: required when `JWT_TESTER_KEYCHAIN_BACKEND=file`.
- `JWT_TESTER_KEYCHAIN_DIR`: optional override for the file-backend storage directory.
- `JWT_TESTER_DOCKER`: set to `1` to allow the file keychain backend (used in Docker).

### UI build configuration (jwt-tester ui)

- `JWT_TESTER_UI_ASSETS_DIR`: override the location of prebuilt UI assets.
- `JWT_TESTER_NPM`: override the npm executable used for UI builds.

## Logging and diagnostics

Keep these rules:

- Never print secret material in logs.
- When `--json` is set, diagnostics should go into structured fields (not mixed into stdout).
- Consider printing non-fatal warnings to stderr in text mode.

## Shell completion

Provide:

- `jwt-tester completion bash|zsh|fish|powershell|elvish|nushell`

Output must be deterministic and should not require network access.



