# CLI UX and conventions

## Command model

Binary name: `jwt-tester`.

General structure:

```
jwt-tester <command> [options] [args]
```

Design for both:

- interactive use (humans),
- automation (pipelines, CI).

## Global flags (recommended)

- `--json`: machine-readable output (see `output.md`)
- `--no-color`: disable ANSI color even on TTY
- `--quiet`: suppress non-essential output (still prints primary result on success)
- `--verbose` / `-v`: include debug context (not secrets)
- `--version`: print version
- `--help`: print help

## STDIN / STDOUT rules

- Any argument that accepts a “string payload” should accept `-` as “read from stdin”.
- Prefer reading *all* stdin (`read_to_end` / `read_to_string`) instead of `read_line`.
  - Many pipelines produce data without a trailing newline or across multiple lines.
- In text mode:
  - If stdout is a TTY, include formatting and helpful labels.
  - If stdout is not a TTY, output only the raw token or raw JSON (so it composes).

## Configuration

Recommended precedence (lowest → highest):

1. Built-in defaults
2. Config file (optional): `~/.config/jwt-tester/config.toml` (or platform equivalent)
3. Environment variables (optional): `JWT_TESTER_*`
4. Command-line flags

Secrets should not be stored in config unless explicitly enabled and clearly documented.

### Vault/keychain configuration (jwt-tester)

- `JWT_TESTER_KEYCHAIN_SERVICE`: overrides the OS keychain “service” name used to store secret material.
  - default (current scaffold): `jwt-tester`

## Logging and diagnostics

Keep these rules:

- Never print secret material in logs.
- When `--json` is set, diagnostics should go into structured fields (not mixed into stdout).
- Consider printing non-fatal warnings to stderr in text mode.

## Shell completion

Provide:

- `jwt-tester completion bash|zsh|fish|powershell|elvish|nushell`

Output must be deterministic and should not require network access.
