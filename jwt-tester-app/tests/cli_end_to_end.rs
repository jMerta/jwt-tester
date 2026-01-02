mod common;
use common::TestVault;

#[test]
fn end_to_end_vault_flow() {
    let vault = TestVault::new();

    let project = vault.run_json(&["vault", "project", "add", "alpha"]);
    let project_id = project["data"]["project"]["id"]
        .as_str()
        .expect("project id");

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
        "--secret",
        "test-secret",
    ]);
    let key_id = key["data"]["key"]["id"].as_str().expect("key id");

    vault.run_json(&[
        "vault",
        "project",
        "set-default-key",
        "--project",
        "alpha",
        "--key-id",
        key_id,
    ]);

    let token_out = vault.run_json(&[
        "encode",
        "--project",
        "alpha",
        "--alg",
        "hs256",
        "--exp",
        "+10m",
        "{\"sub\":\"user\"}",
    ]);
    let token = token_out["data"]["token"].as_str().expect("token");

    let verify = vault.run_json(&["verify", "--project", "alpha", "--explain", token]);
    assert!(verify["data"]["valid"].as_bool().unwrap_or(false));
    assert!(verify["data"]["explain"]["key_source"].is_string());

    let decoded = vault.run_json(&["decode", "--project", "alpha", token]);
    assert!(decoded["data"]["verified"].as_bool().unwrap_or(false));

    let inspect = vault.run_json(&["inspect", token]);
    assert!(inspect["data"]["summary"]["alg"].is_string());

    let split = vault.run_json(&["split", "--format", "json", token]);
    assert!(split["data"]["signature"]["length"].as_u64().unwrap_or(0) > 0);

    let saved = vault.run_json(&[
        "vault",
        "token",
        "add",
        "--project",
        "alpha",
        "--name",
        "saved",
        "--token",
        token,
    ]);
    let token_id = saved["data"]["token"]["id"].as_str().expect("token id");

    let listed = vault.run_json(&["vault", "token", "list", "--project", "alpha"]);
    assert!(!listed["data"]["tokens"].as_array().unwrap().is_empty());

    vault.run_json(&["vault", "token", "delete", token_id]);

    let keys = vault.run_json(&["vault", "key", "list", "--project", "alpha"]);
    assert!(keys["data"]["keys"].as_array().unwrap().len() >= 1);

    let projects = vault.run_json(&["vault", "project", "list"]);
    assert!(projects["data"]["projects"].as_array().unwrap().len() >= 1);

    // Ensure project ID stays stable and matches lookup.
    let fetched = vault.run_json(&["vault", "project", "list"]);
    let ids: Vec<_> = fetched["data"]["projects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["id"].as_str())
        .collect();
    assert!(ids.contains(&project_id));
}
