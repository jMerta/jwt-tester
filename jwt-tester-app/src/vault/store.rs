use super::helpers::default_data_dir;
use super::keychain::KeychainStore;
use super::keychain::OsKeychain;
use super::keychain_file::FileKeychain;
use super::sqlite::init_sqlite;
use super::types::{KeyEntry, ProjectEntry, TokenEntry};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const DEFAULT_KEYCHAIN_SERVICE: &str = "jwt-tester";
const KEYCHAIN_BACKEND_ENV: &str = "JWT_TESTER_KEYCHAIN_BACKEND";
const KEYCHAIN_PASSPHRASE_ENV: &str = "JWT_TESTER_KEYCHAIN_PASSPHRASE";
const KEYCHAIN_DIR_ENV: &str = "JWT_TESTER_KEYCHAIN_DIR";
const KEYCHAIN_DOCKER_ENV: &str = "JWT_TESTER_DOCKER";
const KEYCHAIN_DOCKER_TEST_ENV: &str = "JWT_TESTER_DOCKER_TEST";

#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub no_persist: bool,
    pub data_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Vault {
    pub(super) inner: VaultInner,
}

#[derive(Clone)]
pub(super) enum VaultInner {
    Memory {
        state: Arc<Mutex<MemoryState>>,
    },
    Sqlite {
        db_path: PathBuf,
        keychain_service: String,
        keychain: Arc<dyn KeychainStore>,
    },
}

#[derive(Default)]
pub(super) struct MemoryState {
    pub(super) projects: Vec<ProjectEntry>,
    pub(super) keys: Vec<KeyEntry>,
    pub(super) tokens: Vec<TokenEntry>,
    pub(super) key_material: HashMap<String, String>,
    pub(super) token_material: HashMap<String, String>,
}

impl Vault {
    pub fn open(cfg: VaultConfig) -> anyhow::Result<Self> {
        if cfg.no_persist {
            return Ok(Vault {
                inner: VaultInner::Memory {
                    state: Arc::new(Mutex::new(MemoryState::default())),
                },
            });
        }

        let data_dir = resolve_data_dir(&cfg)?;
        let keychain_service = std::env::var("JWT_TESTER_KEYCHAIN_SERVICE")
            .unwrap_or_else(|_| DEFAULT_KEYCHAIN_SERVICE.to_string());
        let keychain = resolve_keychain(&data_dir)?;
        Self::open_with_data_dir(keychain, keychain_service, data_dir)
    }

    #[cfg(test)]
    pub(crate) fn open_with(
        cfg: VaultConfig,
        keychain: Arc<dyn KeychainStore>,
        keychain_service: String,
    ) -> anyhow::Result<Self> {
        if cfg.no_persist {
            return Ok(Vault {
                inner: VaultInner::Memory {
                    state: Arc::new(Mutex::new(MemoryState::default())),
                },
            });
        }

        let data_dir = resolve_data_dir(&cfg)?;
        Self::open_with_data_dir(keychain, keychain_service, data_dir)
    }

    fn open_with_data_dir(
        keychain: Arc<dyn KeychainStore>,
        keychain_service: String,
        data_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("vault.sqlite3");
        init_sqlite(&db_path)?;

        Ok(Vault {
            inner: VaultInner::Sqlite {
                db_path,
                keychain_service,
                keychain,
            },
        })
    }
}

fn resolve_data_dir(cfg: &VaultConfig) -> anyhow::Result<PathBuf> {
    cfg.data_dir
        .clone()
        .or_else(default_data_dir)
        .ok_or_else(|| anyhow::anyhow!("Unable to determine default data dir"))
}

fn resolve_keychain(data_dir: &Path) -> anyhow::Result<Arc<dyn KeychainStore>> {
    let backend = std::env::var(KEYCHAIN_BACKEND_ENV).unwrap_or_else(|_| "os".to_string());
    let passphrase = std::env::var(KEYCHAIN_PASSPHRASE_ENV).ok();
    let root = std::env::var(KEYCHAIN_DIR_ENV).ok().map(PathBuf::from);
    let allow_file_backend = is_docker_environment();
    resolve_keychain_from(&backend, passphrase, root, data_dir, allow_file_backend)
}

fn resolve_keychain_from(
    backend: &str,
    passphrase: Option<String>,
    root: Option<PathBuf>,
    data_dir: &Path,
    allow_file_backend: bool,
) -> anyhow::Result<Arc<dyn KeychainStore>> {
    let backend = backend.trim().to_lowercase();
    match backend.as_str() {
        "os" => Ok(Arc::new(OsKeychain::new())),
        "file" => {
            if !allow_file_backend {
                anyhow::bail!(
                    "file keychain backend is only supported in Docker (set {KEYCHAIN_DOCKER_ENV}=1)"
                );
            }
            let passphrase = passphrase.ok_or_else(|| {
                anyhow::anyhow!("{KEYCHAIN_PASSPHRASE_ENV} must be set for file keychain")
            })?;
            let root = root.unwrap_or_else(|| data_dir.join("keychain"));
            Ok(Arc::new(FileKeychain::new(root, passphrase)?))
        }
        other => Err(anyhow::anyhow!(
            "unsupported keychain backend '{other}' (use 'os' or 'file')"
        )),
    }
}

fn is_docker_environment() -> bool {
    is_docker_environment_with(Path::new("/.dockerenv"))
}

fn is_docker_environment_with(marker: &Path) -> bool {
    let docker_env = env_flag_set(KEYCHAIN_DOCKER_ENV);
    if !docker_env {
        return false;
    }

    if marker.exists() {
        return true;
    }

    if cfg!(debug_assertions) && env_flag_set(KEYCHAIN_DOCKER_TEST_ENV) {
        return true;
    }

    false
}

fn env_flag_set(name: &str) -> bool {
    std::env::var(name)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{is_docker_environment_with, resolve_keychain_from};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn docker_env_requires_flag_and_marker() {
        let original = std::env::var(super::KEYCHAIN_DOCKER_ENV).ok();
        let original_test = std::env::var(super::KEYCHAIN_DOCKER_TEST_ENV).ok();
        let dir = TempDir::new().expect("temp dir");
        let marker = dir.path().join("dockerenv");
        fs::write(&marker, "1").expect("write marker");

        std::env::set_var(super::KEYCHAIN_DOCKER_ENV, "1");
        assert!(is_docker_environment_with(&marker));
        assert!(!is_docker_environment_with(&dir.path().join("missing")));

        std::env::remove_var(super::KEYCHAIN_DOCKER_ENV);
        assert!(!is_docker_environment_with(&marker));

        if let Some(value) = original {
            std::env::set_var(super::KEYCHAIN_DOCKER_ENV, value);
        }
        if let Some(value) = original_test {
            std::env::set_var(super::KEYCHAIN_DOCKER_TEST_ENV, value);
        }
    }

    #[test]
    fn docker_env_allows_test_override_in_debug() {
        if !cfg!(debug_assertions) {
            return;
        }
        let original = std::env::var(super::KEYCHAIN_DOCKER_ENV).ok();
        let original_test = std::env::var(super::KEYCHAIN_DOCKER_TEST_ENV).ok();
        let dir = TempDir::new().expect("temp dir");
        let marker = dir.path().join("missing");

        std::env::set_var(super::KEYCHAIN_DOCKER_ENV, "1");
        std::env::set_var(super::KEYCHAIN_DOCKER_TEST_ENV, "1");
        assert!(is_docker_environment_with(&marker));

        std::env::remove_var(super::KEYCHAIN_DOCKER_ENV);
        std::env::remove_var(super::KEYCHAIN_DOCKER_TEST_ENV);
        assert!(!is_docker_environment_with(&marker));

        if let Some(value) = original {
            std::env::set_var(super::KEYCHAIN_DOCKER_ENV, value);
        }
        if let Some(value) = original_test {
            std::env::set_var(super::KEYCHAIN_DOCKER_TEST_ENV, value);
        }
    }

    #[test]
    fn resolve_keychain_file_rejects_without_docker_flag() {
        let dir = TempDir::new().expect("temp dir");
        let err = resolve_keychain_from("file", None, None, dir.path(), false)
            .err()
            .expect("missing");
        assert!(err.to_string().contains("only supported in Docker"));
    }

    #[test]
    fn resolve_keychain_file_requires_passphrase() {
        let dir = TempDir::new().expect("temp dir");
        let err = resolve_keychain_from("file", None, None, dir.path(), true)
            .err()
            .expect("missing");
        assert!(err.to_string().contains("JWT_TESTER_KEYCHAIN_PASSPHRASE"));
    }

    #[test]
    fn resolve_keychain_file_defaults_to_data_dir() {
        let dir = TempDir::new().expect("temp dir");
        let keychain = resolve_keychain_from(
            "file",
            Some("passphrase".to_string()),
            None,
            dir.path(),
            true,
        )
        .expect("file keychain");
        keychain.set_password("svc", "acct", "secret").expect("set");
        let kc_dir = dir.path().join("keychain");
        let count = fs::read_dir(&kc_dir).expect("read keychain dir").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn resolve_keychain_rejects_unknown_backend() {
        let dir = TempDir::new().expect("temp dir");
        let err = resolve_keychain_from("nope", None, None, dir.path(), true)
            .err()
            .expect("unknown");
        assert!(err.to_string().contains("unsupported keychain backend"));
    }
}
