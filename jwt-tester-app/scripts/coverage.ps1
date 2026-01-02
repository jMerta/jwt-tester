param(
    [switch]$Html,
    [switch]$Lcov
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo not found in PATH. Install Rust toolchain first."
    exit 1
}

if (-not (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue)) {
    Write-Error "cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"
    exit 1
}

$baseArgs = @("llvm-cov", "--workspace", "--all-features")

if (-not $Html -and -not $Lcov) {
    cargo @baseArgs
    exit $LASTEXITCODE
}

if ($Html) {
    cargo @baseArgs "--html" "--output-dir" "target/coverage/html"
}

if ($Lcov) {
    cargo @baseArgs "--lcov" "--output-path" "target/coverage/lcov.info"
}
