# Command specification

This file defines **what the CLI does**, independent of implementation language.

## Common notation

- `TOKEN`: a JWT string (3 dot-separated segments)
- `-`: read from stdin
- `@path`: read from a file (convention; see `input.md`)

## `jwt-tester decode`

Purpose: parse a JWT and print header + payload.

Key rules:

- `decode` does **not** claim authenticity unless a key is provided.
- It should succeed even if signature is invalid (because it is not verifying).
  - If a key is provided, invalid signature/claims should return an error.

Suggested interface:

```
jwt-tester decode [--json] [--date[=<UTC|local|+HH:MM>]] [--out <PATH>] <TOKEN|->
  [--alg <ALG>] (--secret <S>|--key <K>|--jwks <JWKS>|--project <PROJECT>)
  [--key-format <pem|der>]
  [--kid <KID>] [--allow-single-jwk]
  [--key-id <UUID> | --key-name <NAME>]
  [--try-all-keys]
  [--iss <ISS>] [--sub <SUB>] [--aud <AUD>]
  [--leeway-secs <N>] [--ignore-exp]
  [--require <claim> ...]
  [--explain]
```

Outputs:

- text: labeled “UNVERIFIED” unless verification succeeds (then “VERIFIED”)
- json: `{ ok, data: { header, payload, dates, verified?, verification? } }`

Exit codes:

- `0`: parsed successfully
- non-zero: malformed token / base64 decode failure / JSON parse failure

## `jwt-tester verify`

Purpose: verify signature and validate claims.

Suggested interface:

```
jwt-tester verify [--json] [--alg <ALG>] (--secret <S>|--key <K>|--jwks <JWKS>|--project <PROJECT>) <TOKEN|->
  [--key-format <pem|der>]
  [--kid <KID>] [--allow-single-jwk]
  [--key-id <UUID> | --key-name <NAME>]
  [--try-all-keys]
  [--iss <ISS>] [--sub <SUB>] [--aud <AUD>]
  [--leeway-secs <N>] [--ignore-exp]
  [--require <claim> ...]
  [--explain]
```

Key rules:

- If `--alg` is omitted, the tool infers it from the JWT header.
- The tool must treat the JWT header as untrusted input.
- The tool must clearly differentiate:
  - signature validity,
  - claim validation failures (exp/nbf/iss/aud).
- If `--project` is provided and `--secret/--key/--jwks` is not, the tool resolves key material from the local vault (see `vault.md`).
- If the token header contains `kid`, the vault resolver selects a key with a matching stored `kid` before falling back to defaults.

MVP implemented in `jwt-tester-app/` today:

```
jwt-tester verify [--alg <hs256|hs384|hs512|rs256|rs384|rs512|ps256|ps384|ps512|es256|es384|eddsa>] <TOKEN|->
  (--secret <S> | --key <K> | --jwks <JWKS> | --project <PROJECT>)
  [--key-format <pem|der>]
  [--kid <KID>] [--allow-single-jwk]
  [--key-id <UUID> | --key-name <NAME>]
  [--try-all-keys]
  [--iss <ISS>] [--sub <SUB>] [--aud <AUD> ...]
  [--ignore-exp] [--leeway-secs <N>]
  [--require <CLAIM> ...]
  [--explain]
```

Current MVP deferrals:

- remote JWKS URLs / OIDC discovery / caching
- policy files for verification (`--policy`)

Exit codes (recommended stable contract):

- `0`: signature valid and all validations pass
- `10`: token malformed
- `11`: signature invalid / key mismatch
- `12`: claims invalid (expired, nbf in future, issuer mismatch, etc.)
- `13`: key input invalid (bad PEM/DER/JWKS, missing `kid`, etc.)
- `14`: internal error

## `jwt-tester encode`

Purpose: create and sign a JWT from claims.

Suggested interface:

```
jwt-tester encode --alg <ALG> (--secret <S>|--key <K>) [<CLAIMS_JSON|-|@file.json>]
  [--header <HEADER_JSON|-|@file.json>]
  [--kid <KID>]
  [--typ <TYP>] [--no-typ]
  [--iss <ISS>] [--sub <SUB>] [--aud <AUD>] [--jti <JTI>]
  [--iat[=<TIME>]] [--no-iat]
  [--nbf <TIME>] [--exp <TIME>]
  [--claim <k=v> ...]
  [--keep-payload-order]
  [--out <PATH>]
  [--project <PROJECT>]
  [--key-id <UUID>]
```

Rules:

- Claim merges are deterministic (see `input.md`).
- By default payload keys are sorted; `--keep-payload-order` preserves input order.
- `--exp` with no value defaults to `+30m`.
- If both `--exp` and `--no-exp` exist in your design, document precedence.
- If `--project` is provided and `--secret/--key` is not, the tool resolves signing key material from the local vault (see `vault.md`).

MVP implemented in `jwt-tester-app/` today:

```
jwt-tester encode --alg <hs256|hs384|hs512|rs256|rs384|rs512|ps256|ps384|ps512|es256|es384|eddsa>
  (--secret <S> | --key <K> | --project <PROJECT>)
  [--key-format <pem|der>]
  [<CLAIMS_JSON|-|@file.json>]
  [--header <HEADER_JSON|-|@file.json>]
  [--kid <KID>] [--typ <TYP>] [--no-typ]
  [--iss <ISS>] [--sub <SUB>] [--aud <AUD> ...] [--jti <JTI>]
  [--iat[=<TIME>]] [--no-iat]
  [--nbf <TIME>] [--exp <TIME>]
  [--claim <k=v> ...]
  [--claim-file <PATH> ...]
  [--keep-payload-order]
  [--out <PATH>]
  [--key-id <UUID> | --key-name <NAME>]
```

Current MVP deferrals:

- custom (non-standard) JWT header fields beyond the standard header keys

Exit codes:

- `0`: token created
- non-zero: invalid claims/header JSON, invalid key, or signing error

## `jwt-tester inspect` (recommended)

Purpose: human-friendly summary (especially for timestamps, alg/kid, token size).

Suggested interface:

```
jwt-tester inspect <TOKEN|->
  [--date[=<UTC|local|+HH:MM>]]
  [--show-segments]
```

## `jwt-tester split` (recommended)

Purpose: output segments (header/payload/signature) individually, base64url-decoded.

Suggested interface:

```
jwt-tester split <TOKEN|->
  [--format json|text]
```

This is useful for scripts and debugging.

## `jwt-tester completion`

```
jwt-tester completion <bash|zsh|fish|powershell|elvish|nushell>
```

Nushell completion is supported alongside bash/zsh/fish/powershell/elvish.

## `jwt-tester ui` (recommended)

Purpose: start a **local-only** web interface on localhost to:

- manage keys/secrets locally (“vault”),
- build/inspect/verify tokens interactively,
- export/import vault data intentionally.

Suggested interface:

```
jwt-tester ui
  [--host 127.0.0.1]
  [--port <0|PORT>]
  [--open]
  [--data-dir <PATH>]
  [--no-persist]
  [--lock-after <DURATION>]
  [--require-passphrase]
  [--allow-remote]   # strongly discouraged; see ui.md
```

MVP notes:

- UI covers vault CRUD plus token builder/inspector/verify.
- Vault export/import is available via CLI and UI.
- Flags `--open`, `--lock-after`, and `--require-passphrase` are deferred.

Rules:

- Default bind is `127.0.0.1` (not LAN).
- Default port can be ephemeral (`--port 0`) to avoid collisions.
- The UI must never exfiltrate keys; it should not fetch remote assets by default.

Output:

- prints the local URL to stdout (e.g. `http://127.0.0.1:18455/`)

Exit codes:

- `0`: server started and shut down cleanly
- non-zero: bind failure / storage error / migration failure

## `jwt-tester vault` (recommended)

Purpose: manage vault entries from CLI (useful for headless usage and for LLM-driven workflows).

Suggested interface:

```
jwt-tester vault project add <NAME> [--description <TEXT>] [--tag <TAG> ...]
jwt-tester vault project list
jwt-tester vault project set-default-key --project <NAME> (--key-id <UUID> | --key-name <NAME> | --clear)
jwt-tester vault key add --project <NAME> [--name <KEY_NAME>] [--kid <KID>] [--description <TEXT>] [--tag <TAG> ...] --kind hmac --secret <SECRET|-|@file>
jwt-tester vault key list --project <NAME>
jwt-tester vault token add --project <NAME> --name <TOKEN_NAME> --token <TOKEN|-|@file>
jwt-tester vault token list --project <NAME>
jwt-tester vault export --passphrase <PASS> [--out <PATH>]
jwt-tester vault import --bundle <BUNDLE|-|@file> --passphrase <PASS> [--replace]
```


