mod common;

use common::{at_path, fixture_path, TestVault};

#[test]
fn vault_export_import_roundtrip() {
    let vault = TestVault::new();
    let secret = fixture_path("hmac.key");

    let project = vault.run_json(&[
        "vault",
        "project",
        "add",
        "alpha",
        "--description",
        "staging env",
        "--tag",
        "staging",
        "--tag",
        "api",
    ]);
    let project_id = project["data"]["project"]["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let key_id = key["data"]["key"]["id"].as_str().unwrap().to_string();

    let _ = vault.run_json(&[
        "vault",
        "project",
        "set-default-key",
        "--project",
        "alpha",
        "--key-id",
        &key_id,
    ]);

    let _ = vault.run_json(&[
        "vault",
        "token",
        "add",
        "--project",
        "alpha",
        "--name",
        "sample",
        "--token",
        "ey.fake.token",
    ]);

    let dir = tempfile::TempDir::new().expect("temp dir");
    let out_path = dir.path().join("vault.json");
    let _ = vault.run_json(&[
        "vault",
        "export",
        "--passphrase",
        "passphrase",
        "--out",
        out_path.to_str().unwrap(),
    ]);

    let imported = TestVault::new();
    let _ = imported.run_json(&[
        "vault",
        "import",
        "--bundle",
        &at_path(&out_path),
        "--passphrase",
        "passphrase",
    ]);

    let projects = imported.run_json(&["vault", "project", "list"]);
    let project = &projects["data"]["projects"][0];
    assert_eq!(project["name"].as_str().unwrap(), "alpha");
    assert_eq!(project["description"].as_str().unwrap(), "staging env");
    assert_eq!(project["default_key_id"].as_str().unwrap(), key_id);

    let keys = imported.run_json(&["vault", "key", "list", "--project", "alpha"]);
    let key = &keys["data"]["keys"][0];
    assert_eq!(key["kid"].as_str().unwrap(), "kid-1");

    let token = imported.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--exp",
        "+1h",
    ]);
    let token = token["data"]["token"].as_str().unwrap().to_string();

    let verified = imported.run_json(&["verify", "--project", "alpha", "--alg", "hs256", &token]);
    assert_eq!(verified["data"]["valid"], true);

    let _ = imported.run_json(&["vault", "project", "delete", &project_id]);
}

#[test]
fn vault_import_rejects_wrong_passphrase_and_non_empty() {
    let vault = TestVault::new();
    let secret = fixture_path("hmac.key");

    let _ = vault.run_json(&["vault", "project", "add", "alpha"]);
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

    let dir = tempfile::TempDir::new().expect("temp dir");
    let out_path = dir.path().join("vault.json");
    let _ = vault.run_json(&[
        "vault",
        "export",
        "--passphrase",
        "passphrase",
        "--out",
        out_path.to_str().unwrap(),
    ]);

    let target = TestVault::new();
    target.assert_exit(
        &[
            "vault",
            "import",
            "--bundle",
            &at_path(&out_path),
            "--passphrase",
            "wrong",
        ],
        13,
    );

    let _ = target.run_json(&["vault", "project", "add", "existing"]);
    target.assert_exit(
        &[
            "vault",
            "import",
            "--bundle",
            &at_path(&out_path),
            "--passphrase",
            "passphrase",
        ],
        13,
    );

    let _ = target.run_json(&[
        "vault",
        "import",
        "--bundle",
        &at_path(&out_path),
        "--passphrase",
        "passphrase",
        "--replace",
    ]);
}
