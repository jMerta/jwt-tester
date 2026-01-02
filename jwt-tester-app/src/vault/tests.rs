use super::{KeyEntryInput, MemoryKeychain, ProjectInput, TokenEntryInput, Vault, VaultConfig};
use std::sync::Arc;
use tempfile::TempDir;

fn memory_vault() -> Vault {
    Vault::open(VaultConfig {
        no_persist: true,
        data_dir: None,
    })
    .expect("open memory vault")
}

fn sqlite_vault() -> (TempDir, Vault, Arc<MemoryKeychain>) {
    let dir = TempDir::new().expect("temp dir");
    let keychain = Arc::new(MemoryKeychain::new());
    let vault = Vault::open_with(
        VaultConfig {
            no_persist: false,
            data_dir: Some(dir.path().to_path_buf()),
        },
        keychain.clone(),
        "jwt-tester-test".to_string(),
    )
    .expect("open sqlite vault");
    (dir, vault, keychain)
}

fn add_project(vault: &Vault, name: &str) -> super::ProjectEntry {
    vault
        .add_project(ProjectInput {
            name: name.to_string(),
            description: Some(" notes ".to_string()),
            tags: vec![
                " alpha ".to_string(),
                "beta".to_string(),
                "alpha".to_string(),
            ],
        })
        .expect("add project")
}

#[test]
fn project_crud_and_find() {
    let vault = memory_vault();
    let project = add_project(&vault, "alpha");
    assert_eq!(project.name, "alpha");
    assert_eq!(project.description.as_deref(), Some("notes"));
    assert_eq!(project.tags, vec!["alpha".to_string(), "beta".to_string()]);

    let listed = vault.list_projects().expect("list projects");
    assert_eq!(listed.len(), 1);

    let found = vault.find_project_by_name("alpha").expect("find project");
    assert_eq!(found.unwrap().id, project.id);

    let found = vault
        .find_project_by_id(&project.id)
        .expect("find project by id");
    assert_eq!(found.unwrap().name, "alpha");

    let duplicate = vault.add_project(ProjectInput {
        name: "alpha".to_string(),
        description: None,
        tags: Vec::new(),
    });
    assert!(duplicate.is_err());

    let empty = vault.add_project(ProjectInput {
        name: "   ".to_string(),
        description: None,
        tags: Vec::new(),
    });
    assert!(empty.is_err());

    let none = vault.find_project_by_name("   ").expect("find empty");
    assert!(none.is_none());
}

#[test]
fn key_crud_and_default_clears() {
    let vault = memory_vault();
    let project = add_project(&vault, "alpha");

    let key = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: " ".to_string(),
            kind: "hmac".to_string(),
            secret: "super-secret".to_string(),
            kid: Some("kid1".to_string()),
            description: Some("desc".to_string()),
            tags: vec!["a".to_string()],
        })
        .expect("add key");
    assert!(key.name.starts_with("key-"));

    vault
        .set_default_key(&project.id, Some(&key.id))
        .expect("set default");
    let updated = vault
        .find_project_by_id(&project.id)
        .expect("find project")
        .expect("project");
    assert_eq!(updated.default_key_id.as_deref(), Some(key.id.as_str()));

    let material = vault.get_key_material(&key.id).expect("get material");
    assert_eq!(material, "super-secret");

    let _second = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: "primary".to_string(),
            kind: "hmac".to_string(),
            secret: "secret-2".to_string(),
            kid: None,
            description: None,
            tags: Vec::new(),
        })
        .expect("add key");
    let keys = vault.list_keys(Some(&project.id)).expect("list keys");
    assert_eq!(keys.len(), 2);

    vault.delete_key(&key.id).expect("delete key");
    let updated = vault
        .find_project_by_id(&project.id)
        .expect("find project")
        .expect("project");
    assert!(updated.default_key_id.is_none());

    let missing = vault.get_key_material("missing");
    assert!(missing.is_err());

    let bad_project = vault.add_key(KeyEntryInput {
        project_id: " ".to_string(),
        name: "x".to_string(),
        kind: "hmac".to_string(),
        secret: "secret".to_string(),
        kid: None,
        description: None,
        tags: Vec::new(),
    });
    assert!(bad_project.is_err());

    let bad_secret = vault.add_key(KeyEntryInput {
        project_id: project.id.clone(),
        name: "x".to_string(),
        kind: "hmac".to_string(),
        secret: "   ".to_string(),
        kid: None,
        description: None,
        tags: Vec::new(),
    });
    assert!(bad_secret.is_err());
}

#[test]
fn token_crud_and_project_delete_cascade() {
    let vault = memory_vault();
    let project = add_project(&vault, "alpha");

    let token = vault
        .add_token(TokenEntryInput {
            project_id: project.id.clone(),
            name: "t1".to_string(),
            token: "token-value".to_string(),
        })
        .expect("add token");
    let material = vault.get_token_material(&token.id).expect("token material");
    assert_eq!(material, "token-value");

    let tokens = vault.list_tokens(Some(&project.id)).expect("list tokens");
    assert_eq!(tokens.len(), 1);

    vault.delete_token(&token.id).expect("delete token");
    let tokens = vault.list_tokens(Some(&project.id)).expect("list tokens");
    assert!(tokens.is_empty());

    let key = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: "k1".to_string(),
            kind: "hmac".to_string(),
            secret: "secret".to_string(),
            kid: None,
            description: None,
            tags: Vec::new(),
        })
        .expect("add key");
    let token = vault
        .add_token(TokenEntryInput {
            project_id: project.id.clone(),
            name: "t2".to_string(),
            token: "token-2".to_string(),
        })
        .expect("add token");

    vault.delete_project(&project.id).expect("delete project");
    let keys = vault.list_keys(Some(&project.id)).expect("list keys");
    let tokens = vault.list_tokens(Some(&project.id)).expect("list tokens");
    assert!(keys.is_empty());
    assert!(tokens.is_empty());

    let _ = key;
    let _ = token;
}

#[test]
fn export_import_roundtrip_and_replace() {
    let vault = memory_vault();
    let project = add_project(&vault, "alpha");
    let key = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: "k1".to_string(),
            kind: "hmac".to_string(),
            secret: "secret".to_string(),
            kid: None,
            description: None,
            tags: Vec::new(),
        })
        .expect("add key");
    let token = vault
        .add_token(TokenEntryInput {
            project_id: project.id.clone(),
            name: "t1".to_string(),
            token: "token".to_string(),
        })
        .expect("add token");

    let bundle = vault.export_bundle("passphrase").expect("export bundle");
    let other = memory_vault();
    other
        .import_bundle(&bundle, "passphrase", false)
        .expect("import bundle");

    let projects = other.list_projects().expect("list projects");
    assert_eq!(projects.len(), 1);
    let keys = other.list_keys(None).expect("list keys");
    let tokens = other.list_tokens(None).expect("list tokens");
    assert_eq!(keys.len(), 1);
    assert_eq!(tokens.len(), 1);
    assert_eq!(other.get_key_material(&keys[0].id).unwrap(), "secret");
    assert_eq!(other.get_token_material(&tokens[0].id).unwrap(), "token");

    let err = other.import_bundle(&bundle, "passphrase", false);
    assert!(err.is_err());

    other
        .import_bundle(&bundle, "passphrase", true)
        .expect("import replace");

    assert_eq!(key.project_id, project.id);
    assert_eq!(token.project_id, project.id);

    let empty_pass = vault.export_bundle(" ");
    assert!(empty_pass.is_err());
}

#[test]
fn sqlite_roundtrip_persists_metadata() {
    let (dir, vault, keychain) = sqlite_vault();
    let project = add_project(&vault, "alpha");
    let key = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: "k1".to_string(),
            kind: "hmac".to_string(),
            secret: "secret".to_string(),
            kid: None,
            description: None,
            tags: Vec::new(),
        })
        .expect("add key");
    vault
        .set_default_key(&project.id, Some(&key.id))
        .expect("set default");
    let token = vault
        .add_token(TokenEntryInput {
            project_id: project.id.clone(),
            name: "t1".to_string(),
            token: "token".to_string(),
        })
        .expect("add token");

    let _keep_dir = dir;
    let reopened = Vault::open_with(
        VaultConfig {
            no_persist: false,
            data_dir: Some(_keep_dir.path().to_path_buf()),
        },
        keychain.clone(),
        "jwt-tester-test".to_string(),
    )
    .expect("reopen sqlite vault");
    let projects = reopened.list_projects().expect("list projects");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].default_key_id.as_deref(), Some(key.id.as_str()));
    let keys = reopened.list_keys(Some(&project.id)).expect("list keys");
    assert_eq!(keys.len(), 1);
    let tokens = reopened
        .list_tokens(Some(&project.id))
        .expect("list tokens");
    assert_eq!(tokens.len(), 1);
    assert_eq!(reopened.get_key_material(&keys[0].id).unwrap(), "secret");
    assert_eq!(reopened.get_token_material(&tokens[0].id).unwrap(), "token");
    assert_eq!(token.project_id, project.id);
}

#[test]
fn sqlite_delete_project_cleans_keychain() {
    let (_dir, vault, keychain) = sqlite_vault();
    let project = add_project(&vault, "alpha");
    let key = vault
        .add_key(KeyEntryInput {
            project_id: project.id.clone(),
            name: "k1".to_string(),
            kind: "hmac".to_string(),
            secret: "secret".to_string(),
            kid: None,
            description: None,
            tags: Vec::new(),
        })
        .expect("add key");
    let token = vault
        .add_token(TokenEntryInput {
            project_id: project.id.clone(),
            name: "t1".to_string(),
            token: "token".to_string(),
        })
        .expect("add token");

    assert!(keychain.len() >= 2);
    vault.delete_project(&project.id).expect("delete project");
    assert_eq!(keychain.len(), 0);
    let _ = key;
    let _ = token;
}
