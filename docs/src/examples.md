# Usage Examples

This guide covers common workflows for `jwt-tester`.

## 1. Quick Encode & Verify (No Vault)

Useful for one-off tasks where you have the secret in an env var.

**Encode a token:**
```bash
# Create an HS256 token with a subject and 1 hour expiry
export JWT_SECRET="my-super-secret-key"
jwt-tester encode --alg hs256 --secret env:JWT_SECRET \
  --sub "user-123" \
  --exp "+1h" \
  --claim "role=admin"
```

**Verify a token:**
```bash
# Verify the token from stdin
echo "eyJ..." | jwt-tester verify --secret env:JWT_SECRET
```

## 2. Working with RSA Keys

**Encode with a private key file:**
```bash
jwt-tester encode --alg rs256 --key @private_key.pem \
  --sub "app-service" \
  --out token.jwt
```

**Verify with a public key file:**
```bash
jwt-tester verify --alg rs256 --key @public_key.pem token.jwt
```

## 3. Project Workflow (Using the Vault)

Avoid pasting secrets repeatedly by setting up a project.

**Setup:**
```bash
# 1. Create a project
jwt-tester vault project add "backend-api"

# 2. Add the HMAC secret to the vault
jwt-tester vault key add --project "backend-api" \
  --name "staging-key" \
  --kind hmac \
  --secret env:JWT_SECRET
  
# 3. Set it as default (optional)
jwt-tester vault project set-default-key --project "backend-api" --key-name "staging-key"
```

**Daily Usage:**
```bash
# Encode (uses default key for "backend-api")
jwt-tester encode --project "backend-api" --sub "me"

# Verify (auto-selects key)
jwt-tester verify --project "backend-api" < token.jwt
```

## 4. Key Rotation

Handle multiple keys (e.g., current and next) gracefully.

```bash
# Add a new key with a specific Key ID (kid)
jwt-tester vault key add --project "backend-api" \
  --name "2024-key" \
  --kind hmac \
  --kid "key-2024-v1" \
  --secret "new-secret-value"
```

When verifying:
*   If the token header has `"kid": "key-2024-v1"`, `jwt-tester` will automatically pick this new key.
*   If the token has no `kid` or an old `kid`, it falls back to the matching key or default.

## 5. Inspection & Debugging

**Inspect a token (no key needed):**
```bash
jwt-tester inspect token.jwt
```

**Check expiration in local time:**
```bash
jwt-tester inspect token.jwt --date local
```

**Split a token for scripting:**
```bash
# Get just the payload JSON
jwt-tester split token.jwt --format json | jq .payload
```

## 6. Exporting the Vault

Backup your configuration and secrets.

```bash
# Export to an encrypted JSON file
jwt-tester vault export --passphrase "my-backup-password" --out backup.json

# Import on another machine
jwt-tester vault import --bundle @backup.json --passphrase "my-backup-password"
```
