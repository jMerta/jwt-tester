mod common;

use common::{assert_exit, at_path, encode_token, fixture_path, run_json};
use tempfile::NamedTempFile;

#[test]
fn decode_includes_dates() {
    let secret = fixture_path("hmac.key");
    let token = encode_token(&[
        "encode",
        "--alg",
        "hs256",
        "--secret",
        &at_path(&secret),
        "--iat",
        "now",
        "--exp",
        "+1h",
    ]);

    let out = run_json(&["decode", "--date", "utc", &token]);
    assert!(out["data"]["dates"]["exp"]["raw"].is_i64());
}

#[test]
fn inspect_shows_segments() {
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

    let out = run_json(&["inspect", "--show-segments", &token]);
    assert_eq!(out["data"]["segments"].as_array().unwrap().len(), 3);
}

#[test]
fn split_outputs_signature_hex() {
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

    let out = run_json(&["split", "--format", "json", &token]);
    assert!(out["data"]["signature"]["length"].as_u64().unwrap() > 0);
}

#[test]
fn decode_verifies_with_secret() {
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

    let out = run_json(&["decode", "--secret", &at_path(&secret), &token]);
    assert_eq!(out["data"]["verified"], true);
    assert_eq!(out["data"]["verification"]["valid"], true);
}

#[test]
fn decode_verify_rejects_bad_signature() {
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

    assert_exit(&["decode", "--secret", &at_path(&wrong), &token], 11);
}

#[test]
fn decode_out_writes_json() {
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
    let file = NamedTempFile::new().expect("temp file");
    let path = file.path().to_path_buf();

    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .args(["decode", "--out"])
        .arg(&path)
        .arg(&token)
        .output()
        .expect("run decode");
    assert!(output.status.success());

    let raw = std::fs::read_to_string(&path).expect("read out");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("json");
    assert_eq!(json["ok"], true);
    assert!(json["data"]["header"].is_object());
}

#[test]
fn decode_rejects_invalid_token() {
    assert_exit(&["decode", "not-a-token"], 10);
}
