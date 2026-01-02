#!/usr/bin/env bash
set -euo pipefail

cli_only=false
ui=false
release=false
passthrough=()

for arg in "$@"; do
  case "$arg" in
    --cli-only)
      cli_only=true
      ;;
    --ui)
      ui=true
      ;;
    --release)
      release=true
      ;;
    *)
      passthrough+=("$arg")
      ;;
  esac
done

if $cli_only && $ui; then
  echo "Choose either --cli-only or --ui." >&2
  exit 1
fi

manifest_dir="$(cd "$(dirname "$0")" && pwd)"
cmd=(cargo build --manifest-path "${manifest_dir}/Cargo.toml")

if $release; then
  cmd+=(--release)
fi

if $cli_only; then
  cmd+=(--no-default-features --features cli-only)
elif $ui; then
  cmd+=(--features ui)
fi

cmd+=("${passthrough[@]}")

echo "Running: ${cmd[*]}"
"${cmd[@]}"
