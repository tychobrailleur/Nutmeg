/* secret.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use async_trait::async_trait;
use log::debug;

/// Trait to allow for mocking the secret service
#[async_trait]
pub trait SecretStorageService: Send + Sync {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError>;
    async fn get_secret(&self, key: &str) -> Result<Option<String>, SecretError>;
    async fn delete_secret(&self, key: &str) -> Result<(), SecretError>;

    async fn clear_all_oauth_secrets(&self) -> Result<(), SecretError> {
        let _ = self.delete_secret("oauth_consumer_key").await;
        let _ = self.delete_secret("oauth_consumer_secret").await;
        let _ = self.delete_secret("access_token").await;
        let _ = self.delete_secret("access_secret").await;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret service error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct SystemSecretService;

impl SystemSecretService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemSecretService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStorageService for SystemSecretService {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let key = key.to_string();
        let value = value.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), SecretError> {
            let entry = keyring::Entry::new("nutmeg", &key)?;
            entry.set_password(&value)?;
            debug!("Stored secret for key: {}", key);
            Ok(())
        })
        .await
        .map_err(|e| SecretError::Io(std::io::Error::other(e.to_string())))??;
        Ok(())
    }

    async fn get_secret(&self, key: &str) -> Result<Option<String>, SecretError> {
        let key = key.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<String>, SecretError> {
            let entry = keyring::Entry::new("nutmeg", &key)?;
            match entry.get_password() {
                Ok(password) => Ok(Some(password)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(SecretError::Keyring(e)),
            }
        })
        .await
        .map_err(|e| SecretError::Io(std::io::Error::other(e.to_string())))?
    }

    async fn delete_secret(&self, key: &str) -> Result<(), SecretError> {
        let key = key.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), SecretError> {
            let entry = keyring::Entry::new("nutmeg", &key)?;
            // Keyring throws an error if we try to delete a non-existent key, so we ignore NoEntry
            match entry.delete_credential() {
                Ok(()) => {
                    debug!("Deleted secret for key: {}", key);
                    Ok(())
                }
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(SecretError::Keyring(e)),
            }
        })
        .await
        .map_err(|e| SecretError::Io(std::io::Error::other(e.to_string())))??;
        Ok(())
    }
}

#[cfg(test)]
pub struct MockSecretService {
    storage: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
}

#[cfg(test)]
impl MockSecretService {
    pub fn new() -> Self {
        Self {
            storage: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl SecretStorageService for MockSecretService {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let mut storage = self.storage.lock().await;
        storage.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get_secret(&self, key: &str) -> Result<Option<String>, SecretError> {
        let storage = self.storage.lock().await;
        Ok(storage.get(key).cloned())
    }

    async fn delete_secret(&self, key: &str) -> Result<(), SecretError> {
        let mut storage = self.storage.lock().await;
        storage.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_storage() {
        let service = MockSecretService::new();
        service.store_secret("user_token", "abc-123").await.unwrap();

        let secret = service.get_secret("user_token").await.unwrap();
        assert_eq!(secret, Some("abc-123".to_string()));

        service.delete_secret("user_token").await.unwrap();
        let secret = service.get_secret("user_token").await.unwrap();
        assert_eq!(secret, None);
    }
}
