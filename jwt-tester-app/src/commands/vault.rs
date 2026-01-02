use crate::cli::{KeyCmd, ProjectCmd, TokenCmd, VaultArgs, VaultCmd};
use crate::error::{AppError, AppResult};
use crate::io_utils::read_input;
use crate::keygen::{
    generate_key_material, parse_ec_curve, KeyGenSpec, DEFAULT_HMAC_BYTES, DEFAULT_RSA_BITS,
};
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use crate::vault::{
    KeyEntry, KeyEntryInput, ProjectEntry, ProjectInput, TokenEntry, TokenEntryInput, Vault,
    VaultConfig,
};
use crate::vault_export::ExportBundle;
use serde_json::json;
use std::path::PathBuf;

fn resolve_project_selector(vault: &Vault, selector: &str) -> AppResult<ProjectEntry> {
    if let Some(project) = vault
        .find_project_by_name(selector)
        .map_err(|e| AppError::invalid_key(e.to_string()))?
    {
        return Ok(project);
    }
    if let Some(project) = vault
        .find_project_by_id(selector)
        .map_err(|e| AppError::invalid_key(e.to_string()))?
    {
        return Ok(project);
    }
    Err(AppError::invalid_key(format!(
        "project not found: {selector}"
    )))
}

fn resolve_named_key(vault: &Vault, project_id: &str, name: &str) -> AppResult<KeyEntry> {
    let keys = vault
        .list_keys(Some(project_id))
        .map_err(|e| AppError::invalid_key(e.to_string()))?;
    let matches: Vec<_> = keys.into_iter().filter(|k| k.name == name).collect();
    if matches.is_empty() {
        return Err(AppError::invalid_key(
            "key name not found in project".to_string(),
        ));
    }
    if matches.len() > 1 {
        return Err(AppError::invalid_key(format!(
            "multiple keys named '{name}' found; use key id"
        )));
    }
    Ok(matches.into_iter().next().expect("single match"))
}

fn resolve_named_token(vault: &Vault, project_id: &str, name: &str) -> AppResult<TokenEntry> {
    let tokens = vault
        .list_tokens(Some(project_id))
        .map_err(|e| AppError::invalid_key(e.to_string()))?;
    let matches: Vec<_> = tokens
        .into_iter()
        .filter(|token| token.name == name)
        .collect();
    if matches.is_empty() {
        return Err(AppError::invalid_key(
            "token name not found in project".to_string(),
        ));
    }
    if matches.len() > 1 {
        return Err(AppError::invalid_key(format!(
            "multiple tokens named '{name}' found; use token id"
        )));
    }
    Ok(matches.into_iter().next().expect("single match"))
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(",")
    }
}

fn opt_or_dash(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

fn build_keygen_spec(
    kind: &str,
    hmac_bytes: Option<usize>,
    rsa_bits: Option<usize>,
    ec_curve: Option<String>,
) -> AppResult<(KeyGenSpec, &'static str)> {
    match kind {
        "hmac" => {
            if rsa_bits.is_some() || ec_curve.is_some() {
                return Err(AppError::invalid_key(
                    "--rsa-bits/--ec-curve are only valid for RSA/EC keys".to_string(),
                ));
            }
            Ok((
                KeyGenSpec::Hmac {
                    bytes: hmac_bytes.unwrap_or(DEFAULT_HMAC_BYTES),
                },
                "base64url",
            ))
        }
        "rsa" => {
            if hmac_bytes.is_some() || ec_curve.is_some() {
                return Err(AppError::invalid_key(
                    "--hmac-bytes/--ec-curve are only valid for HMAC/EC keys".to_string(),
                ));
            }
            Ok((
                KeyGenSpec::Rsa {
                    bits: rsa_bits.unwrap_or(DEFAULT_RSA_BITS),
                },
                "pem",
            ))
        }
        "ec" => {
            if hmac_bytes.is_some() || rsa_bits.is_some() {
                return Err(AppError::invalid_key(
                    "--hmac-bytes/--rsa-bits are only valid for HMAC/RSA keys".to_string(),
                ));
            }
            let curve = parse_ec_curve(ec_curve.as_deref())?;
            Ok((KeyGenSpec::Ec { curve }, "pem"))
        }
        "eddsa" => {
            if hmac_bytes.is_some() || rsa_bits.is_some() || ec_curve.is_some() {
                return Err(AppError::invalid_key(
                    "generation options are not valid for EdDSA keys".to_string(),
                ));
            }
            Ok((KeyGenSpec::EdDsa, "pem"))
        }
        "jwks" => Err(AppError::invalid_key(
            "JWKS generation is not supported; paste JWKS JSON instead".to_string(),
        )),
        other => Err(AppError::invalid_key(format!(
            "unsupported key kind '{other}' for generation"
        ))),
    }
}

pub fn run(no_persist: bool, data_dir: Option<PathBuf>, args: VaultArgs, cfg: OutputConfig) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let vault = Vault::open(VaultConfig {
            no_persist,
            data_dir,
        })
        .map_err(|e| AppError::invalid_key(e.to_string()))?;

        execute(&vault, args)
    })();

    match result {
        Ok(out) => {
            emit_ok(cfg, out);
            0
        }
        Err(err) => {
            let code = err.exit_code();
            emit_err(cfg, err);
            code
        }
    }
}

pub(crate) fn execute(vault: &Vault, args: VaultArgs) -> AppResult<CommandOutput> {
    let out = match args.cmd {
        VaultCmd::Project(cmd) => match cmd {
            ProjectCmd::Add {
                name,
                description,
                tag,
            } => {
                let p = vault
                    .add_project(ProjectInput {
                        name,
                        description,
                        tags: tag,
                    })
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                CommandOutput::new(
                    json!({ "project": p }),
                    format!("created project: {} ({})", p.name, p.id),
                )
            }
            ProjectCmd::List { details } => {
                let list = vault
                    .list_projects()
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                let mut lines = Vec::new();
                for p in &list {
                    let line = if details {
                        let default = opt_or_dash(p.default_key_id.as_deref());
                        let tags = format_tags(&p.tags);
                        let desc = opt_or_dash(p.description.as_deref());
                        format!(
                            "{}  {}  default_key_id={} tags={} desc={}",
                            p.id, p.name, default, tags, desc
                        )
                    } else {
                        let default = p
                            .default_key_id
                            .as_deref()
                            .map(|id| format!(" default_key_id={id}"))
                            .unwrap_or_default();
                        format!("{}  {}{}", p.id, p.name, default)
                    };
                    lines.push(line);
                }
                CommandOutput::new(json!({ "projects": list }), lines.join("\n"))
            }
            ProjectCmd::Delete { id, name } => {
                if id.is_some() && name.is_some() {
                    return Err(AppError::invalid_key(
                        "provide either a project id or --name".to_string(),
                    ));
                }
                let project = if let Some(name) = name {
                    vault
                        .find_project_by_name(&name)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?
                        .ok_or_else(|| {
                            AppError::invalid_key(format!("project not found: {name}"))
                        })?
                } else if let Some(id) = id {
                    vault
                        .find_project_by_id(&id)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?
                        .ok_or_else(|| AppError::invalid_key(format!("project not found: {id}")))?
                } else {
                    return Err(AppError::invalid_key(
                        "provide a project id or --name".to_string(),
                    ));
                };
                vault
                    .delete_project(&project.id)
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                CommandOutput::new(
                    json!({ "deleted": project.id }),
                    format!("deleted project: {} ({})", project.name, project.id),
                )
            }
            ProjectCmd::SetDefaultKey {
                project,
                key_id,
                key_name,
                clear,
            } => {
                let p = resolve_project_selector(vault, &project)?;

                if clear {
                    vault
                        .set_default_key(&p.id, None)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?;
                    return Ok(CommandOutput::new(
                        json!({ "project": p.id, "default_key_id": null }),
                        format!("cleared default key for project {}", p.name),
                    ));
                }

                let keys = vault
                    .list_keys(Some(&p.id))
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                let key = if let Some(id) = key_id {
                    keys.into_iter()
                        .find(|k| k.id == id)
                        .ok_or_else(|| AppError::invalid_key("key id not found in project"))?
                } else if let Some(name) = key_name {
                    vault
                        .find_key_in_project(&p.id, &name)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?
                        .ok_or_else(|| AppError::invalid_key("key name not found in project"))?
                } else {
                    return Err(AppError::invalid_key(
                        "provide --key-id or --key-name (or use --clear)",
                    ));
                };

                vault
                    .set_default_key(&p.id, Some(&key.id))
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                CommandOutput::new(
                    json!({ "project": p.id, "default_key_id": key.id }),
                    format!(
                        "set default key for project {} to {} ({})",
                        p.name, key.name, key.id
                    ),
                )
            }
        },
        VaultCmd::Key(cmd) => match cmd {
            KeyCmd::Add {
                project,
                name,
                kind,
                kid,
                description,
                tag,
                secret,
            } => {
                let p = resolve_project_selector(vault, &project)?;
                let secret = read_input(&secret)?;
                let k = vault
                    .add_key(KeyEntryInput {
                        project_id: p.id,
                        name: name.unwrap_or_default(),
                        kind,
                        secret,
                        kid,
                        description,
                        tags: tag,
                    })
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                CommandOutput::new(
                    json!({ "key": k }),
                    format!("created key: {} ({})", k.name, k.id),
                )
            }
            KeyCmd::Generate {
                project,
                name,
                kind,
                kid,
                description,
                tag,
                hmac_bytes,
                rsa_bits,
                ec_curve,
                reveal,
                out,
            } => {
                let p = resolve_project_selector(vault, &project)?;
                let kind = kind.trim().to_ascii_lowercase();
                if kind.is_empty() {
                    return Err(AppError::invalid_key("key kind is required".to_string()));
                }
                let (spec, format) = build_keygen_spec(&kind, hmac_bytes, rsa_bits, ec_curve)?;
                let secret = generate_key_material(spec)?;
                let k = vault
                    .add_key(KeyEntryInput {
                        project_id: p.id,
                        name: name.unwrap_or_default(),
                        kind,
                        secret: secret.clone(),
                        kid,
                        description,
                        tags: tag,
                    })
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;

                if let Some(path) = &out {
                    std::fs::write(path, secret.as_bytes()).map_err(|e| {
                        AppError::internal(format!("failed to write {}: {e}", path.display()))
                    })?;
                }

                let mut data = json!({ "key": k.clone(), "format": format });
                if let Some(obj) = data.as_object_mut() {
                    if reveal {
                        obj.insert("material".to_string(), json!(secret.clone()));
                    }
                    if let Some(path) = &out {
                        obj.insert("path".to_string(), json!(path.display().to_string()));
                    }
                }

                let mut text = format!("generated key: {} ({})", k.name, k.id);
                if let Some(path) = out {
                    text.push_str(&format!("\nmaterial written to {}", path.display()));
                }
                if reveal {
                    text.push_str("\n\n");
                    text.push_str(&secret);
                }
                CommandOutput::new(data, text)
            }
            KeyCmd::List { project, details } => {
                let p = resolve_project_selector(vault, &project)?;
                let keys = vault
                    .list_keys(Some(&p.id))
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                let mut lines = Vec::new();
                for k in &keys {
                    let line = if details {
                        let kid = opt_or_dash(k.kid.as_deref());
                        let tags = format_tags(&k.tags);
                        let desc = opt_or_dash(k.description.as_deref());
                        format!(
                            "{}  {}  {}  kid={} tags={} desc={}",
                            k.id, k.kind, k.name, kid, tags, desc
                        )
                    } else {
                        format!("{}  {}  {}", k.id, k.kind, k.name)
                    };
                    lines.push(line);
                }
                CommandOutput::new(json!({ "keys": keys }), lines.join("\n"))
            }
            KeyCmd::Delete { id, project, name } => {
                if id.is_some() && (project.is_some() || name.is_some()) {
                    return Err(AppError::invalid_key(
                        "provide either a key id or --project/--name".to_string(),
                    ));
                }
                if let Some(id) = id {
                    vault
                        .delete_key(&id)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?;
                    CommandOutput::new(json!({ "deleted": id }), format!("deleted key: {id}"))
                } else {
                    let project = project.ok_or_else(|| {
                        AppError::invalid_key("provide --project with --name".to_string())
                    })?;
                    let name = name.ok_or_else(|| {
                        AppError::invalid_key("provide --name (or delete by id)".to_string())
                    })?;
                    let p = resolve_project_selector(vault, &project)?;
                    let key = resolve_named_key(vault, &p.id, &name)?;
                    vault
                        .delete_key(&key.id)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?;
                    CommandOutput::new(
                        json!({ "deleted": key.id }),
                        format!("deleted key: {} ({})", key.name, key.id),
                    )
                }
            }
        },
        VaultCmd::Token(cmd) => match cmd {
            TokenCmd::Add {
                project,
                name,
                token,
            } => {
                let p = resolve_project_selector(vault, &project)?;
                let token = read_input(&token)?;
                let t = vault
                    .add_token(TokenEntryInput {
                        project_id: p.id,
                        name,
                        token,
                    })
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                CommandOutput::new(
                    json!({ "token": t }),
                    format!("created token: {} ({})", t.name, t.id),
                )
            }
            TokenCmd::List { project, details } => {
                let p = resolve_project_selector(vault, &project)?;
                let tokens = vault
                    .list_tokens(Some(&p.id))
                    .map_err(|e| AppError::invalid_key(e.to_string()))?;
                let mut lines = Vec::new();
                for t in &tokens {
                    let line = if details {
                        format!("{}  {}  created_at={}", t.id, t.name, t.created_at)
                    } else {
                        format!("{}  {}", t.id, t.name)
                    };
                    lines.push(line);
                }
                CommandOutput::new(json!({ "tokens": tokens }), lines.join("\n"))
            }
            TokenCmd::Delete { id, project, name } => {
                if id.is_some() && (project.is_some() || name.is_some()) {
                    return Err(AppError::invalid_key(
                        "provide either a token id or --project/--name".to_string(),
                    ));
                }
                if let Some(id) = id {
                    vault
                        .delete_token(&id)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?;
                    CommandOutput::new(json!({ "deleted": id }), format!("deleted token: {id}"))
                } else {
                    let project = project.ok_or_else(|| {
                        AppError::invalid_key("provide --project with --name".to_string())
                    })?;
                    let name = name.ok_or_else(|| {
                        AppError::invalid_key("provide --name (or delete by id)".to_string())
                    })?;
                    let p = resolve_project_selector(vault, &project)?;
                    let token = resolve_named_token(vault, &p.id, &name)?;
                    vault
                        .delete_token(&token.id)
                        .map_err(|e| AppError::invalid_key(e.to_string()))?;
                    CommandOutput::new(
                        json!({ "deleted": token.id }),
                        format!("deleted token: {} ({})", token.name, token.id),
                    )
                }
            }
        },
        VaultCmd::Export { out, passphrase } => {
            let passphrase = read_input(&passphrase)?;
            let bundle = vault
                .export_bundle(&passphrase)
                .map_err(|e| AppError::invalid_key(e.to_string()))?;
            let bundle_value = serde_json::to_value(&bundle)
                .map_err(|e| AppError::internal(format!("serialize bundle: {e}")))?;
            let bundle_json = serde_json::to_string_pretty(&bundle)
                .map_err(|e| AppError::internal(format!("serialize bundle: {e}")))?;

            if let Some(path) = out {
                std::fs::write(&path, bundle_json.as_bytes())
                    .map_err(|e| AppError::internal(format!("failed to write {path:?}: {e}")))?;
                CommandOutput::new(
                    json!({ "path": path }),
                    format!("exported vault to {}", path.display()),
                )
            } else {
                CommandOutput::new(json!({ "bundle": bundle_value }), bundle_json)
            }
        }
        VaultCmd::Import {
            bundle,
            passphrase,
            replace,
        } => {
            let passphrase = read_input(&passphrase)?;
            let raw = read_input(&bundle)?;
            let parsed: ExportBundle = serde_json::from_str(&raw)
                .map_err(|e| AppError::invalid_key(format!("invalid bundle JSON: {e}")))?;
            vault
                .import_bundle(&parsed, &passphrase, replace)
                .map_err(|e| AppError::invalid_key(e.to_string()))?;
            CommandOutput::new(json!({ "imported": true }), "imported vault".to_string())
        }
    };
    Ok(out)
}
