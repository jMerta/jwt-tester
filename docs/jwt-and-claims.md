# JWT structure and claim handling

## JWT segments

A typical JWT (JWS) has 3 dot-separated base64url segments:

1. header (JSON)
2. payload (JSON)
3. signature (bytes)

The CLI must:

- base64url-decode header and payload,
- parse them as JSON objects (or return a clear error),
- avoid assuming the payload is trustworthy unless `verify` passed.

## Registered claims (common)

- `iss` (issuer): string
- `sub` (subject): string
- `aud` (audience): string or array of strings
- `exp` (expiration time): NumericDate
- `nbf` (not before): NumericDate
- `iat` (issued at): NumericDate
- `jti` (JWT ID): string

Your CLI should:

- allow setting these via dedicated flags in `encode`,
- allow validating them via dedicated flags in `verify`.

## NumericDate conversion

NumericDate is typically a UNIX timestamp (seconds since epoch).

For `decode`/`inspect`:

- if `--date` is provided, render `exp/nbf/iat` as RFC3339 strings.
- preserve the original numeric value in JSON output (or provide both).

## Durations and relative times

Good UX examples:

- `--exp +30m`
- `--nbf -10s`
- `--exp "2 days"`

Implementation tip:

- support human duration parsing,
- convert relative durations relative to “now” at execution time.

Be explicit in docs whether relative times are interpreted in seconds and whether “ago” is supported.

