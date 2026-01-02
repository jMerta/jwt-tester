use serde_json::Value;

mod common;
use common::{at_path, encode_token, fixture_path};

fn parse_json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("invalid JSON output")
}

#[test]
fn encode_and_verify_with_stdin_secret() {
    let secret = "stdin-secret";
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .args(["encode", "--alg", "hs256", "--secret", "-", "--exp", "+1h"])
        .write_stdin(secret)
        .output()
        .expect("encode");
    assert!(output.status.success(), "encode failed: {output:?}");
    let json = parse_json(&output);
    let token = json["data"]["token"].as_str().expect("token");

    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .args(["verify", "--alg", "hs256", "--secret", "-", token])
        .write_stdin(secret)
        .output()
        .expect("verify");
    assert!(output.status.success(), "verify failed: {output:?}");
    let json = parse_json(&output);
    assert!(json["data"]["valid"].as_bool().unwrap_or(false));
}

#[test]
fn decode_reads_token_from_stdin() {
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

    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .args(["decode", "-"])
        .write_stdin(token.as_bytes())
        .output()
        .expect("decode");
    assert!(output.status.success(), "decode failed: {output:?}");
    let json = parse_json(&output);
    assert!(json["data"]["payload"].is_object());
}

#[test]
fn encode_claims_from_env_and_env_secret() {
    let secret = "env-secret";
    let claims = r#"{"sub":"env-user"}"#;

    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .env("JWT_TESTER_ENV_SECRET", secret)
        .env("JWT_TESTER_ENV_CLAIMS", claims)
        .args([
            "encode",
            "--alg",
            "hs256",
            "--secret",
            "env:JWT_TESTER_ENV_SECRET",
            "--exp",
            "+1h",
            "env:JWT_TESTER_ENV_CLAIMS",
        ])
        .output()
        .expect("encode");
    assert!(output.status.success(), "encode failed: {output:?}");
    let json = parse_json(&output);
    let token = json["data"]["token"].as_str().expect("token");

    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .env("JWT_TESTER_ENV_SECRET", secret)
        .args([
            "verify",
            "--alg",
            "hs256",
            "--secret",
            "env:JWT_TESTER_ENV_SECRET",
            token,
        ])
        .output()
        .expect("verify");
    assert!(output.status.success(), "verify failed: {output:?}");
    let json = parse_json(&output);
    assert!(json["data"]["valid"].as_bool().unwrap_or(false));
}
