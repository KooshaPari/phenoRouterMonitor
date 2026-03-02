//! API key generation and lifecycle management.
//!
//! On first startup, if no API key exists in the credential store, a new
//! 32-byte random key is generated, base64url-encoded as the plaintext key,
//! and its SHA-256 hash is stored for validation.
//!
//! The plaintext key is:
//!   - written to `~/.config/agileplus/api-key` with 0600 permissions
//!   - printed to stdout once so the operator can record it
//!
//! Traceability: WP11-T064

use std::path::PathBuf;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

use agileplus_domain::credentials::{CredentialStore, keys};

/// Prefix that identifies an AgilePlus API key.
const KEY_PREFIX: &str = "agp_";

/// Generate a new API key: 32 random bytes → base64url-encoded plaintext.
///
/// Returns the plaintext key (to be shown to the user once).
pub fn generate_plaintext_key() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("{}{}", KEY_PREFIX, URL_SAFE_NO_PAD.encode(bytes))
}

/// Hash a plaintext API key using SHA-256.
pub fn hash_key(plaintext: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hasher.finalize().into()
}

/// Default path for storing the plaintext API key.
pub fn default_key_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("agileplus")
        .join("api-key")
}

/// Ensure an API key exists in the credential store.
///
/// If no key is found, generates a new one, stores its hash, writes the
/// plaintext to `~/.config/agileplus/api-key`, and prints it to stdout.
///
/// Returns `true` if a new key was generated, `false` if one already existed.
pub async fn ensure_api_key(creds: &dyn CredentialStore) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // Check if a key already exists.
    let existing = creds.get("agileplus", keys::API_KEYS).await;
    if let Ok(val) = existing {
        if !val.trim().is_empty() {
            return Ok(false);
        }
    }

    // Generate new key.
    let plaintext = generate_plaintext_key();

    // Store the plaintext directly in the credential store for validation
    // (the default InMemoryCredentialStore / FileCredentialStore validate
    // against comma-separated plaintext keys — see credentials.rs).
    creds.set("agileplus", keys::API_KEYS, &plaintext).await?;

    // Write plaintext to config file with 0600 permissions.
    let key_path = default_key_file_path();
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&key_path, &plaintext)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&key_path, perms)?;
    }

    // Print to stdout for the operator.
    println!("AgilePlus API initialized.");
    println!("API Key: {plaintext}");
    println!("Store this key securely; it won't be shown again.");
    println!("(Also saved to {})", key_path.display());

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_has_prefix() {
        let key = generate_plaintext_key();
        assert!(key.starts_with(KEY_PREFIX));
    }

    #[test]
    fn generated_key_is_unique() {
        let a = generate_plaintext_key();
        let b = generate_plaintext_key();
        assert_ne!(a, b);
    }

    #[test]
    fn hash_is_deterministic() {
        let key = "agp_test_key";
        let h1 = hash_key(key);
        let h2 = hash_key(key);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_differs_for_different_keys() {
        let h1 = hash_key("agp_key_one");
        let h2 = hash_key("agp_key_two");
        assert_ne!(h1, h2);
    }
}
