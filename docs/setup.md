# Setup and Deployment

## Local Development Setup

To build `jwt-tester` from source, you need a standard Rust environment and Node.js for the UI.

### Prerequisites

1.  **Rust**: Install via [rustup.rs](https://rustup.rs).
2.  **Node.js**: Install Node.js 18+ and npm (for building the UI).
3.  **OS Dependencies**:
    *   **Linux**: `libsecret-1-dev`, `pkg-config` (required for `keyring` crate).
        ```bash
        sudo apt-get install pkg-config libsecret-1-dev
        ```
    *   **macOS/Windows**: Native keychain support is built-in.

### Building

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-org/jwt-tester
    cd jwt-tester/jwt-tester-app
    ```

2.  **Build the project**:
    This will automatically build the UI assets (via `build.rs` triggering npm) and then compile the Rust binary.
    ```bash
    cargo build --release
    ```

3.  **Run**:
    ```bash
    ./target/release/jwt-tester --help
    ```

### CLI-Only Build

If you don't need the web UI or don't have Node.js installed, you can build the CLI-only version:

```bash
cargo build --release --no-default-features --features cli-only
```

## Docker Deployment

`jwt-tester` can run in a Docker container. This is useful for:
*   CI/CD pipelines.
*   Environments without local Rust/Node toolchains.
*   Isolated testing.

**Note:** The Docker image uses a file-based keychain backend because the container doesn't have access to the host OS keychain.

### Running the Image

```bash
# Run the UI on port 3000
docker run -p 3000:3000 -v $(pwd)/data:/data jwt-tester
```

### Building the Image

```bash
docker build -t jwt-tester -f jwt-tester-app/Dockerfile .
```

The Dockerfile uses a multi-stage build:
1.  `ui-builder`: Node.js image to build frontend assets.
2.  `builder`: Rust image to compile the binary.
3.  `final`: Slim Debian image with the binary and assets.

## Cross-Compilation

To cross-compile for other platforms (e.g., from Linux to Windows), usage of `cross` is recommended, though you must handle the `keyring` dependency carefully or disable the UI/keychain for specific targets if needed.

## Configuration

The application is primarily configured via CLI flags, but it respects:

*   `JWT_TESTER_KEYCHAIN_SERVICE`: Overrides the service name used in the OS keychain.
*   `JWT_TESTER_UI_ASSETS_DIR`: Point to external UI assets (skips embedded assets).
*   `JWT_TESTER_NPM`: Path to npm executable (for build scripts).
