# JWT CLI tool — documentation index

This documentation set is a product + engineering spec for building a JWT CLI tool that:

- encodes (signs) tokens,
- decodes tokens safely,
- verifies tokens with sane defaults,
- supports modern key formats (PEM/DER/JWKS),
- behaves well in shell pipelines (stdin/stdout, exit codes, JSON output),
- avoids common JWT security footguns.

## Reading order

1.  **Vision & Goals**: `vision.md` — goals, non-goals, and “better than the reference CLI”
2.  **Getting Started**:
    *   `setup.md` — Installation, building, and deployment (Docker).
    *   `examples.md` — Common workflows and CLI recipes.
3.  **Core Concepts**:
    *   `input.md` — How tokens/claims/keys are provided.
    *   `output.md` — Output formats + exit codes.
    *   `jwt-and-claims.md` — JWT structure + claim handling rules.
    *   `vault.md` — Project grouping and secret resolution.
4.  **Reference**:
    *   `cli.md` — CLI UX conventions.
    *   `commands.md` — Command-by-command specification.
    *   `api.md` — HTTP API reference for the UI.
5.  **Architecture & Internals**:
    *   `architecture.md` — Internal modules and design.
    *   `diagrams.md` — Visual diagrams of system context and flows.
    *   `dependencies.md` — Key libraries and crates.
    *   `testing.md` — Test strategy + fixtures.
    *   `security.md` — Safe defaults + threat modeling notes.
6.  **Future**:
    *   `roadmap.md` — Optional “next level” features.

## Terminology

- **JWT**: `base64url(header).base64url(payload).base64url(signature)`
- **JWS**: signed JWT (what most people mean by “JWT”)
- **Claims**: key/value JSON payload inside the JWT
- **Secret**: symmetric key for HMAC (HS256/384/512)
- **Private key**: asymmetric signing key (RSA/ECDSA/EdDSA)
- **JWKS**: JSON Web Key Set (a JSON document containing keys)
- **Vault**: Local storage for managing project configurations and keys (metadata in SQLite, secrets in OS Keychain).
