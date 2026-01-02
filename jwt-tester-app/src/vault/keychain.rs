use anyhow::Context;

pub trait KeychainStore: Send + Sync {
    fn set_password(&self, service: &str, account: &str, secret: &str) -> anyhow::Result<()>;
    fn get_password(&self, service: &str, account: &str) -> anyhow::Result<String>;
    fn delete_password(&self, service: &str, account: &str) -> anyhow::Result<()>;
}

pub struct OsKeychain;

impl OsKeychain {
    pub fn new() -> Self {
        Self
    }
}

impl KeychainStore for OsKeychain {
    fn set_password(&self, service: &str, account: &str, secret: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(service, account)
            .with_context(|| format!("open keychain entry for {service}:{account}"))?;
        entry
            .set_password(secret)
            .with_context(|| format!("set keychain password for {service}:{account}"))?;
        Ok(())
    }

    fn get_password(&self, service: &str, account: &str) -> anyhow::Result<String> {
        let entry = keyring::Entry::new(service, account)
            .with_context(|| format!("open keychain entry for {service}:{account}"))?;
        entry
            .get_password()
            .with_context(|| format!("get keychain password for {service}:{account}"))
    }

    fn delete_password(&self, service: &str, account: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(service, account)
            .with_context(|| format!("open keychain entry for {service}:{account}"))?;
        let _ = entry.delete_credential();
        Ok(())
    }
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct MemoryKeychain {
    store: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

#[cfg(test)]
impl MemoryKeychain {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn len(&self) -> usize {
        self.store.lock().unwrap().len()
    }

    fn key(service: &str, account: &str) -> String {
        format!("{service}:{account}")
    }
}

#[cfg(test)]
impl KeychainStore for MemoryKeychain {
    fn set_password(&self, service: &str, account: &str, secret: &str) -> anyhow::Result<()> {
        let mut locked = self.store.lock().unwrap();
        locked.insert(Self::key(service, account), secret.to_string());
        Ok(())
    }

    fn get_password(&self, service: &str, account: &str) -> anyhow::Result<String> {
        let locked = self.store.lock().unwrap();
        locked
            .get(&Self::key(service, account))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("keychain entry not found"))
    }

    fn delete_password(&self, service: &str, account: &str) -> anyhow::Result<()> {
        let mut locked = self.store.lock().unwrap();
        locked.remove(&Self::key(service, account));
        Ok(())
    }
}
