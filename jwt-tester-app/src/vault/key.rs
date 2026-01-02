use super::helpers::{normalize_opt_string, normalize_tags, now_unix, parse_tags, serialize_tags};
use super::store::{Vault, VaultInner};
use super::types::{KeyEntry, KeyEntryInput};
use rusqlite::{params, Connection};
use uuid::Uuid;

impl Vault {
    pub fn list_keys(&self, project_id: Option<&str>) -> anyhow::Result<Vec<KeyEntry>> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let locked = state.lock().unwrap();
                let keys = locked.keys.clone();
                Ok(match project_id {
                    Some(pid) => keys.into_iter().filter(|k| k.project_id == pid).collect(),
                    None => keys,
                })
            }
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                let keys = if let Some(pid) = project_id {
                    let mut stmt = conn.prepare(
                        "SELECT id, project_id, name, kind, created_at, kid, description, tags FROM keys WHERE project_id = ?1 ORDER BY created_at DESC",
                    )?;
                    let rows = stmt.query_map(params![pid], |row| {
                        let tags = parse_tags(row.get(7)?);
                        Ok(KeyEntry {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            name: row.get(2)?,
                            kind: row.get(3)?,
                            created_at: row.get(4)?,
                            kid: row.get(5)?,
                            description: row.get(6)?,
                            tags,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                } else {
                    let mut stmt = conn.prepare(
                        "SELECT id, project_id, name, kind, created_at, kid, description, tags FROM keys ORDER BY created_at DESC",
                    )?;
                    let rows = stmt.query_map([], |row| {
                        let tags = parse_tags(row.get(7)?);
                        Ok(KeyEntry {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            name: row.get(2)?,
                            kind: row.get(3)?,
                            created_at: row.get(4)?,
                            kid: row.get(5)?,
                            description: row.get(6)?,
                            tags,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                };
                Ok(keys)
            }
        }
    }

    pub fn add_key(&self, input: KeyEntryInput) -> anyhow::Result<KeyEntry> {
        if input.project_id.trim().is_empty() {
            anyhow::bail!("project_id is required");
        }
        if input.secret.trim().is_empty() {
            anyhow::bail!("secret is required");
        }

        let id = Uuid::new_v4().to_string();
        let created_at = now_unix();

        let name = {
            let trimmed = input.name.trim();
            if trimmed.is_empty() {
                format!("key-{}", id.chars().take(8).collect::<String>())
            } else {
                trimmed.to_string()
            }
        };

        let kid = normalize_opt_string(input.kid);
        let description = normalize_opt_string(input.description);
        let tags = normalize_tags(input.tags);
        let tags_json = serialize_tags(&tags);

        let row = KeyEntry {
            id: id.clone(),
            project_id: input.project_id,
            name,
            kind: input.kind,
            created_at,
            kid,
            description,
            tags,
        };

        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.key_material.insert(row.id.clone(), input.secret);
                locked.keys.push(row.clone());
            }
            VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            } => {
                let account = format!("key:{id}");
                keychain.set_password(keychain_service, &account, &input.secret)?;

                let conn = Connection::open(db_path)?;
                conn.execute(
                    "INSERT INTO keys (id, project_id, name, kind, created_at, kid, description, tags, keychain_service, keychain_account) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        row.id,
                        row.project_id,
                        row.name,
                        row.kind,
                        row.created_at,
                        row.kid,
                        row.description,
                        tags_json,
                        keychain_service,
                        account
                    ],
                )?;
            }
        }

        Ok(row)
    }

    pub fn find_key_in_project(
        &self,
        project_id: &str,
        key_name: &str,
    ) -> anyhow::Result<Option<KeyEntry>> {
        let key_name = key_name.trim();
        if key_name.is_empty() {
            return Ok(None);
        }
        let keys = self.list_keys(Some(project_id))?;
        Ok(keys.into_iter().find(|k| k.name == key_name))
    }

    pub fn get_key_material(&self, key_id: &str) -> anyhow::Result<String> {
        match &self.inner {
            VaultInner::Memory { state } => state
                .lock()
                .unwrap()
                .key_material
                .get(key_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("key material not found")),
            VaultInner::Sqlite {
                db_path, keychain, ..
            } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn
                    .prepare("SELECT keychain_service, keychain_account FROM keys WHERE id = ?1")?;
                let (service, account): (String, String) =
                    stmt.query_row(params![key_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
                keychain.get_password(&service, &account)
            }
        }
    }

    pub fn delete_key(&self, key_id: &str) -> anyhow::Result<()> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.keys.retain(|k| k.id != key_id);
                locked.key_material.remove(key_id);
                for p in &mut locked.projects {
                    if p.default_key_id.as_deref() == Some(key_id) {
                        p.default_key_id = None;
                    }
                }
                Ok(())
            }
            VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare("SELECT keychain_account FROM keys WHERE id = ?1")?;
                let account: String = stmt.query_row(params![key_id], |row| row.get(0))?;
                let _ = keychain.delete_password(keychain_service, &account);

                conn.execute("DELETE FROM keys WHERE id = ?1", params![key_id])?;
                conn.execute(
                    "UPDATE projects SET default_key_id = NULL WHERE default_key_id = ?1",
                    params![key_id],
                )?;
                Ok(())
            }
        }
    }
}
