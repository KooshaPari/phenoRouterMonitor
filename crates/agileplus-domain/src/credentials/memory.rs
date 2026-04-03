use std::collections::HashMap;
use std::sync::RwLock;

use zeroize::Zeroizing;

use super::error::CredentialError;
use super::store::CredentialStore;

/// In-memory credential store for unit tests and CI environments without a keychain.
#[derive(Default)]
pub struct InMemoryCredentialStore {
    store: RwLock<HashMap<String, HashMap<String, Zeroizing<String>>>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn namespace(service: &str, key: &str) -> String {
        format!("{service}::{key}")
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        let store = self.store.read().unwrap();
        store
            .get(service)
            .and_then(|m| m.get(key))
            .map(|v| v.as_str().to_string())
            .ok_or_else(|| CredentialError::NotFound(Self::namespace(service, key)))
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        let mut store = self.store.write().unwrap();
        store
            .entry(service.to_string())
            .or_default()
            .insert(key.to_string(), Zeroizing::new(value.to_string()));
        Ok(())
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        let mut store = self.store.write().unwrap();
        if let Some(svc) = store.get_mut(service) {
            if svc.remove(key).is_none() {
                return Err(CredentialError::NotFound(Self::namespace(service, key)));
            }
        } else {
            return Err(CredentialError::NotFound(Self::namespace(service, key)));
        }
        Ok(())
    }

    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        let store = self.store.read().unwrap();
        Ok(store
            .get(service)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::file::FileCredentialStore;
    use crate::credentials::keys::API_KEYS;
    use crate::credentials::store;

    #[test]
    fn in_memory_set_get_delete() {
        let store = InMemoryCredentialStore::new();
        store.set("svc", "key1", "val1").unwrap();
        assert_eq!(store.get("svc", "key1").unwrap(), "val1");
        store.delete("svc", "key1").unwrap();
        assert!(matches!(
            store.get("svc", "key1"),
            Err(CredentialError::NotFound(_))
        ));
    }

    #[test]
    fn in_memory_list_keys() {
        let store = InMemoryCredentialStore::new();
        store.set("svc", "a", "1").unwrap();
        store.set("svc", "b", "2").unwrap();
        let mut keys = store.list_keys("svc").unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn validate_api_key_single() {
        let store = InMemoryCredentialStore::new();
        store.set("agileplus", API_KEYS, "secret-key-abc").unwrap();
        assert!(store.validate_api_key("secret-key-abc").unwrap());
        assert!(!store.validate_api_key("wrong-key").unwrap());
    }

    #[test]
    fn validate_api_key_multiple() {
        let store = InMemoryCredentialStore::new();
        store
            .set("agileplus", API_KEYS, "key-one, key-two, key-three")
            .unwrap();
        assert!(store.validate_api_key("key-two").unwrap());
        assert!(!store.validate_api_key("key-four").unwrap());
    }

    #[test]
    fn validate_api_key_no_keys_stored() {
        let store = InMemoryCredentialStore::new();
        assert!(!store.validate_api_key("anything").unwrap());
    }

    #[test]
    fn file_store_set_get() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("creds.json");
        let store = FileCredentialStore::new(&path);
        store.set("svc", "tok", "abc123").unwrap();
        assert!(path.exists());
        assert_eq!(store.get("svc", "tok").unwrap(), "abc123");
    }

    #[test]
    fn constant_time_eq_same_length() {
        assert!(store::constant_time_eq(b"hello", b"hello"));
        assert!(!store::constant_time_eq(b"hello", b"hellx"));
    }

    #[test]
    fn constant_time_eq_different_length() {
        assert!(!store::constant_time_eq(b"hello", b"helloo"));
    }
}
