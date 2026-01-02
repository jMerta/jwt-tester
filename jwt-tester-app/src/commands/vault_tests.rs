use super::vault::execute;
use crate::cli::{KeyCmd, ProjectCmd, TokenCmd, VaultArgs, VaultCmd};
use crate::error::ErrorKind;
use crate::vault::{Vault, VaultConfig};

fn memory_vault() -> Vault {
    Vault::open(VaultConfig {
        no_persist: true,
        data_dir: None,
    })
    .expect("open vault")
}

#[test]
fn execute_project_add_list_delete() {
    let vault = memory_vault();
    let add = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: Some("notes".to_string()),
                tag: vec!["one".to_string()],
            }),
        },
    )
    .expect("add project");
    let project_id = add.data["project"]["id"].as_str().expect("project id");

    let list = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::List { details: false }),
        },
    )
    .expect("list projects");
    assert_eq!(list.data["projects"].as_array().unwrap().len(), 1);

    let delete = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Delete {
                id: Some(project_id.to_string()),
                name: None,
            }),
        },
    )
    .expect("delete project");
    assert_eq!(delete.data["deleted"], project_id);

    let list = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::List { details: false }),
        },
    )
    .expect("list projects");
    assert!(list.data["projects"].as_array().unwrap().is_empty());
}

#[test]
fn execute_set_default_key_variants() {
    let vault = memory_vault();
    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");

    let key_out = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Add {
                project: "alpha".to_string(),
                name: Some("primary".to_string()),
                kind: "hmac".to_string(),
                kid: Some("kid1".to_string()),
                description: None,
                tag: Vec::new(),
                secret: "secret".to_string(),
            }),
        },
    )
    .expect("add key");
    let key_id = key_out.data["key"]["id"].as_str().expect("key id");

    let set_default = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::SetDefaultKey {
                project: "alpha".to_string(),
                key_id: Some(key_id.to_string()),
                key_name: None,
                clear: false,
            }),
        },
    )
    .expect("set default key");
    assert_eq!(set_default.data["default_key_id"], key_id);

    let clear = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::SetDefaultKey {
                project: "alpha".to_string(),
                key_id: None,
                key_name: None,
                clear: true,
            }),
        },
    )
    .expect("clear default key");
    assert!(clear.data["default_key_id"].is_null());

    let key_out = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Add {
                project: "alpha".to_string(),
                name: Some("named".to_string()),
                kind: "hmac".to_string(),
                kid: None,
                description: None,
                tag: Vec::new(),
                secret: "secret".to_string(),
            }),
        },
    )
    .expect("add key");
    let _ = key_out;

    let set_by_name = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::SetDefaultKey {
                project: "alpha".to_string(),
                key_id: None,
                key_name: Some("named".to_string()),
                clear: false,
            }),
        },
    )
    .expect("set default by name");
    assert!(set_by_name.data["default_key_id"].is_string());

    let err = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::SetDefaultKey {
                project: "alpha".to_string(),
                key_id: None,
                key_name: None,
                clear: false,
            }),
        },
    )
    .expect_err("expected error");
    assert_eq!(err.kind, ErrorKind::InvalidKey);
}

#[test]
fn execute_key_token_export_import() {
    let vault = memory_vault();
    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");

    let key = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Add {
                project: "alpha".to_string(),
                name: None,
                kind: "hmac".to_string(),
                kid: None,
                description: None,
                tag: Vec::new(),
                secret: "secret".to_string(),
            }),
        },
    )
    .expect("add key");
    let key_id = key.data["key"]["id"].as_str().expect("key id");

    let list_keys = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::List {
                project: "alpha".to_string(),
                details: false,
            }),
        },
    )
    .expect("list keys");
    assert_eq!(list_keys.data["keys"].as_array().unwrap().len(), 1);

    let token = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Token(TokenCmd::Add {
                project: "alpha".to_string(),
                name: "t1".to_string(),
                token: "jwt".to_string(),
            }),
        },
    )
    .expect("add token");
    let token_id = token.data["token"]["id"].as_str().expect("token id");

    let list_tokens = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Token(TokenCmd::List {
                project: "alpha".to_string(),
                details: false,
            }),
        },
    )
    .expect("list tokens");
    assert_eq!(list_tokens.data["tokens"].as_array().unwrap().len(), 1);

    let export = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Export {
                out: None,
                passphrase: "passphrase".to_string(),
            },
        },
    )
    .expect("export vault");
    assert!(export.data["bundle"].is_object());
    let import = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Import {
                bundle: export.text.clone(),
                passphrase: "passphrase".to_string(),
                replace: true,
            },
        },
    )
    .expect("import vault");
    assert_eq!(import.data["imported"], true);

    let delete_token = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Token(TokenCmd::Delete {
                id: Some(token_id.to_string()),
                project: None,
                name: None,
            }),
        },
    )
    .expect("delete token");
    assert_eq!(delete_token.data["deleted"], token_id);

    let delete_key = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Delete {
                id: Some(key_id.to_string()),
                project: None,
                name: None,
            }),
        },
    )
    .expect("delete key");
    assert_eq!(delete_key.data["deleted"], key_id);
}

#[test]
fn execute_project_delete_by_name() {
    let vault = memory_vault();
    let add = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");
    let project_id = add.data["project"]["id"].as_str().expect("project id");

    let deleted = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Delete {
                id: None,
                name: Some("alpha".to_string()),
            }),
        },
    )
    .expect("delete by name");
    assert_eq!(deleted.data["deleted"], project_id);
}

#[test]
fn execute_project_list_details_includes_tags() {
    let vault = memory_vault();
    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: Some("notes".to_string()),
                tag: vec!["one".to_string(), "two".to_string()],
            }),
        },
    )
    .expect("add project");

    let list = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::List { details: true }),
        },
    )
    .expect("list details");
    assert!(list.text.contains("tags="));
    assert!(list.text.contains("desc="));
}

#[test]
fn execute_key_list_accepts_project_id() {
    let vault = memory_vault();
    let project = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");
    let project_id = project.data["project"]["id"].as_str().expect("project id");

    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Add {
                project: "alpha".to_string(),
                name: Some("primary".to_string()),
                kind: "hmac".to_string(),
                kid: None,
                description: None,
                tag: Vec::new(),
                secret: "secret".to_string(),
            }),
        },
    )
    .expect("add key");

    let list = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::List {
                project: project_id.to_string(),
                details: false,
            }),
        },
    )
    .expect("list keys by id");
    assert_eq!(list.data["keys"].as_array().unwrap().len(), 1);
}

#[test]
fn execute_key_delete_by_name() {
    let vault = memory_vault();
    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");

    let key = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Add {
                project: "alpha".to_string(),
                name: Some("primary".to_string()),
                kind: "hmac".to_string(),
                kid: None,
                description: None,
                tag: Vec::new(),
                secret: "secret".to_string(),
            }),
        },
    )
    .expect("add key");
    let key_id = key.data["key"]["id"].as_str().expect("key id");

    let deleted = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Key(KeyCmd::Delete {
                id: None,
                project: Some("alpha".to_string()),
                name: Some("primary".to_string()),
            }),
        },
    )
    .expect("delete key by name");
    assert_eq!(deleted.data["deleted"], key_id);
}

#[test]
fn execute_token_delete_by_name() {
    let vault = memory_vault();
    execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Project(ProjectCmd::Add {
                name: "alpha".to_string(),
                description: None,
                tag: Vec::new(),
            }),
        },
    )
    .expect("add project");

    let token = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Token(TokenCmd::Add {
                project: "alpha".to_string(),
                name: "t1".to_string(),
                token: "jwt".to_string(),
            }),
        },
    )
    .expect("add token");
    let token_id = token.data["token"]["id"].as_str().expect("token id");

    let deleted = execute(
        &vault,
        VaultArgs {
            cmd: VaultCmd::Token(TokenCmd::Delete {
                id: None,
                project: Some("alpha".to_string()),
                name: Some("t1".to_string()),
            }),
        },
    )
    .expect("delete token by name");
    assert_eq!(deleted.data["deleted"], token_id);
}
