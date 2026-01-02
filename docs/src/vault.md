# Vault model: projects, keys, tokens

The “vault” is the feature that prevents you from having to pass secrets/keys every time.

It supports:

- grouping by **project**,
- storing **key material** (secrets/private keys/JWKS) locally,
- optionally storing **sample JWTs** locally,
- resolving the “right key” from `project` (and optionally `kid`).

This is especially useful when driving `jwt-tester` from LLM-assisted workflows: the prompt/tool call can include `project` and the tool can find the correct key without the user pasting it again.

Note: you can later evolve `project` into `domain + project` (or add tags/environments) without changing the core idea; keep IDs stable and add the extra column(s) when you need them.

## CLI surface (current)

- `project`: add, list, delete, set-default-key
- `key`: add, generate, list, delete
- `token`: add, list, delete
- `export` / `import`

Secret/token/passphrase inputs accept `prompt[:LABEL]`, `-`, `@file`, and `env:NAME` (see `input.md`).

### Key generation (current)

`jwt-tester vault key generate` can create and store:

- HMAC secrets (`--hmac-bytes <N>`)
- RSA keys (`--rsa-bits 2048|3072|4096`)
- EC keys (`--ec-curve P-256|P-384`)

Use `--reveal` to print generated material and `--out <PATH>` to write it to a file.

## Entities

### Project

A **project** is the primary namespace for stored artifacts.

Fields (recommended):

- `id` (uuid)
- `name` (string)
- `created_at`
- optional `description` and `tags` (stored and surfaced by CLI/UI)

Uniqueness rule (recommended):

- enforce `name` unique (so lookups are deterministic).

### Key

A **key** is a stored secret or keypair (or a JWKS document).

Key metadata is stored in the DB; secret bytes should be stored in OS keychain by default.

Fields (recommended):

- `id` (uuid)
- `project_id`
- `name` (optional display name; if omitted, the tool should auto-generate one like `key-<id8>`)
- `kind`: `hmac` | `rsa` | `ec` | `eddsa` | `jwks`
  - `rsa` covers RS* and PS* algorithms; `ec` covers ES256/ES384.
- `kid` (optional; used for kid-based selection)
- `alg_hint` (optional)
- `created_at`
- `storage_ref` (keychain service + account)

### Token (optional)

A **token** is a stored JWT string (usually sample tokens).

Tokens can contain personal data; treat them as sensitive:

- store token bytes in OS keychain by default (like keys),
- store only metadata in DB.

Fields (recommended):

- `id` (uuid)
- `project_id`
- `name`
- `created_at`
- `storage_ref` (keychain service + account)

## Resolution: how project picks a key

### Primary lookup

When a command accepts `--project`, it resolves a project record and then selects a key.

You need a deterministic selection policy. Recommended options:

1. Explicit key id:
   - `--key-id <uuid>` always wins.
2. Explicit key name:
   - `--key-name <NAME>` always wins.
3. `kid`-based selection (implemented):
   - if verifying and the token header has `kid`, select the key whose stored `kid` matches.
4. Default key per project:
   - store `default_key_id` on the project and use it.
5. MVP rule (simple and deterministic):
   - if the project has exactly **one** key, use it; otherwise require `--key-id` / `--key-name` or configure a default key.
6. If ambiguous:
   - error with a list of candidate keys.

### Example CLI UX

```
# Verify using stored defaults (no secret on command line)
jwt-tester verify --project api <TOKEN>

# Verify by explicit key id
jwt-tester verify --key-id 2c7b... <TOKEN>

# Set default key for a project (CLI or UI)
jwt-tester vault project set-default-key --project api --key-id 2c7b...
```

In the current Rust scaffold, `set-default-key` is implemented via:

- `jwt-tester vault project set-default-key --project <NAME> --key-id <UUID>`

### Default key behavior

Recommended UX:

- `jwt-tester vault project set-default-key --project api --key-id <UUID>`
- If a default key is configured:
  - `jwt-tester verify --project api ...` uses it automatically.
  - `jwt-tester encode --project api ...` uses it automatically.

If the default key fails verification:

- do **not** silently fall back to other keys unless the user explicitly opts in (see below).

### Multiple keys: should we try all if default fails?

Recommendation:

- **No, not by default.**

Reasons:

- It can hide configuration mistakes (wrong project, wrong key) and make debugging harder.
- It changes the meaning of “signature invalid”: with try-all, “invalid” becomes “no key in this project matched”.
- It can be slow on projects with many keys, and it can create confusing output (“it worked sometimes”).

If you need it (key rotations, unknown signing key), expose it explicitly:

- `jwt-tester verify --project api --try-all-keys ...`

Behavior when `--try-all-keys` is set:

- Attempt the default key first (if configured), then try remaining keys.
- Only treat **InvalidSignature** as “try next”; for other errors (expired, claim failures, malformed token), stop and return that error.

## Storage strategy (recommended)

Default:

- metadata: SQLite (in app data dir)
- secret bytes: OS keychain (one entry per key/token)

Optional:

- passphrase-encrypted export/import bundle for portability

Implemented:

- `jwt-tester vault export --passphrase ...` creates an encrypted bundle.
- `jwt-tester vault import --bundle ... --passphrase ...` restores it (optionally `--replace`).
