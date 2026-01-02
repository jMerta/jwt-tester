use crate::error::{AppError, AppResult};
use crate::vault::{KeyEntry, ProjectEntry, Vault};
use jsonwebtoken::Algorithm;

pub(super) fn expected_kind(alg: Algorithm) -> String {
    match alg {
        Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => "hmac".to_string(),
        Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::PS256
        | Algorithm::PS384
        | Algorithm::PS512 => "rsa".to_string(),
        Algorithm::ES256 | Algorithm::ES384 => "ec".to_string(),
        Algorithm::EdDSA => "eddsa".to_string(),
    }
}

pub(super) fn resolve_project_keys(
    vault: &Vault,
    project_name: &str,
    key_id: &Option<String>,
    key_name: &Option<String>,
    token_kid: Option<String>,
    try_all: bool,
) -> AppResult<(ProjectEntry, Vec<KeyEntry>)> {
    let project = vault
        .find_project_by_name(project_name)
        .map_err(|e| AppError::invalid_key(e.to_string()))?
        .ok_or_else(|| AppError::invalid_key(format!("project not found: {project_name}")))?;

    let keys = vault
        .list_keys(Some(&project.id))
        .map_err(|e| AppError::invalid_key(e.to_string()))?;
    if keys.is_empty() {
        return Err(AppError::invalid_key("project has no keys"));
    }

    if let Some(id) = key_id {
        let k = keys
            .iter()
            .find(|k| &k.id == id)
            .cloned()
            .ok_or_else(|| AppError::invalid_key("key id not found in project"))?;
        return Ok((project, vec![k]));
    }

    if let Some(name) = key_name {
        let k = keys
            .iter()
            .find(|k| &k.name == name)
            .cloned()
            .ok_or_else(|| AppError::invalid_key("key name not found in project"))?;
        return Ok((project, vec![k]));
    }

    if let Some(kid) = token_kid.as_deref() {
        let matches: Vec<_> = keys
            .iter()
            .filter(|k| k.kid.as_deref() == Some(kid))
            .cloned()
            .collect();
        if matches.len() == 1 {
            let selected = matches[0].clone();
            if try_all {
                let mut candidates = vec![selected.clone()];
                for k in &keys {
                    if k.id != selected.id {
                        candidates.push(k.clone());
                    }
                }
                return Ok((project, candidates));
            }
            return Ok((project, vec![selected]));
        }
        if matches.len() > 1 {
            return Err(AppError::invalid_key(format!(
                "multiple keys match kid '{kid}'"
            )));
        }
        return Err(AppError::invalid_key(format!(
            "no key with kid '{kid}' found in project"
        )));
    }

    if let Some(default_id) = project.default_key_id.as_deref() {
        let default = keys
            .iter()
            .find(|k| k.id == default_id)
            .cloned()
            .ok_or_else(|| AppError::invalid_key("project default_key_id points to missing key"))?;
        if try_all {
            let mut candidates = vec![default.clone()];
            for k in &keys {
                if k.id != default.id {
                    candidates.push(k.clone());
                }
            }
            return Ok((project, candidates));
        }
        return Ok((project, vec![default]));
    }

    if keys.len() == 1 {
        return Ok((project, vec![keys[0].clone()]));
    }

    Err(AppError::invalid_key(format!(
        "project has {} keys and no default; specify --key-id/--key-name or set a default key",
        keys.len()
    )))
}

pub(super) fn resolve_project_key_single(
    vault: &Vault,
    project_name: &str,
    key_id: &Option<String>,
    key_name: &Option<String>,
) -> AppResult<(ProjectEntry, KeyEntry)> {
    let (project, keys) = resolve_project_keys(vault, project_name, key_id, key_name, None, false)?;
    Ok((project, keys.into_iter().next().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{KeyEntryInput, ProjectInput, Vault, VaultConfig};

    fn memory_vault() -> Vault {
        Vault::open(VaultConfig {
            no_persist: true,
            data_dir: None,
        })
        .expect("open vault")
    }

    fn add_project(vault: &Vault, name: &str) -> ProjectEntry {
        vault
            .add_project(ProjectInput {
                name: name.to_string(),
                description: None,
                tags: Vec::new(),
            })
            .expect("add project")
    }

    fn add_hmac_key(vault: &Vault, project_id: &str, name: &str, kid: Option<&str>) -> KeyEntry {
        vault
            .add_key(KeyEntryInput {
                project_id: project_id.to_string(),
                name: name.to_string(),
                kind: "hmac".to_string(),
                secret: "secret".to_string(),
                kid: kid.map(|v| v.to_string()),
                description: None,
                tags: Vec::new(),
            })
            .expect("add key")
    }

    #[test]
    fn expected_kind_maps_algorithms() {
        assert_eq!(expected_kind(Algorithm::HS256), "hmac");
        assert_eq!(expected_kind(Algorithm::RS256), "rsa");
        assert_eq!(expected_kind(Algorithm::ES256), "ec");
        assert_eq!(expected_kind(Algorithm::EdDSA), "eddsa");
    }

    #[test]
    fn resolve_project_keys_errors_when_project_missing() {
        let vault = memory_vault();
        let err = resolve_project_keys(&vault, "missing", &None, &None, None, false).unwrap_err();
        assert!(err.to_string().contains("project not found"));
    }

    #[test]
    fn resolve_project_keys_errors_when_no_keys() {
        let vault = memory_vault();
        add_project(&vault, "alpha");
        let err = resolve_project_keys(&vault, "alpha", &None, &None, None, false).unwrap_err();
        assert!(err.to_string().contains("project has no keys"));
    }

    #[test]
    fn resolve_project_keys_by_id_and_name() {
        let vault = memory_vault();
        let project = add_project(&vault, "alpha");
        let key1 = add_hmac_key(&vault, &project.id, "one", None);
        let key2 = add_hmac_key(&vault, &project.id, "two", None);

        let (_p, keys) =
            resolve_project_keys(&vault, "alpha", &Some(key1.id.clone()), &None, None, false)
                .expect("resolve by id");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key1.id);

        let (_p, keys) = resolve_project_keys(
            &vault,
            "alpha",
            &None,
            &Some(key2.name.clone()),
            None,
            false,
        )
        .expect("resolve by name");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key2.id);
    }

    #[test]
    fn resolve_project_keys_kid_selection_and_try_all() {
        let vault = memory_vault();
        let project = add_project(&vault, "alpha");
        let key1 = add_hmac_key(&vault, &project.id, "one", Some("kid1"));
        let key2 = add_hmac_key(&vault, &project.id, "two", Some("kid2"));

        let (_p, keys) = resolve_project_keys(
            &vault,
            "alpha",
            &None,
            &None,
            Some("kid1".to_string()),
            true,
        )
        .expect("resolve by kid");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].id, key1.id);

        let err = resolve_project_keys(
            &vault,
            "alpha",
            &None,
            &None,
            Some("missing".to_string()),
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("no key with kid"));

        let _ = add_hmac_key(&vault, &project.id, "dupe", Some("kid1"));
        let err = resolve_project_keys(
            &vault,
            "alpha",
            &None,
            &None,
            Some("kid1".to_string()),
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("multiple keys match kid"));
        assert_eq!(key2.name, "two");
    }

    #[test]
    fn resolve_project_keys_default_and_single_key() {
        let vault = memory_vault();
        let project = add_project(&vault, "alpha");
        let _key1 = add_hmac_key(&vault, &project.id, "one", None);
        let key2 = add_hmac_key(&vault, &project.id, "two", None);

        vault
            .set_default_key(&project.id, Some(&key2.id))
            .expect("set default key");

        let (_p, keys) = resolve_project_keys(&vault, "alpha", &None, &None, None, false)
            .expect("resolve default");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key2.id);

        let (_p, keys) = resolve_project_keys(&vault, "alpha", &None, &None, None, true)
            .expect("resolve default try all");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].id, key2.id);

        vault
            .set_default_key(&project.id, None)
            .expect("clear default key");
        let err = resolve_project_keys(&vault, "alpha", &None, &None, None, false).unwrap_err();
        assert!(err.to_string().contains("project has"));
    }

    #[test]
    fn resolve_project_keys_when_single_key_no_default() {
        let vault = memory_vault();
        add_project(&vault, "solo");
        let project = vault.find_project_by_name("solo").unwrap().unwrap();
        let key = add_hmac_key(&vault, &project.id, "only", None);

        let (_p, keys) = resolve_project_keys(&vault, "solo", &None, &None, None, false)
            .expect("resolve single key");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key.id);
    }
}
