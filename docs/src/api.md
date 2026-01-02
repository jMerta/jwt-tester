# API Reference (UI Backend)

The `jwt-tester ui` command starts a local HTTP server that provides a REST API for the frontend. This API allows the UI to interact with the vault and perform JWT operations.

**Base URL:** `http://127.0.0.1:<PORT>`
**Content-Type:** `application/json`

## Security
- **Localhost only by default:** The server binds to `127.0.0.1` unless `--allow-remote` is used.
- **CSRF protection:** All `POST`/`DELETE` requests require the `x-csrf-token` header.
  - Obtain a token via `GET /api/csrf`.
- **Origin checks:** Non-GET requests with an `Origin` not starting with `http://127.0.0.1` or `http://localhost` are rejected.
- **CORS:** Disabled (no cross-origin access by default).

## Response envelope
Successful responses:
```json
{ "ok": true, "data": { } }
```
Errors:
```json
{ "ok": false, "error": "message", "code": "OPTIONAL_CODE" }
```

---

## Core Endpoints
### Health Check
**GET** `/api/health`
```json
{ "ok": true }
```

### CSRF Token
**GET** `/api/csrf`
```json
{ "ok": true, "csrf": "base64url_token" }
```

---

## JWT Operations (all require `x-csrf-token`)

### Encode Token
**POST** `/api/jwt/encode`

**Request (fields mirror CLI encode flags):**
```json
{
  "project": "project_name",
  "alg": "hs256",
  "key_id": "optional_uuid",
  "key_name": "optional_name",
  "claims": "{\"sub\":\"123\"}",
  "kid": "optional_kid",
  "typ": "JWT",
  "no_typ": false,
  "iss": "issuer",
  "sub": "subject",
  "aud": ["aud1"],
  "jti": "id",
  "iat": "now",
  "no_iat": false,
  "nbf": "+5m",
  "exp": "+30m"
}
```

**Response:**
```json
{
  "ok": true,
  "data": { "token": "eyJ...", "key_source": "project:backend-api" }
}
```

### Verify Token
**POST** `/api/jwt/verify`

**Request:**
```json
{
  "project": "project_name",
  "token": "eyJ...",
  "alg": "auto",
  "key_id": "optional_uuid",
  "key_name": "optional_name",
  "try_all_keys": false,
  "ignore_exp": false,
  "leeway_secs": 30,
  "iss": "issuer",
  "sub": "subject",
  "aud": ["aud1"],
  "require": ["exp"],
  "explain": true
}
```

**Response:**
```json
{
  "ok": true,
  "data": { "valid": true, "claims": { }, "explain": { } }
}
```

### Inspect Token
**POST** `/api/jwt/inspect`

**Request:**
```json
{ "token": "eyJ...", "date": "utc", "show_segments": true }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "header": { },
    "payload": { },
    "summary": { "alg": "HS256", "kid": null, "typ": "JWT", "sizes": { } },
    "dates": { },
    "segments": ["header_b64", "payload_b64", "sig_b64"]
  }
}
```

---

## Vault Management (all `POST`/`DELETE` require `x-csrf-token`)

### Projects
- **GET** `/api/vault/projects`
- **POST** `/api/vault/projects`
  - Body: `{ "name": "prod", "description": "...", "tags": ["a"] }`
- **POST** `/api/vault/projects/:id/default-key`
  - Body: `{ "key_id": "uuid" }` (omit or set `null` to clear)
- **DELETE** `/api/vault/projects/:id`

### Keys
- **GET** `/api/vault/keys?project_id=...`
- **POST** `/api/vault/keys`
  - Body: `{ "project_id": "...", "name": "my-key", "kind": "hmac", "secret": "...", "kid": "...", "description": "...", "tags": ["a"] }`
- **POST** `/api/vault/keys/generate`
  - Body: `{ "project_id": "...", "name": "key", "kind": "rsa", "rsa_bits": 2048 }`
  - Response includes the generated material: `{ "ok": true, "data": { "key": { ... }, "material": "...", "format": "pem" } }`
- **DELETE** `/api/vault/keys/:id`

### Tokens (Samples)
- **GET** `/api/vault/tokens?project_id=...`
- **POST** `/api/vault/tokens`
  - Body: `{ "project_id": "...", "name": "sample", "token": "..." }`
- **POST** `/api/vault/tokens/:id/material`
  - Response: `{ "ok": true, "data": { "token": "..." } }`
- **DELETE** `/api/vault/tokens/:id`

### Import / Export
- **POST** `/api/vault/export`
  - Body: `{ "passphrase": "..." }`
  - Response: `{ "ok": true, "data": { "bundle": "{...}" } }`
- **POST** `/api/vault/import`
  - Body: `{ "bundle": "{...}", "passphrase": "...", "replace": true }`
