use crate::vault_export;
use std::collections::{HashMap, HashSet};

pub(super) fn validate_snapshot(snapshot: &vault_export::VaultSnapshot) -> anyhow::Result<()> {
    if snapshot.version != vault_export::EXPORT_VERSION {
        anyhow::bail!("unsupported snapshot version {}", snapshot.version);
    }

    let mut project_ids = HashSet::new();
    let mut project_names = HashSet::new();
    for project in &snapshot.projects {
        if !project_ids.insert(project.id.as_str()) {
            anyhow::bail!("duplicate project id {}", project.id);
        }
        if !project_names.insert(project.name.as_str()) {
            anyhow::bail!("duplicate project name {}", project.name);
        }
    }

    let mut key_ids = HashSet::new();
    let mut key_project = HashMap::new();
    for key in &snapshot.keys {
        if !key_ids.insert(key.entry.id.as_str()) {
            anyhow::bail!("duplicate key id {}", key.entry.id);
        }
        if !project_ids.contains(key.entry.project_id.as_str()) {
            anyhow::bail!(
                "key {} references unknown project {}",
                key.entry.id,
                key.entry.project_id
            );
        }
        key_project.insert(key.entry.id.as_str(), key.entry.project_id.as_str());
    }

    let mut token_ids = HashSet::new();
    for token in &snapshot.tokens {
        if !token_ids.insert(token.entry.id.as_str()) {
            anyhow::bail!("duplicate token id {}", token.entry.id);
        }
        if !project_ids.contains(token.entry.project_id.as_str()) {
            anyhow::bail!(
                "token {} references unknown project {}",
                token.entry.id,
                token.entry.project_id
            );
        }
    }

    for project in &snapshot.projects {
        if let Some(default_id) = project.default_key_id.as_deref() {
            let Some(project_id) = key_project.get(default_id) else {
                anyhow::bail!(
                    "project {} default_key_id {} not found",
                    project.name,
                    default_id
                );
            };
            if *project_id != project.id.as_str() {
                anyhow::bail!(
                    "project {} default_key_id {} belongs to a different project",
                    project.name,
                    default_id
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_snapshot;
    use crate::vault::{KeyEntry, ProjectEntry, TokenEntry};
    use crate::vault_export::{KeyExport, TokenExport, VaultSnapshot, EXPORT_VERSION};

    fn base_snapshot() -> VaultSnapshot {
        VaultSnapshot {
            version: EXPORT_VERSION,
            exported_at: 1,
            projects: vec![ProjectEntry {
                id: "p1".to_string(),
                name: "alpha".to_string(),
                created_at: 1,
                default_key_id: None,
                description: None,
                tags: vec![],
            }],
            keys: vec![KeyExport {
                entry: KeyEntry {
                    id: "k1".to_string(),
                    project_id: "p1".to_string(),
                    name: "key".to_string(),
                    kind: "hmac".to_string(),
                    created_at: 1,
                    kid: None,
                    description: None,
                    tags: vec![],
                },
                material: "secret".to_string(),
            }],
            tokens: vec![TokenExport {
                entry: TokenEntry {
                    id: "t1".to_string(),
                    project_id: "p1".to_string(),
                    name: "tok".to_string(),
                    created_at: 1,
                },
                token: "token".to_string(),
            }],
        }
    }

    #[test]
    fn validate_snapshot_rejects_wrong_version() {
        let mut snapshot = base_snapshot();
        snapshot.version = 99;
        let err = validate_snapshot(&snapshot).expect_err("expected error");
        assert!(err.to_string().contains("unsupported snapshot version"));
    }

    #[test]
    fn validate_snapshot_rejects_duplicate_project_id() {
        let mut snapshot = base_snapshot();
        snapshot.projects.push(snapshot.projects[0].clone());
        let err = validate_snapshot(&snapshot).expect_err("expected error");
        assert!(err.to_string().contains("duplicate project id"));
    }

    #[test]
    fn validate_snapshot_rejects_key_with_unknown_project() {
        let mut snapshot = base_snapshot();
        snapshot.keys[0].entry.project_id = "missing".to_string();
        let err = validate_snapshot(&snapshot).expect_err("expected error");
        assert!(err.to_string().contains("references unknown project"));
    }

    #[test]
    fn validate_snapshot_rejects_default_key_mismatch() {
        let mut snapshot = base_snapshot();
        snapshot.projects.push(ProjectEntry {
            id: "p2".to_string(),
            name: "bravo".to_string(),
            created_at: 1,
            default_key_id: None,
            description: None,
            tags: vec![],
        });
        snapshot.projects[0].default_key_id = Some("k1".to_string());
        snapshot.keys[0].entry.project_id = "p2".to_string();
        let err = validate_snapshot(&snapshot).expect_err("expected error");
        assert!(err.to_string().contains("belongs to a different project"));
    }
}
