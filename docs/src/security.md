# Security model and pitfalls

This document is the “do not accidentally build a footgun” section.

## Threat model (what an attacker controls)

In almost all real situations, an attacker can control:

- the entire token string,
- header values (`alg`, `kid`, etc.),
- payload values (claims).

Therefore:

- treat header and claims as untrusted input until verification passes.

## Algorithm confusion

Prefer explicit algorithms when verifying. `jwt-tester` can infer `alg` from the
token header when `--alg` is omitted, but this is weaker and should be used with
caution.

Safer patterns:

- `verify --alg RS256 --key @public.pem <token>`
- `verify --jwks @jwks.json --alg RS256 <token>`

Less safe (supported, but avoid in sensitive contexts):

- `verify --key @public.pem <token>` while implicitly using header `alg`

## Disallow `alg=none`

Unless you have a very specific internal use case:

- reject `none` for `encode`,
- reject `none` for `verify`,
- allow `decode` to show it as metadata only.

## Key selection rules

If JWKS has multiple keys:

- require `kid` selection (from token header or explicit `--kid`).
- avoid trying “all keys until one works” unless behind an explicit flag, because:
  - it can mask configuration errors,
  - it can leak timing info and cost CPU.

## Claim validation

Recommended defaults for `verify`:

- Validate `exp` (unless `--ignore-exp`).
- Validate `nbf` (if present).
- Use small leeway (e.g. 30s).
- If `--iss/--aud/--sub` is provided, validate it strictly.

Be careful with `aud`:

- it may be a string or array; implement both.

## Output safety

- Never print secrets.
- Don’t encourage putting secrets on the command line (shell history).
- Provide `--secret -` patterns and document them as recommended.

## Local UI mode (`jwt-tester ui`)

A localhost UI increases the attack surface because it introduces an HTTP server and a browser context.

Safe-by-default requirements:

- **Bind to localhost only** by default (`127.0.0.1` / `::1`). Do not listen on `0.0.0.0` unless the user explicitly opts in (and the UI displays a strong warning).
- **No remote dependencies by default**: avoid loading JS/CSS/fonts from CDNs; prefer embedded/bundled assets.
- **CSRF / origin protections**:
  - verify `Origin`/`Host` headers,
  - require a per-session CSRF token (header-based or same-site cookie strategy),
  - disable CORS by default.
- **Clickjacking protection**: set `X-Frame-Options: DENY` (or CSP `frame-ancestors 'none'`).
- **Content Security Policy**: lock down `script-src` to self (and avoid inline scripts where possible).

Secrets-at-rest requirements:

- Treat vault storage as sensitive:
  - **preferred**: store secret blobs directly in OS keychain / credential manager,
  - optional: support passphrase-based encrypted export/import for portability.
- Offer a “lock” behavior:
  - auto-lock after inactivity,
  - require passphrase to reveal/export private key material.

Token-at-rest note:

- Saved JWTs may contain sensitive claims. If you store tokens, store them with the same protections as keys (OS keychain by default).

Operational warnings:

- Localhost binding does not protect against malware on the same machine.
- If `--allow-remote` exists, assume it will be used incorrectly; document it as dangerous and consider removing it entirely.
