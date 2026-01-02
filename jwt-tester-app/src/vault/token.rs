use super::store::{Vault, VaultInner};
use super::types::{TokenEntry, TokenEntryInput};
use rusqlite::{params, Connection};
use uuid::Uuid;

impl Vault {
    pub fn list_tokens(&self, project_id: Option<&str>) -> anyhow::Result<Vec<TokenEntry>> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let locked = state.lock().unwrap();
                let tokens = locked.tokens.clone();
                Ok(match project_id {
                    Some(pid) => tokens.into_iter().filter(|t| t.project_id == pid).collect(),
                    None => tokens,
                })
            }
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                let tokens = if let Some(pid) = project_id {
                    let mut stmt = conn.prepare(
                        "SELECT id, project_id, name, created_at FROM tokens WHERE project_id = ?1 ORDER BY created_at DESC",
                    )?;
                    let rows = stmt.query_map(params![pid], |row| {
                        Ok(TokenEntry {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            name: row.get(2)?,
                            created_at: row.get(3)?,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                } else {
                    let mut stmt = conn.prepare(
                        "SELECT id, project_id, name, created_at FROM tokens ORDER BY created_at DESC",
                    )?;
                    let rows = stmt.query_map([], |row| {
                        Ok(TokenEntry {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            name: row.get(2)?,
                            created_at: row.get(3)?,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                };
                Ok(tokens)
            }
        }
    }

    pub fn add_token(&self, input: TokenEntryInput) -> anyhow::Result<TokenEntry> {
        if input.project_id.trim().is_empty() {
            anyhow::bail!("project_id is required");
        }
        if input.name.trim().is_empty() {
            anyhow::bail!("name is required");
        }
        if input.token.trim().is_empty() {
            anyhow::bail!("token is required");
        }

        let id = Uuid::new_v4().to_string();
        let created_at = super::helpers::now_unix();

        let row = TokenEntry {
            id: id.clone(),
            project_id: input.project_id,
            name: input.name,
            created_at,
        };

        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.token_material.insert(row.id.clone(), input.token);
                locked.tokens.push(row.clone());
            }
            VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            } => {
                let account = format!("token:{id}");
                keychain.set_password(keychain_service, &account, &input.token)?;

                let conn = Connection::open(db_path)?;
                conn.execute(
                    "INSERT INTO tokens (id, project_id, name, created_at, keychain_service, keychain_account) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![row.id, row.project_id, row.name, row.created_at, keychain_service, account],
                )?;
            }
        }

        Ok(row)
    }

    pub fn get_token_material(&self, token_id: &str) -> anyhow::Result<String> {
        match &self.inner {
            VaultInner::Memory { state } => state
                .lock()
                .unwrap()
                .token_material
                .get(token_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("token material not found")),
            VaultInner::Sqlite {
                db_path, keychain, ..
            } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare(
                    "SELECT keychain_service, keychain_account FROM tokens WHERE id = ?1",
                )?;
                let (service, account): (String, String) =
                    stmt.query_row(params![token_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
                keychain.get_password(&service, &account)
            }
        }
    }

    pub fn delete_token(&self, token_id: &str) -> anyhow::Result<()> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.tokens.retain(|t| t.id != token_id);
                locked.token_material.remove(token_id);
                Ok(())
            }
            VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare("SELECT keychain_account FROM tokens WHERE id = ?1")?;
                let account: String = stmt.query_row(params![token_id], |row| row.get(0))?;
                let _ = keychain.delete_password(keychain_service, &account);

                conn.execute("DELETE FROM tokens WHERE id = ?1", params![token_id])?;
                Ok(())
            }
        }
    }
}
