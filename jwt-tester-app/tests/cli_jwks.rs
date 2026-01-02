mod common;

use common::{assert_exit, at_path, encode_token, fixture_path, run_json};

#[test]
fn jwks_uses_header_kid() {
    let priv_key = fixture_path("rsa_private.pem");
    let jwks = fixture_path("jwks.json");

    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--kid",
        "rsa1",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "rs256",
        "--jwks",
        &at_path(&jwks),
        &token,
    ]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn jwks_requires_kid_when_multiple_keys() {
    let priv_key = fixture_path("rsa_private.pem");
    let jwks = fixture_path("jwks.json");

    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    assert_exit(
        &[
            "verify",
            "--alg",
            "rs256",
            "--jwks",
            &at_path(&jwks),
            &token,
        ],
        13,
    );
}

#[test]
fn jwks_allow_single_key_without_kid() {
    let priv_key = fixture_path("rsa_private.pem");
    let jwks = fixture_path("jwks_single.json");

    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--exp",
        "+1h",
    ]);

    assert_exit(
        &[
            "verify",
            "--alg",
            "rs256",
            "--jwks",
            &at_path(&jwks),
            &token,
        ],
        13,
    );

    let out = run_json(&[
        "verify",
        "--alg",
        "rs256",
        "--jwks",
        &at_path(&jwks),
        "--allow-single-jwk",
        &token,
    ]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn decode_verifies_with_jwks() {
    let priv_key = fixture_path("rsa_private.pem");
    let jwks = fixture_path("jwks.json");

    let token = encode_token(&[
        "encode",
        "--alg",
        "rs256",
        "--key",
        &at_path(&priv_key),
        "--kid",
        "rsa1",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["decode", "--jwks", &at_path(&jwks), &token]);
    assert_eq!(out["data"]["verified"], true);
    assert_eq!(out["data"]["verification"]["valid"], true);
}
