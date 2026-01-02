# Testing strategy

## Test layers

1. Unit tests
   - duration parsing
   - claim merge logic
   - key format detection
2. Integration tests (CLI)
   - encode → verify round trips for each algorithm
   - decode malformed tokens (wrong segment count, invalid base64url, non-JSON)
   - verify failures (bad signature, expired, nbf, wrong issuer/audience)
3. Golden tests
   - known tokens with expected decoded JSON output
4. Property tests / fuzzing (optional but high value)
   - random tokens and random bytes should never crash the CLI

## Fixtures

Maintain fixtures in a dedicated folder (recommended):
- jwt-tester-app/tests/fixtures (see generate.py for regeneration)

- HMAC secret bytes
- RSA private/public keys (PEM/DER)
- EC keys (P-256/P-384)
- Ed25519 keys
- JWKS documents with:
  - single key,
  - multiple keys,
  - missing `kid`,
  - wrong kty/alg combos

## “Security regression” tests

- Ensure `verify` infers algorithm from the header when `--alg` is omitted.
- Ensure `verify` does not accept `alg=none`.
- Ensure `decode` only claims validity when verification inputs are provided.

## Coverage

- Install tooling (once): `cargo install cargo-llvm-cov`
- Run summary: `jwt-tester-app/scripts/coverage.ps1` (Windows) or `jwt-tester-app/scripts/coverage.sh`
- Generate HTML report: add `-Html` (PowerShell) or `--html` (bash) to the script call.
- Generate LCOV report: add `-Lcov` (PowerShell) or `--lcov` (bash) to the script call.


