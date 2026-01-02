# Localhost UI mode (`jwt-tester ui`)

This document specifies an **optional** web UI that runs on the user’s machine and helps them work with JWTs interactively.

The intent is:

- improve UX for “token workshop” tasks,
- store keys locally in a user-controlled vault,
- keep the security model conservative by default.

## High-level behavior

Command:

```
jwt-tester ui
```

Default behavior:

- starts an HTTP server bound to localhost only (`127.0.0.1`, and optionally `::1`)
- serves a small web app at `/`
- prints the URL (and optionally opens a browser tab)
- persists data locally (unless `--no-persist`)

## MVP scope (current)

- UI covers vault CRUD (projects, keys, tokens), including:
  - project description/tags
  - key kid/description/tags
  - default key selection per project
- Token builder, inspector, and verify screens are implemented.
- Vault export/import (passphrase-encrypted bundle) is implemented.
- `--open`, `--lock-after`, and `--require-passphrase` are deferred.

## CLI flags (recommended)

```
jwt-tester ui
  --host 127.0.0.1        # default
  --port 0                # default (ephemeral port)
  --open                  # deferred
  --data-dir <PATH>       # where vault DB/files live
  --no-persist            # ephemeral session only
  --lock-after <DURATION> # deferred
  --require-passphrase    # deferred
  --allow-remote          # dangerous; requires explicit opt-in + UI warning
```

If you offer `--allow-remote`, treat it as a footgun:

- require `--allow-remote` AND a non-localhost host value
- print a prominent warning to stderr
- show a warning banner in the UI

## UX: core screens and flows

### 1) Vault (Keys)

Capabilities:

- manage **projects**:
  - create project: `project` (name)
  - select active project to filter keys/tokens
- add a key/secret:
  - paste PEM/DER
  - import `@file`
  - import JWKS
  - generate a key (optional: RSA/EC/Ed25519; HMAC random bytes)
- tag and search keys
- store metadata (non-secret):
  - name, description, created_at, algorithm family, key type, `kid`, thumbprint
- “reveal secret”:
  - require unlock/passphrase
  - show for a short time, copy-to-clipboard action
- export:
  - export encrypted vault bundle (default)
  - allow plain export only with explicit confirmation

Data model (suggested):

- `Project`:
  - `id` (uuid)
  - `name` (string)
  - `created_at`
- `KeyEntry`:
  - `id` (uuid)
  - `project_id` (uuid)
  - `name`
  - `kind`: `hmac` | `rsa` | `ec` | `eddsa` | `jwks`
  - `kid` (optional)
  - `alg_hint` (optional)
  - `created_at`
  - `storage_ref` (e.g. keychain `service` + `account`)
  - `blob_format` (optional): `pem` | `der` | `jwks-json`

### 2) Token builder (Encode)

Capabilities:

- choose signing algorithm + key entry
- edit claims JSON with validation
- quick claim helpers:
  - `iss/sub/aud/jti`
  - `iat/nbf/exp` with duration shortcuts (`+30m`, `2h`, “tomorrow” if you implement it)
- produce token
- copy token to clipboard
- optionally save a “preset” (template) for repeated use
- optionally save generated tokens into the vault under the active project (as “samples”)

### 3) Token inspector (Decode/Inspect)

Capabilities:

- paste token (or upload file)
- show:
  - header JSON
  - payload JSON
  - signature bytes (base64url)
- timestamp rendering toggles:
  - UTC / local / fixed offset
- warnings:
  - “UNVERIFIED” by default until a verification step succeeds

### 4) Verification (Verify)

Capabilities:

- choose a key/JWKS from vault
- set verification policy:
  - allowed algorithms (explicit)
  - issuer/audience/subject requirements
  - leeway
  - exp required vs optional
- run verify and show:
  - signature: pass/fail
  - claim checks: pass/fail with details
  - “explain” view: which validations ran

## Storage and encryption

### Data directory

Default location should follow OS conventions, e.g.:

- Windows: `%APPDATA%\\<app>\\`
- macOS: `~/Library/Application Support/<app>/`
- Linux: `~/.local/share/<app>/`

Allow override via `--data-dir`.

### Persistence modes

- persistent (default): encrypted vault on disk
- ephemeral (`--no-persist`): keep everything in memory; nothing written to disk

### Encryption strategy (recommended)

Best default (recommended): **store secret material in the OS keychain**, and keep only metadata in a local DB.

Why this is “better”:

- Strong baseline security without asking users to manage a passphrase.
- Uses platform protections the user already relies on.
- Avoids implementing your own at-rest crypto for the first iteration.

Suggested design:

1. **Metadata DB (local)**:
   - stores: `projects`, `keys`, `tokens` metadata
   - does **not** store private key/secret bytes
2. **OS keychain entry per key**:
   - stores the raw secret/key material (PEM/DER/JWKS JSON) for that `id`
   - uses a stable `service` name (see “Naming” below) and an `account` key like `key:<uuid>`
3. **OS keychain entry per token (optional)**:
   - stores token strings for saved samples under an `account` like `token:<uuid>`

Optional (later): passphrase export/import

- Export an encrypted bundle (user-provided passphrase) so vaults can be moved between machines.
- Keep OS keychain as the day-to-day default, and use passphrase only for portability.

Vault lock/unlock:

- lock at startup (optional) or unlock automatically if OS keychain is used
- auto-lock after inactivity (`--lock-after`)
- lock on “browser tab closed” is not reliable; prefer server-side timers

Import/export:

- default: export an **encrypted** bundle (passphrase-based) that does not depend on OS keychain
- require explicit UI confirmation for any plain export of key material

### Naming (important)

Keychain storage depends on a stable identifier:

- `service` name should not change casually, or the tool won’t be able to find stored secrets.
- Prefer a reverse-domain style string (e.g. `com.yourorg.<app>`) rather than the binary name alone.

## Local HTTP server

### Binding and startup

- bind to `127.0.0.1` by default
- use an ephemeral port by default (`--port 0`) and print the final URL

### Routes (suggested)

- `GET /` → UI app shell
- `GET /assets/*` → static assets
- `GET /api/health` → returns `{ ok: true }`
- `POST /api/decode`
- `POST /api/verify`
- `POST /api/encode`
- `GET /api/vault/keys` / `POST /api/vault/keys` / `DELETE /api/vault/keys/:id`
- `GET /api/vault/projects` / `POST /api/vault/projects` / `DELETE /api/vault/projects/:id`
- `GET /api/vault/tokens` / `POST /api/vault/tokens` / `DELETE /api/vault/tokens/:id`
- `POST /api/vault/lock` / `POST /api/vault/unlock`
- `POST /api/vault/export` / `POST /api/vault/import`

### Security headers (minimum)

- `Content-Security-Policy` (restrict to self)
- `X-Frame-Options: DENY` (or CSP `frame-ancestors 'none'`)
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`

### CSRF and origin

Because this runs in a browser, assume:

- other local web pages could attempt to send requests to your localhost server

Protections:

- validate `Origin` and `Host`
- require a CSRF token for state-changing requests
- disable CORS by default

## How this makes the tool better

Compared to a pure CLI:

- easier for non-experts to avoid insecure verification patterns
- reusable key vault instead of repeatedly pasting secrets
- repeatable presets/templates for common claim sets
- “explain verification” view makes debugging auth issues faster

Compared to the reference CLI:

- explicit separation of decode vs verify in the UI (no “false sense of validity”)
- safer defaults (localhost-only, no remote assets, vault encryption)
