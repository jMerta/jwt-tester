use super::keychain::KeychainStore;
use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_VERSION: u8 = 1;
const KDF_NAME: &str = "argon2id";
const CIPHER_NAME: &str = "xchacha20poly1305";
const KDF_MEM_KIB: u32 = 65_536;
const KDF_ITERATIONS: u32 = 3;
const KDF_PARALLELISM: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct KdfParams {
    name: String,
    mem_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedEntry {
    version: u8,
    kdf: KdfParams,
    cipher: String,
    nonce: String,
    ciphertext: String,
}

pub(crate) struct FileKeychain {
    root: PathBuf,
    passphrase: String,
}

impl FileKeychain {
    pub(crate) fn new(root: PathBuf, passphrase: String) -> anyhow::Result<Self> {
        if passphrase.trim().is_empty() {
            anyhow::bail!("keychain passphrase is required");
        }
        fs::create_dir_all(&root).with_context(|| format!("create keychain dir {:?}", root))?;
        Ok(Self { root, passphrase })
    }

    fn entry_path(&self, service: &str, account: &str) -> PathBuf {
        let mut key = String::with_capacity(service.len() + account.len() + 1);
        key.push_str(service);
        key.push('\0');
        key.push_str(account);
        let encoded = URL_SAFE_NO_PAD.encode(key.as_bytes());
        self.root.join(format!("{encoded}.json"))
    }

    fn read_entry(&self, path: &Path) -> anyhow::Result<EncryptedEntry> {
        let data = fs::read(path).with_context(|| format!("read keychain entry {:?}", path))?;
        serde_json::from_slice(&data).context("parse keychain entry")
    }

    fn write_entry(&self, path: &Path, entry: &EncryptedEntry) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(entry).context("serialize keychain entry")?;
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, payload)
            .with_context(|| format!("write keychain entry {:?}", tmp_path))?;
        if path.exists() {
            fs::remove_file(path).with_context(|| format!("replace keychain entry {:?}", path))?;
        }
        fs::rename(&tmp_path, path)
            .with_context(|| format!("persist keychain entry {:?}", path))?;
        Ok(())
    }
}

impl KeychainStore for FileKeychain {
    fn set_password(&self, service: &str, account: &str, secret: &str) -> anyhow::Result<()> {
        let path = self.entry_path(service, account);
        let entry = encrypt_secret(&self.passphrase, secret)?;
        self.write_entry(&path, &entry)?;
        Ok(())
    }

    fn get_password(&self, service: &str, account: &str) -> anyhow::Result<String> {
        let path = self.entry_path(service, account);
        if !path.exists() {
            return Err(anyhow::anyhow!("keychain entry not found"));
        }
        let entry = self.read_entry(&path)?;
        decrypt_secret(&self.passphrase, &entry)
    }

    fn delete_password(&self, service: &str, account: &str) -> anyhow::Result<()> {
        let path = self.entry_path(service, account);
        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| format!("delete keychain entry {:?}", path)),
        }
    }
}

fn encrypt_secret(passphrase: &str, secret: &str) -> anyhow::Result<EncryptedEntry> {
    if passphrase.trim().is_empty() {
        anyhow::bail!("keychain passphrase is required");
    }

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
        .encrypt(nonce, secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("encrypt keychain entry: {e:?}"))?;

    Ok(EncryptedEntry {
        version: ENTRY_VERSION,
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

fn decrypt_secret(passphrase: &str, entry: &EncryptedEntry) -> anyhow::Result<String> {
    if entry.version != ENTRY_VERSION {
        anyhow::bail!("unsupported keychain entry version {}", entry.version);
    }
    if entry.kdf.name != KDF_NAME {
        anyhow::bail!("unsupported kdf {}", entry.kdf.name);
    }
    if entry.cipher != CIPHER_NAME {
        anyhow::bail!("unsupported cipher {}", entry.cipher);
    }
    if passphrase.trim().is_empty() {
        anyhow::bail!("keychain passphrase is required");
    }

    let salt = URL_SAFE_NO_PAD
        .decode(&entry.kdf.salt)
        .context("decode salt")?;
    let nonce = URL_SAFE_NO_PAD
        .decode(&entry.nonce)
        .context("decode nonce")?;
    let ciphertext = URL_SAFE_NO_PAD
        .decode(&entry.ciphertext)
        .context("decode ciphertext")?;

    let params = Params::new(
        entry.kdf.mem_kib,
        entry.kdf.iterations,
        entry.kdf.parallelism,
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
        .map_err(|e| anyhow::anyhow!("decrypt keychain entry: {e:?}"))?;
    let secret = String::from_utf8(plaintext).context("decode keychain secret")?;
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::FileKeychain;
    use crate::vault::keychain::KeychainStore;
    use tempfile::TempDir;

    #[test]
    fn file_keychain_roundtrip() {
        let dir = TempDir::new().expect("temp dir");
        let keychain =
            FileKeychain::new(dir.path().join("kc"), "passphrase".to_string()).expect("keychain");
        keychain.set_password("svc", "acct", "secret").expect("set");
        let value = keychain.get_password("svc", "acct").expect("get");
        assert_eq!(value, "secret");
        keychain.delete_password("svc", "acct").expect("delete");
        assert!(keychain.get_password("svc", "acct").is_err());
    }

    #[test]
    fn file_keychain_rejects_wrong_passphrase() {
        let dir = TempDir::new().expect("temp dir");
        let keychain =
            FileKeychain::new(dir.path().join("kc"), "passphrase".to_string()).expect("keychain");
        keychain.set_password("svc", "acct", "secret").expect("set");
        let other =
            FileKeychain::new(dir.path().join("kc"), "wrong".to_string()).expect("keychain");
        let err = other.get_password("svc", "acct");
        assert!(err.is_err());
    }
}
