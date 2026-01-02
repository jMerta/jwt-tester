use super::helpers::serialize_tags;
use super::snapshot::validate_snapshot;
use super::store::{Vault, VaultInner};
use crate::vault_export;
use rusqlite::{params, Connection};

impl Vault {
    pub fn export_bundle(&self, passphrase: &str) -> anyhow::Result<vault_export::ExportBundle> {
        let projects = self.list_projects()?;
        let keys = self.list_keys(None)?;
        let tokens = self.list_tokens(None)?;

        let mut key_exports = Vec::with_capacity(keys.len());
        for key in keys {
            let material = self.get_key_material(&key.id)?;
            key_exports.push(vault_export::KeyExport {
                entry: key,
                material,
            });
        }

        let mut token_exports = Vec::with_capacity(tokens.len());
        for token in tokens {
            let material = self.get_token_material(&token.id)?;
            token_exports.push(vault_export::TokenExport {
                entry: token,
                token: material,
            });
        }

        let snapshot = vault_export::build_snapshot(projects, key_exports, token_exports);
        vault_export::encrypt_snapshot(&snapshot, passphrase)
    }

    pub fn import_bundle(
        &self,
        bundle: &vault_export::ExportBundle,
        passphrase: &str,
        replace: bool,
    ) -> anyhow::Result<()> {
        let snapshot = vault_export::decrypt_snapshot(bundle, passphrase)?;
        validate_snapshot(&snapshot)?;

        if replace {
            self.clear_all()?;
        } else if !self.is_empty()? {
            anyhow::bail!("vault is not empty; use --replace to overwrite");
        }

        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.projects = snapshot.projects.clone();
                locked.keys = snapshot.keys.iter().map(|k| k.entry.clone()).collect();
                locked.tokens = snapshot.tokens.iter().map(|t| t.entry.clone()).collect();
                locked.key_material = snapshot
                    .keys
                    .iter()
                    .map(|k| (k.entry.id.clone(), k.material.clone()))
                    .collect();
                locked.token_material = snapshot
                    .tokens
                    .iter()
                    .map(|t| (t.entry.id.clone(), t.token.clone()))
                    .collect();
            }
            VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            } => {
                let conn = Connection::open(db_path)?;
                for project in &snapshot.projects {
                    let tags_json = serialize_tags(&project.tags);
                    conn.execute(
                        "INSERT INTO projects (id, name, created_at, default_key_id, description, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            project.id,
                            project.name,
                            project.created_at,
                            project.default_key_id,
                            project.description,
                            tags_json
                        ],
                    )?;
                }

                for key in &snapshot.keys {
                    let account = format!("key:{}", key.entry.id);
                    keychain.set_password(keychain_service, &account, &key.material)?;

                    let tags_json = serialize_tags(&key.entry.tags);
                    let insert = conn.execute(
                        "INSERT INTO keys (id, project_id, name, kind, created_at, kid, description, tags, keychain_service, keychain_account) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            key.entry.id,
                            key.entry.project_id,
                            key.entry.name,
                            key.entry.kind,
                            key.entry.created_at,
                            key.entry.kid,
                            key.entry.description,
                            tags_json,
                            keychain_service,
                            account
                        ],
                    );
                    if let Err(err) = insert {
                        let _ = keychain.delete_password(keychain_service, &account);
                        return Err(err.into());
                    }
                }

                for token in &snapshot.tokens {
                    let account = format!("token:{}", token.entry.id);
                    keychain.set_password(keychain_service, &account, &token.token)?;

                    let insert = conn.execute(
                        "INSERT INTO tokens (id, project_id, name, created_at, keychain_service, keychain_account) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            token.entry.id,
                            token.entry.project_id,
                            token.entry.name,
                            token.entry.created_at,
                            keychain_service,
                            account
                        ],
                    );
                    if let Err(err) = insert {
                        let _ = keychain.delete_password(keychain_service, &account);
                        return Err(err.into());
                    }
                }
            }
        }

        Ok(())
    }

    fn is_empty(&self) -> anyhow::Result<bool> {
        Ok(self.list_projects()?.is_empty()
            && self.list_keys(None)?.is_empty()
            && self.list_tokens(None)?.is_empty())
    }

    fn clear_all(&self) -> anyhow::Result<()> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.projects.clear();
                locked.keys.clear();
                locked.tokens.clear();
                locked.key_material.clear();
                locked.token_material.clear();
            }
            VaultInner::Sqlite { .. } => {
                let projects = self.list_projects()?;
                for p in projects {
                    self.delete_project(&p.id)?;
                }
            }
        }
        Ok(())
    }
}
