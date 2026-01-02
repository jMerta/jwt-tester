mod common;

use base64::Engine;
use common::{assert_exit, at_path, encode_token, fixture_path, run_json};

fn hmac_roundtrip(alg: &str) {
    let secret = fixture_path("hmac.key");
    let token = encode_token(&[
        "encode",
        "--alg",
        alg,
        "--secret",
        &at_path(&secret),
        "--exp",
        "+1h",
    ]);
    let out = run_json(&[
        "verify",
        "--alg",
        alg,
        "--secret",
        &at_path(&secret),
        &token,
    ]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn hs256_encode_verify_roundtrip() {
    let secret = fixture_path("hmac.key");
    let claims_file = fixture_path("claims.json");

    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--iss",
        "issuer-1",
        "--sub",
        "user-123",
        "--aud",
        "aud-1",
        "--aud",
        "aud-2",
        "--claim",
        "role=admin",
        "--claim",
        "active=true",
        "--claim-file",
        &at_path(&claims_file),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&[
        "verify",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--iss",
        "issuer-1",
        "--sub",
        "user-123",
        "--aud",
        "aud-1",
        "--aud",
        "aud-2",
        &token,
    ]);

    assert_eq!(out["data"]["valid"], true);
    assert_eq!(out["data"]["claims"]["role"], "admin");
    assert_eq!(out["data"]["claims"]["tier"], "gold");
}

#[test]
fn hs384_encode_verify_roundtrip() {
    hmac_roundtrip("hs384");
}

#[test]
fn hs512_encode_verify_roundtrip() {
    hmac_roundtrip("hs512");
}

#[test]
fn hs256_verify_infers_alg_from_header() {
    let secret = fixture_path("hmac.key");
    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["verify", "--secret", &at_path(&secret), &token]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn hs256_accepts_b64_secret() {
    let secret = fixture_path("hmac.key");
    let secret_raw = std::fs::read_to_string(&secret).expect("secret");
    let encoded = base64::engine::general_purpose::STANDARD.encode(secret_raw.trim());
    let spec = format!("b64:{}", encoded);

    let token = encode_token(&[
        "encode", "--alg", "hs256", "--secret", &spec, "--exp", "+1h",
    ]);

    let out = run_json(&["verify", "--secret", &spec, &token]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn hs256_rejects_bad_signature() {
    let secret = fixture_path("hmac.key");
    let wrong = fixture_path("hmac_alt.key");

    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--exp",
        "+1h",
    ]);

    assert_exit(
        &[
            "verify",
            "--alg",
            "hs256",
            "--secret",
            &at_path(&wrong),
            &token,
        ],
        11,
    );
}

#[test]
fn hs256_expired_token_behavior() {
    let secret = fixture_path("hmac.key");
    let expired = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--exp=-1h",
    ]);

    assert_exit(
        &[
            "verify",
            "--alg",
            "hs256",
            "--secret",
            &at_path(&secret),
            &expired,
        ],
        12,
    );

    let out = run_json(&[
        "verify",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--ignore-exp",
        &expired,
    ]);
    assert_eq!(out["data"]["valid"], true);
}

#[test]
fn hs256_rejects_key_flag() {
    let secret = fixture_path("hmac.key");
    let rsa_pub = fixture_path("rsa_public.pem");
    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--exp",
        "+1h",
    ]);

    assert_exit(
        &[
            "verify",
            "--alg",
            "hs256",
            "--key",
            &at_path(&rsa_pub),
            &token,
        ],
        13,
    );
}

#[test]
fn exp_flag_defaults_to_30m() {
    let secret = fixture_path("hmac.key");
    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--exp",
    ]);

    let out = run_json(&["decode", &token]);
    assert!(out["data"]["payload"]["exp"].is_number());
}

#[test]
fn keep_payload_order_preserves_input_order() {
    let secret = fixture_path("hmac.key");
    let claims = r#"{"b":1,"a":2}"#;

    let token_sorted = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        claims,
    ]);
    let token_kept = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--keep-payload-order",
        claims,
    ]);

    let out_sorted = run_json(&["decode", &token_sorted]);
    let out_kept = run_json(&["decode", &token_kept]);

    let payload_sorted = serde_json::to_string(&out_sorted["data"]["payload"]).unwrap();
    let payload_kept = serde_json::to_string(&out_kept["data"]["payload"]).unwrap();

    assert!(payload_sorted.find("\"a\"").unwrap() < payload_sorted.find("\"b\"").unwrap());
    assert!(payload_kept.find("\"b\"").unwrap() < payload_kept.find("\"a\"").unwrap());
}
