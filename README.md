# jwt-tester

jwt-tester is a local-first JWT CLI plus a localhost-only UI. It supports both direct key input (for one-off work) and a vault (for reuse without retyping secrets). The vault stores only metadata in SQLite; secret material and saved JWT strings live in the OS keychain.

This repo contains the production Rust implementation in `jwt-tester-app/` plus detailed design and spec notes under `docs/src/`.

## CLI Demo

Inline demo (plays on GitHub):

<video src="https://github.com/user-attachments/assets/23a5e481-32a2-41e4-ac1d-f81bb3bd6028" controls muted playsinline></video>

## Documentation

Full documentation is available in the `docs/src/` directory:

- [**Setup & Installation**](docs/src/setup.md): Build from source or use Docker.
- [**Usage Examples**](docs/src/examples.md): Common CLI workflows.
- [**Command Reference**](docs/src/commands.md): Detailed CLI command specs.
- [**Vault Guide**](docs/src/vault.md): How to use projects and stored keys.
- [**UI API Reference**](docs/src/api.md): REST API docs for the web interface.
- [**Architecture**](docs/src/architecture.md) & [**Diagrams**](docs/src/diagrams.md): System design.

## Features (MVP)
- Algorithms: HS256/384/512, RS256/384/512, PS256/384/512, ES256/384, EdDSA
- Direct or vault key input
- Commands: encode, verify, decode (unverified unless key provided), inspect, split
- Vault with project grouping, optional default key per project
- Local UI (localhost only by default) for vault CRUD + token builder/inspect/verify
- Vault export/import (passphrase-encrypted bundle)
- JSON output mode and stable exit codes

## Quick Start

Install via npm:

```bash
npm install -g jwt-tester-tool
jwt-tester --help
```

Docker (GHCR):

```bash
docker pull ghcr.io/jmerta/jwt-tester:latest
docker run --rm -p 3000:3000 -v $(pwd)/data:/data \
  -e JWT_TESTER_KEYCHAIN_PASSPHRASE="change-me" \
  ghcr.io/jmerta/jwt-tester:latest
```

From the repo root:

```powershell
cd jwt-tester-app
cargo build --release
./target/release/jwt-tester --help
```

See [docs/src/setup.md](docs/src/setup.md) for detailed build instructions including Docker and cross-compilation.

## License

MIT

