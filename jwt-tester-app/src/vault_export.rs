use crate::vault::{KeyEntry, ProjectEntry, TokenEntry};
use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const EXPORT_VERSION: u8 = 1;
const KDF_NAME: &str = "argon2id";
const CIPHER_NAME: &str = "xchacha20poly1305";
const KDF_MEM_KIB: u32 = 65_536;
const KDF_ITERATIONS: u32 = 3;
const KDF_PARALLELISM: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBundle {
    pub version: u8,
    pub kdf: KdfParams,
    pub cipher: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KdfParams {
    pub name: String,
    pub mem_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSnapshot {
    pub version: u8,
    pub exported_at: i64,
    pub projects: Vec<ProjectEntry>,
    pub keys: Vec<KeyExport>,
    pub tokens: Vec<TokenExport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyExport {
    pub entry: KeyEntry,
    pub material: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenExport {
    pub entry: TokenEntry,
    pub token: String,
}

pub fn build_snapshot(
    projects: Vec<ProjectEntry>,
    keys: Vec<KeyExport>,
    tokens: Vec<TokenExport>,
) -> VaultSnapshot {
    VaultSnapshot {
        version: EXPORT_VERSION,
        exported_at: now_unix(),
        projects,
        keys,
        tokens,
    }
}

pub fn encrypt_snapshot(
    snapshot: &VaultSnapshot,
    passphrase: &str,
) -> anyhow::Result<ExportBundle> {
    if passphrase.trim().is_empty() {
        anyhow::bail!("passphrase is required");
    }

    let plaintext = serde_json::to_vec(snapshot).context("serialize vault snapshot")?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let params = Params::new(KDF_MEM_KIB, KDF_ITERATIONS, KDF_PARALLELISM, None)
        .map_err(|e| anyhow::anyhow!("invalid kdf params: {e:?}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("derive key from passphrase: {e:?}"))?;

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow::anyhow!("encrypt vault snapshot: {e:?}"))?;

    Ok(ExportBundle {
        version: EXPORT_VERSION,
        kdf: KdfParams {
            name: KDF_NAME.to_string(),
            mem_kib: KDF_MEM_KIB,
            iterations: KDF_ITERATIONS,
            parallelism: KDF_PARALLELISM,
            salt: URL_SAFE_NO_PAD.encode(salt),
        },
        cipher: CIPHER_NAME.to_string(),
        nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
        ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
    })
}

pub fn decrypt_snapshot(bundle: &ExportBundle, passphrase: &str) -> anyhow::Result<VaultSnapshot> {
    if bundle.version != EXPORT_VERSION {
        anyhow::bail!("unsupported export version {}", bundle.version);
    }
    if bundle.kdf.name != KDF_NAME {
        anyhow::bail!("unsupported kdf {}", bundle.kdf.name);
    }
    if bundle.cipher != CIPHER_NAME {
        anyhow::bail!("unsupported cipher {}", bundle.cipher);
    }
    if passphrase.trim().is_empty() {
        anyhow::bail!("passphrase is required");
    }

    let salt = URL_SAFE_NO_PAD
        .decode(&bundle.kdf.salt)
        .context("decode salt")?;
    let nonce = URL_SAFE_NO_PAD
        .decode(&bundle.nonce)
        .context("decode nonce")?;
    let ciphertext = URL_SAFE_NO_PAD
        .decode(&bundle.ciphertext)
        .context("decode ciphertext")?;

    let params = Params::new(
        bundle.kdf.mem_kib,
        bundle.kdf.iterations,
        bundle.kdf.parallelism,
        None,
    )
    .map_err(|e| anyhow::anyhow!("invalid kdf params: {e:?}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("derive key from passphrase: {e:?}"))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = XNonce::from_slice(&nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("decrypt vault snapshot: {e:?}"))?;

    let snapshot: VaultSnapshot =
        serde_json::from_slice(&plaintext).context("parse vault snapshot")?;
    if snapshot.version != EXPORT_VERSION {
        anyhow::bail!("unsupported snapshot version {}", snapshot.version);
    }
    Ok(snapshot)
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{KeyEntry, ProjectEntry, TokenEntry};

    #[test]
    fn export_encrypt_decrypt_roundtrip() {
        let snapshot = VaultSnapshot {
            version: EXPORT_VERSION,
            exported_at: 123,
            projects: vec![ProjectEntry {
                id: "p1".to_string(),
                name: "alpha".to_string(),
                created_at: 123,
                default_key_id: None,
                description: Some("desc".to_string()),
                tags: vec!["tag".to_string()],
            }],
            keys: vec![KeyExport {
                entry: KeyEntry {
                    id: "k1".to_string(),
                    project_id: "p1".to_string(),
                    name: "key".to_string(),
                    kind: "hmac".to_string(),
                    created_at: 123,
                    kid: Some("kid".to_string()),
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
                    created_at: 123,
                },
                token: "token".to_string(),
            }],
        };

        let bundle = encrypt_snapshot(&snapshot, "passphrase").expect("encrypt");
        let decoded = decrypt_snapshot(&bundle, "passphrase").expect("decrypt");
        assert_eq!(decoded.projects.len(), 1);
        assert_eq!(decoded.keys.len(), 1);
        assert_eq!(decoded.tokens.len(), 1);
        assert_eq!(decoded.projects[0].name, "alpha");
        assert_eq!(decoded.keys[0].material, "secret");
    }

    #[test]
    fn decrypt_rejects_wrong_passphrase() {
        let snapshot = VaultSnapshot {
            version: EXPORT_VERSION,
            exported_at: 1,
            projects: vec![],
            keys: vec![],
            tokens: vec![],
        };
        let bundle = encrypt_snapshot(&snapshot, "good").expect("encrypt");
        let err = decrypt_snapshot(&bundle, "bad");
        assert!(err.is_err());
    }
}
