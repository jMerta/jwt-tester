use super::helpers::{normalize_opt_string, normalize_tags, now_unix, parse_tags, serialize_tags};
use super::store::{Vault, VaultInner};
use super::types::{ProjectEntry, ProjectInput};
use rusqlite::{params, Connection};
use uuid::Uuid;

impl Vault {
    pub fn list_projects(&self) -> anyhow::Result<Vec<ProjectEntry>> {
        match &self.inner {
            VaultInner::Memory { state } => Ok(state.lock().unwrap().projects.clone()),
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare(
                    "SELECT id, name, created_at, default_key_id, description, tags FROM projects ORDER BY created_at DESC",
                )?;
                let rows = stmt.query_map([], |row| {
                    let tags = parse_tags(row.get(5)?);
                    Ok(ProjectEntry {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        default_key_id: row.get(3)?,
                        description: row.get(4)?,
                        tags,
                    })
                })?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            }
        }
    }

    pub fn add_project(&self, input: ProjectInput) -> anyhow::Result<ProjectEntry> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            anyhow::bail!("project name is required");
        }

        let description = normalize_opt_string(input.description);
        let tags = normalize_tags(input.tags);
        let tags_json = serialize_tags(&tags);

        let row = ProjectEntry {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: now_unix(),
            default_key_id: None,
            description,
            tags,
        };

        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                if locked.projects.iter().any(|p| p.name == row.name) {
                    anyhow::bail!("project already exists");
                }
                locked.projects.push(row.clone());
            }
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                conn.execute(
                    "INSERT INTO projects (id, name, created_at, default_key_id, description, tags) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
                    params![row.id, row.name, row.created_at, row.description, tags_json],
                )?;
            }
        }

        Ok(row)
    }

    pub fn find_project(&self, name: &str) -> anyhow::Result<Option<ProjectEntry>> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(None);
        }

        match &self.inner {
            VaultInner::Memory { state } => Ok(state
                .lock()
                .unwrap()
                .projects
                .iter()
                .find(|p| p.name == name)
                .cloned()),
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare(
                    "SELECT id, name, created_at, default_key_id, description, tags FROM projects WHERE name = ?1",
                )?;
                let result = stmt.query_row(params![name], |row| {
                    let tags = parse_tags(row.get(5)?);
                    Ok(ProjectEntry {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        default_key_id: row.get(3)?,
                        description: row.get(4)?,
                        tags,
                    })
                });
                match result {
                    Ok(p) => Ok(Some(p)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    pub fn set_default_key(&self, project_id: &str, key_id: Option<&str>) -> anyhow::Result<()> {
        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                let project = locked
                    .projects
                    .iter_mut()
                    .find(|p| p.id == project_id)
                    .ok_or_else(|| anyhow::anyhow!("project not found"))?;
                project.default_key_id = key_id.map(|s| s.to_string());
                Ok(())
            }
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                conn.execute(
                    "UPDATE projects SET default_key_id = ?1 WHERE id = ?2",
                    params![key_id, project_id],
                )?;
                Ok(())
            }
        }
    }

    pub fn delete_project(&self, project_id: &str) -> anyhow::Result<()> {
        let keys = self.list_keys(Some(project_id))?;
        for k in keys {
            let _ = self.delete_key(&k.id);
        }
        let tokens = self.list_tokens(Some(project_id))?;
        for t in tokens {
            let _ = self.delete_token(&t.id);
        }

        match &self.inner {
            VaultInner::Memory { state } => {
                let mut locked = state.lock().unwrap();
                locked.projects.retain(|p| p.id != project_id);
            }
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
            }
        }

        Ok(())
    }

    pub fn find_project_by_name(&self, name: &str) -> anyhow::Result<Option<ProjectEntry>> {
        self.find_project(name)
    }

    pub fn find_project_by_id(&self, id: &str) -> anyhow::Result<Option<ProjectEntry>> {
        let id = id.trim();
        if id.is_empty() {
            return Ok(None);
        }

        match &self.inner {
            VaultInner::Memory { state } => Ok(state
                .lock()
                .unwrap()
                .projects
                .iter()
                .find(|p| p.id == id)
                .cloned()),
            VaultInner::Sqlite { db_path, .. } => {
                let conn = Connection::open(db_path)?;
                let mut stmt = conn.prepare(
                    "SELECT id, name, created_at, default_key_id, description, tags FROM projects WHERE id = ?1",
                )?;
                let result = stmt.query_row(params![id], |row| {
                    let tags = parse_tags(row.get(5)?);
                    Ok(ProjectEntry {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        default_key_id: row.get(3)?,
                        description: row.get(4)?,
                        tags,
                    })
                });
                match result {
                    Ok(p) => Ok(Some(p)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
        }
    }
}
