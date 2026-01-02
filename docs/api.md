# API Reference (UI Backend)

The `jwt-tester ui` command starts a local HTTP server that provides a REST API for the frontend. This API allows the UI to interact with the vault and perform JWT operations.

**Base URL:** `http://127.0.0.1:<PORT>/api`
**Content-Type:** `application/json`

## Security

- **Localhost Only:** By default, the API binds only to `127.0.0.1`.
- **CSRF Protection:** All state-changing requests (`POST`, `DELETE`) require a `X-CSRF-Token` header.
  - The token can be obtained via `GET /api/csrf`.
- **CORS:** Disabled by default. `Origin` header is checked to block non-local requests.

---

## Core Endpoints

### Health Check
**GET** `/health`
Returns 200 OK if the server is running.
```json
{
  "ok": true
}
```

### CSRF Token
**GET** `/csrf`
Returns the CSRF token required for subsequent POST/DELETE requests.
```json
{
  "ok": true,
  "csrf": "base64_encoded_token"
}
```

---

## JWT Operations

### Encode Token
**POST** `/jwt/encode`
Creates a signed JWT.

**Request:**
```json
{
  "project": "project_name",
  "alg": "HS256",
  "key_id": "optional_uuid",
  "claims": "{\"sub\":\"123\"}",
  "iss": "issuer",
  "exp": "+30m"
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "token": "eyJ...",
    "key_source": "key-name"
  }
}
```

### Verify Token
**POST** `/jwt/verify`
Verifies a token's signature and claims.

**Request:**
```json
{
  "project": "project_name",
  "token": "eyJ...",
  "alg": "HS256",
  "try_all_keys": false,
  "ignore_exp": false
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "valid": true,
    "claims": { ... },
    "explain": { ... } // If explain=true
  }
}
```

### Inspect Token
**POST** `/jwt/inspect`
Decodes a token without verifying (for inspection).

**Request:**
```json
{
  "token": "eyJ...",
  "date": "utc", // utc, local, or +HH:MM
  "show_segments": true
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "header": { ... },
    "payload": { ... },
    "summary": { "alg": "HS256", "kid": "..." },
    "dates": { "exp": "2024-01-01T..." },
    "segments": ["header_b64", "payload_b64", "sig_b64"]
  }
}
```

---

## Vault Management

### Projects

- **GET** `/vault/projects`: List all projects.
- **POST** `/vault/projects`: Create a project.
  - Body: `{ "name": "prod", "description": "...", "tags": ["a"] }`
- **DELETE** `/vault/projects/:id`: Delete a project.
- **POST** `/vault/projects/:id/default-key`: Set default key.
  - Body: `{ "key_id": "uuid" }`

### Keys

- **GET** `/vault/keys?project_id=...`: List keys (optionally filtered).
- **POST** `/vault/keys`: Import an existing key.
  - Body: `{ "project_id": "...", "name": "my-key", "kind": "hmac", "secret": "..." }`
- **POST** `/vault/keys/generate`: Generate a new key.
  - Body: `{ "project_id": "...", "kind": "rsa", "rsa_bits": 2048 }`
- **DELETE** `/vault/keys/:id`: Delete a key.

### Tokens (Samples)

- **GET** `/vault/tokens?project_id=...`: List saved tokens.
- **POST** `/vault/tokens`: Save a token.
  - Body: `{ "project_id": "...", "name": "sample", "token": "..." }`
- **DELETE** `/vault/tokens/:id`: Delete a token.

### Import / Export

- **POST** `/vault/export`: Get encrypted vault dump.
  - Body: `{ "passphrase": "..." }`
- **POST** `/vault/import`: Restore vault from dump.
  - Body: `{ "bundle": "{...}", "passphrase": "...", "replace": true }`
