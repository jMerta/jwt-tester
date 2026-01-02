mod common;

use common::{at_path, encode_token, fixture_path, run_json};

fn rsa_roundtrip(alg: &str) {
    let priv_key = fixture_path("rsa_private.pem");
    let pub_key = fixture_path("rsa_public.pem");

    let token = encode_token(&[
        "encode",
        "--alg",
        alg,
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["verify", "--alg", alg, "--key", &at_path(&pub_key), &token]);

    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn rs256_encode_verify_pem() {
    rsa_roundtrip("rs256");
}

#[test]
fn rs256_verify_infers_alg_from_header() {
    let priv_key = fixture_path("rsa_private.pem");
    let pub_key = fixture_path("rsa_public.pem");
    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["verify", "--key", &at_path(&pub_key), &token]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn rs384_encode_verify_pem() {
    rsa_roundtrip("rs384");
}

#[test]
fn rs512_encode_verify_pem() {
    rsa_roundtrip("rs512");
}

#[test]
fn ps256_encode_verify_pem() {
    rsa_roundtrip("ps256");
}

#[test]
fn ps384_encode_verify_pem() {
    rsa_roundtrip("ps384");
}

#[test]
fn ps512_encode_verify_pem() {
    rsa_roundtrip("ps512");
}

#[test]
fn ps256_verify_infers_alg_from_header() {
    let priv_key = fixture_path("rsa_private.pem");
    let pub_key = fixture_path("rsa_public.pem");
    let token = encode_token(&[
        "encode",
        "--alg",
        "ps256",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["verify", "--key", &at_path(&pub_key), &token]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn rs256_encode_verify_der() {
    let priv_key = fixture_path("rsa_private.der");
    let pub_key = fixture_path("rsa_public.der");

    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--key-format",
        "der",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "rs256",
        "--key",
        &at_path(&pub_key),
        "--key-format",
        "der",
        &token,
    ]);

    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn es256_encode_verify_pem() {
    let priv_key = fixture_path("ec256_private.pem");
    let pub_key = fixture_path("ec256_public.pem");

    let token = encode_token(&[
        "encode",
        "--alg",
        "es256",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "es256",
        "--key",
        &at_path(&pub_key),
        &token,
    ]);

    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn es384_verify_infers_alg_from_header() {
    let priv_key = fixture_path("ec384_private.pem");
    let pub_key = fixture_path("ec384_public.pem");

    let token = encode_token(&[
        "encode",
        "--alg",
        "es384",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["verify", "--key", &at_path(&pub_key), &token]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn es384_encode_verify_pem() {
    let priv_key = fixture_path("ec384_private.pem");
    let pub_key = fixture_path("ec384_public.pem");

    let token = encode_token(&[
        "encode",
        "--alg",
        "es384",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "es384",
        "--key",
        &at_path(&pub_key),
        &token,
    ]);

    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn eddsa_encode_verify_der() {
    let priv_key = fixture_path("ed25519_private.der");
    let pub_key = fixture_path("ed25519_public.der");

    let token = encode_token(&[
        "encode",
        "--alg",
        "eddsa",
        "--key",
        &at_path(&priv_key),
        "--key-format",
        "der",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "eddsa",
        "--key",
        &at_path(&pub_key),
        "--key-format",
        "der",
        &token,
    ]);

    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn eddsa_verify_infers_alg_from_header() {
    let priv_key = fixture_path("ed25519_private.der");
    let pub_key = fixture_path("ed25519_public.der");

    let token = encode_token(&[
        "encode",
        "--alg",
        "eddsa",
        "--key",
        &at_path(&priv_key),
        "--key-format",
        "der",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--key",
        &at_path(&pub_key),
        "--key-format",
        "der",
        &token,
    ]);
    assert_eq!(out["data"]["valid"], true);
}
