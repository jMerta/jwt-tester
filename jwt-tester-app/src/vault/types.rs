use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub default_key_id: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyEntry {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub created_at: i64,
    pub kid: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenEntry {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: i64,
}

pub struct ProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

pub struct KeyEntryInput {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub secret: String,
    pub kid: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

pub struct TokenEntryInput {
    pub project_id: String,
    pub name: String,
    pub token: String,
}
