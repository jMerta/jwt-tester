mod common;
use common::{at_path, encode_token, fixture_path, run_json};

#[test]
fn verify_explain_includes_inferred_flag() {
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

    let out = run_json(&["verify", "--secret", &at_path(&secret), "--explain", &token]);
    assert!(out["data"]["valid"].as_bool().unwrap_or(false));
    assert!(out["data"]["explain"]["alg_inferred"]
        .as_bool()
        .unwrap_or(false));
}
