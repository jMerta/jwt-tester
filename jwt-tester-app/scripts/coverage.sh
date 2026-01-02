#!/usr/bin/env bash
set -euo pipefail

HTML=0
LCOV=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --html) HTML=1; shift ;;
    --lcov) LCOV=1; shift ;;
    *) echo "unknown flag: $1"; exit 1 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH. Install Rust toolchain first." >&2
  exit 1
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov" >&2
  exit 1
fi

BASE_ARGS=("llvm-cov" "--workspace" "--all-features")

if [[ $HTML -eq 0 && $LCOV -eq 0 ]]; then
  cargo "${BASE_ARGS[@]}"
  exit 0
fi

if [[ $HTML -eq 1 ]]; then
  cargo "${BASE_ARGS[@]}" --html --output-dir target/coverage/html
fi

if [[ $LCOV -eq 1 ]]; then
  cargo "${BASE_ARGS[@]}" --lcov --output-path target/coverage/lcov.info
fi
