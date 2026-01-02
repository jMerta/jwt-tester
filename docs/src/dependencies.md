# Dependency Map

Key Rust crates used in `jwt-tester` and their purpose.

## Core Logic

- **`jsonwebtoken`**: The heavy lifter. Handles the implementation of JWT encoding, decoding, and verification logic, including crypto primitives.
- **`serde` / `serde_json`**: Serialization and deserialization of JSON structures (headers, claims, vault metadata).
- **`base64`**: Handling Base64URL encoding/decoding for JWT segments.
- **`time` / `humantime`**: Date parsing and formatting (RFC3339) and duration parsing (e.g., "30m").

## CLI & Input

- **`clap`**: Command-line argument parsing. We use the `derive` feature for type-safe argument structs.
- **`rpassword`**: Securely reading passwords/passphrases from stdin (for vault export/import).

## Data & Vault

- **`rusqlite`**: Embedded SQLite database for storing vault metadata (projects, keys, tokens).
- **`keyring`**: Interface to the OS Keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service). Used to store the actual secrets safely.
- **`uuid`**: Generating unique IDs for vault entities.
- **`directories`**: Finding standard system data directories for the vault DB.

## Web UI

- **`axum`**: High-performance async web framework for the UI API.
- **`tokio`**: Async runtime powering the HTTP server and file I/O.
- **`tracing`**: Structured logging.
- **`rand`**: Generating CSRF tokens.

## Build

- **`npm` (external)**: Used via `std::process::Command` in `build.rs` or runtime to build the frontend assets.
