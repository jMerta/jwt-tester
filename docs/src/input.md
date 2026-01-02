# Inputs: tokens, JSON, secrets, keys

## Token input

Every command that accepts a token should accept either:

- a positional token argument, or
- `-` meaning “read token from stdin”.

Token reading rules:

- Trim surrounding whitespace.
- Reject tokens that do not have exactly 3 segments for JWS.

## JSON input sources

For claims and headers, support these sources:

- inline JSON string: `'{ "sub": "123" }'`
- stdin: `-`
- file: `@claims.json`

Recommended conventions:

- `@path` means “read file contents”.
- `@-` is not recommended (ambiguous); prefer `-` only.

## `--claim k=v` typing rules

Users want:

```
--claim admin=true
--claim n=123
--claim tags='["a","b"]'
```

Suggested parsing:

1. Try to parse the value as JSON (number/bool/null/array/object/string).
2. If that fails, treat it as a string.

### Merge order (deterministic)

Define and document a merge order; example:

1. Base claims from `<CLAIMS_JSON|-|@file>`
2. Add standard claim flags (`--iss`, `--sub`, …)
3. Apply repeated `--claim k=v` in command-line order

Conflict rule: last write wins.

## Secret and key input

You need to support at least:

- HMAC secrets for HS256/384/512
- Private keys for signing (RSA/ECDSA/EdDSA)
- Public keys or JWKS for verification

### Input forms

For secret-like inputs (secrets, keys, tokens, passphrases), the current CLI supports:

- raw string (careful: shell history)
- `prompt[:LABEL]` to read securely from an interactive prompt
- `@path` file input
- `-` stdin input (safe for secrets in scripts)
- `env:NAME` to read from environment
- `b64:BASE64` to decode base64 bytes (crypto commands only; see `--help`)

Note: exact forms vary slightly by command; check the specific `--help` output.

### Format detection vs explicit type

You have two good options:

1. Explicit flags:
   - `--key-pem`, `--key-der`, `--jwks`
2. Heuristics with override:
   - infer from file extension or content, but allow `--key-format`.

Avoid “silent mis-detection” by:

- failing with a clear error when ambiguity exists.

### JWKS key selection

When JWKS contains multiple keys, define selection:

- Prefer `kid` match when JWT header has `kid`.
- If no `kid`:
  - require `--kid` OR
  - require `--allow-single-jwk` and only proceed if JWKS has exactly 1 key.

## Avoiding secret leakage

Document these recommendations:

- Prefer `--secret -` (stdin) or `--secret @file` over inline secrets.
- Redact secrets from logs even in `--verbose`.
