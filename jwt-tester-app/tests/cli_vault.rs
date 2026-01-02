mod common;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use common::{at_path, fixture_path, TestVault};
use tempfile::TempDir;

#[test]
fn vault_roundtrip_and_key_selection() {
    let vault = TestVault::new();
    let secret = fixture_path("hmac.key");
    let alt = fixture_path("hmac_alt.key");

    let project = vault.run_json(&["vault", "project", "add", "alpha"]);
    let project_id = project["data"]["project"]["id"]
        .as_str()
        .expect("project id");

    let _ = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "alpha",
        "--name",
        "primary",
        "--kind",
        "hmac",
        "--secret",
        &at_path(&secret),
    ]);

    let _ = vault.run_json(&[
        "vault",
        "project",
        "set-default-key",
        "--project",
        "alpha",
        "--key-name",
        "primary",
    ]);

    let token = vault.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--exp",
        "+1h",
    ]);
    let token = token["data"]["token"].as_str().unwrap().to_string();

    let verified = vault.run_json(&["verify", "--project", "alpha", "--alg", "hs256", &token]);
    assert_eq!(verified["data"]["valid"], true);

    let _ = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "alpha",
        "--name",
        "secondary",
        "--kind",
        "hmac",
        "--secret",
        &at_path(&alt),
    ]);

    let token_alt = vault.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--key-name",
        "secondary",
        "--exp",
        "+1h",
    ]);
    let token_alt = token_alt["data"]["token"].as_str().unwrap().to_string();

    vault.assert_exit(
        &["verify", "--project", "alpha", "--alg", "hs256", &token_alt],
        11,
    );

    let verified_alt = vault.run_json(&[
        "verify",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--try-all-keys",
        &token_alt,
    ]);
    assert_eq!(verified_alt["data"]["valid"], true);

    let beta = vault.run_json(&["vault", "project", "add", "beta"]);
    let beta_id = beta["data"]["project"]["id"].as_str().unwrap().to_string();

    let _ = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "beta",
        "--name",
        "k1",
        "--kind",
        "hmac",
        "--secret",
        &at_path(&secret),
    ]);
    let _ = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "beta",
        "--name",
        "k2",
        "--kind",
        "hmac",
        "--secret",
        &at_path(&alt),
    ]);

    vault.assert_exit(
        &[
            "encode",
            "--project",
            "beta",
            "--alg",
            "hs256",
            "--exp",
            "+1h",
        ],
        13,
    );

    let _ = vault.run_json(&["vault", "project", "delete", project_id]);
    let _ = vault.run_json(&["vault", "project", "delete", &beta_id]);
}

#[test]
fn vault_metadata_and_kid_selection() {
    let vault = TestVault::new();
    let secret = fixture_path("hmac.key");
    let alt = fixture_path("hmac_alt.key");

    let project = vault.run_json(&[
        "vault",
        "project",
        "add",
        "alpha",
        "--description",
        "staging environment",
        "--tag",
        " staging ",
        "--tag",
        "api",
    ]);
    let project_id = project["data"]["project"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        project["data"]["project"]["description"].as_str().unwrap(),
        "staging environment"
    );
    let tags = project["data"]["project"]["tags"]
        .as_array()
        .expect("project tags");
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].as_str().unwrap(), "staging");
    assert_eq!(tags[1].as_str().unwrap(), "api");

    let key = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "alpha",
        "--name",
        "primary",
        "--kind",
        "hmac",
        "--kid",
        "kid-1",
        "--description",
        "primary key",
        "--tag",
        "primary",
        "--secret",
        &at_path(&secret),
    ]);
    assert_eq!(key["data"]["key"]["kid"].as_str().unwrap(), "kid-1");

    let _ = vault.run_json(&[
        "vault",
        "key",
        "add",
        "--project",
        "alpha",
        "--name",
        "secondary",
        "--kind",
        "hmac",
        "--kid",
        "kid-2",
        "--secret",
        &at_path(&alt),
    ]);

    let token = vault.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--key-name",
        "secondary",
        "--kid",
        "kid-2",
        "--exp",
        "+1h",
    ]);
    let token = token["data"]["token"].as_str().unwrap().to_string();

    let verified = vault.run_json(&["verify", "--project", "alpha", "--alg", "hs256", &token]);
    assert_eq!(verified["data"]["valid"], true);

    let token_bad = vault.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--key-name",
        "primary",
        "--kid",
        "missing",
        "--exp",
        "+1h",
    ]);
    let token_bad = token_bad["data"]["token"].as_str().unwrap().to_string();
    vault.assert_exit(
        &["verify", "--project", "alpha", "--alg", "hs256", &token_bad],
        13,
    );

    let _ = vault.run_json(&["vault", "project", "delete", &project_id]);
}

#[test]
fn vault_key_generate_hmac_reveal_and_out() {
    let vault = TestVault::new();
    let _ = vault.run_json(&["vault", "project", "add", "alpha"]);

    let dir = TempDir::new().expect("temp dir");
    let out_path = dir.path().join("hmac.key");
    let out_str = out_path.to_str().expect("path str");

    let generated = vault.run_json(&[
        "vault",
        "key",
        "generate",
        "--project",
        "alpha",
        "--name",
        "generated",
        "--kind",
        "hmac",
        "--hmac-bytes",
        "24",
        "--reveal",
        "--out",
        out_str,
    ]);

    assert_eq!(generated["data"]["format"].as_str().unwrap(), "base64url");
    let material = generated["data"]["material"].as_str().expect("material");
    let decoded = URL_SAFE_NO_PAD
        .decode(material.as_bytes())
        .expect("decode base64");
    assert_eq!(decoded.len(), 24);

    let written = std::fs::read_to_string(&out_path).expect("read out");
    assert_eq!(written, material);
}

#[test]
fn vault_key_generate_rsa_no_reveal() {
    let vault = TestVault::new();
    let _ = vault.run_json(&["vault", "project", "add", "alpha"]);

    let generated = vault.run_json(&[
        "vault",
        "key",
        "generate",
        "--project",
        "alpha",
        "--name",
        "server",
        "--kind",
        "rsa",
        "--rsa-bits",
        "2048",
    ]);

    assert_eq!(generated["data"]["format"].as_str().unwrap(), "pem");
    assert!(generated["data"].get("material").is_none());
    assert_eq!(generated["data"]["key"]["kind"].as_str().unwrap(), "rsa");
}
