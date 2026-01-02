# Outputs: formats, stderr, exit codes

## Output streams

- **stdout**: primary output (token, JSON result, decoded claims)
- **stderr**: errors and warnings (in text mode)

When `--json` is provided (global flag placed before the subcommand, e.g. `jwt-tester --json decode <TOKEN>`):

- stdout must be valid JSON only (no extra banners).
- stderr should be empty; errors are emitted as JSON on stdout in the current CLI.

## Text output conventions

For `decode`/`inspect`:

- Make it obvious when content is not verified (e.g. prefix with `UNVERIFIED`).
- If verification succeeds (key provided), label as `VERIFIED`.
- Print header then payload, each as pretty JSON.

For `encode`:

- Print only the token by default (so it pipes cleanly).
- If additional info is needed, place it behind `--verbose`.

## JSON output schema (recommended)

Use a consistent envelope:

```json
{
  "ok": true,
  "data": { }
}
```

On error:

```json
{
  "ok": false,
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Token must have 3 segments",
    "details": { }
  }
}
```

Keep `error.code` stable over time.

## Exit codes (recommended)

Pick a stable contract; example mapping:

- `0`: success
- `10`: invalid input token (format/base64/json)
- `11`: signature invalid
- `12`: claims invalid (exp/nbf/iss/aud/…)
- `13`: key/secret invalid
- `14`: internal error

Document these in `--help` and in README.
