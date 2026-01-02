use rusqlite::Connection;
use std::path::Path;

pub(super) fn init_sqlite(path: &Path) -> anyhow::Result<()> {
    let conn = Connection::open(path)?;

    // If an older schema exists (projects had a NOT NULL `domain` column), fail fast with an actionable message.
    // This scaffold is still evolving; the simplest upgrade path is to delete the local DB.
    let has_domain_col: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'domain'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if has_domain_col > 0 {
        anyhow::bail!(
            "Detected an older vault schema (projects had a `domain` column). Delete the local vault DB (vault.sqlite3) to recreate it with the new project-only schema."
        );
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            default_key_id TEXT NULL,
            description TEXT NULL,
            tags TEXT NULL,
            UNIQUE(name)
        )",
        [],
    )?;

    // Add columns for existing DBs created before new fields were introduced.
    ensure_column(
        &conn,
        "projects",
        "default_key_id",
        "ALTER TABLE projects ADD COLUMN default_key_id TEXT NULL",
    )?;
    ensure_column(
        &conn,
        "projects",
        "description",
        "ALTER TABLE projects ADD COLUMN description TEXT NULL",
    )?;
    ensure_column(
        &conn,
        "projects",
        "tags",
        "ALTER TABLE projects ADD COLUMN tags TEXT NULL",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS keys (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            kid TEXT NULL,
            description TEXT NULL,
            tags TEXT NULL,
            keychain_service TEXT NOT NULL,
            keychain_account TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    ensure_column(
        &conn,
        "keys",
        "kid",
        "ALTER TABLE keys ADD COLUMN kid TEXT NULL",
    )?;
    ensure_column(
        &conn,
        "keys",
        "description",
        "ALTER TABLE keys ADD COLUMN description TEXT NULL",
    )?;
    ensure_column(
        &conn,
        "keys",
        "tags",
        "ALTER TABLE keys ADD COLUMN tags TEXT NULL",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tokens (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            keychain_service TEXT NOT NULL,
            keychain_account TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

pub(super) fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    ddl: &str,
) -> anyhow::Result<()> {
    let query =
        format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = '{column}'");
    let count: i64 = conn.query_row(&query, [], |row| row.get(0)).unwrap_or(0);
    if count == 0 {
        conn.execute(ddl, [])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn init_sqlite_creates_tables_and_columns() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("vault.sqlite3");

        init_sqlite(&path).expect("init sqlite");
        let conn = Connection::open(&path).expect("open sqlite");

        let project_cols: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('projects')")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(project_cols.contains(&"default_key_id".to_string()));
        assert!(project_cols.contains(&"description".to_string()));
        assert!(project_cols.contains(&"tags".to_string()));

        let key_cols: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('keys')")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(key_cols.contains(&"kid".to_string()));
        assert!(key_cols.contains(&"description".to_string()));
        assert!(key_cols.contains(&"tags".to_string()));

        let token_cols: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('tokens')")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(token_cols.contains(&"keychain_account".to_string()));
    }

    #[test]
    fn init_sqlite_rejects_legacy_domain_schema() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("vault.sqlite3");
        let conn = Connection::open(&path).expect("open sqlite");
        conn.execute(
            "CREATE TABLE projects (id TEXT PRIMARY KEY, domain TEXT NOT NULL)",
            [],
        )
        .expect("create legacy table");
        drop(conn);

        let err = init_sqlite(&path).expect_err("expected legacy schema error");
        assert!(err.to_string().contains("older vault schema"));
    }
}
