mod common;
use common::{at_path, fixture_path, TestVault};

#[test]
fn text_output_prints_token() {
    let secret = fixture_path("hmac.key");
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .args([
            "encode",
            "--alg",
            "hs256",
            "--secret",
            &at_path(&secret),
            "--exp",
            "+1h",
        ])
        .output()
        .expect("encode");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().split('.').count() == 3,
        "stdout was: {stdout}"
    );
}

#[test]
fn text_output_prints_ok_when_empty() {
    let vault = TestVault::new();
    let output = vault
        .cmd()
        .args(["vault", "project", "list"])
        .output()
        .expect("list projects");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "OK");
}

#[test]
fn text_error_respects_no_color_and_verbose() {
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .args(["--no-color", "--verbose", "decode", "not-a-token"])
        .output()
        .expect("decode");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains('\u{1b}'));
    assert!(stderr.contains('['), "stderr missing code prefix: {stderr}");
}

#[test]
fn text_error_uses_color_by_default() {
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .args(["decode", "not-a-token"])
        .output()
        .expect("decode");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\u{1b}[31m"));
}
