/* secret.rs
 *
 * Copyright 2026 sebastien
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
use std::collections::HashMap;

/// Trait to allow for mocking the secret service
#[async_trait]
pub trait SecretStorageService: Send + Sync {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError>;
    async fn get_secret(&self, key: &str) -> Result<Option<String>, SecretError>;
    async fn delete_secret(&self, key: &str) -> Result<(), SecretError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret service error: {0}")]
    Oo7(#[from] oo7::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}

pub struct GnomeSecretService;

impl GnomeSecretService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GnomeSecretService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStorageService for GnomeSecretService {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let keyring = oo7::Keyring::new().await?;

        // Define attributes to identify the secret
        let mut attributes = HashMap::new();
        attributes.insert("application", "nutmeg");
        attributes.insert("key", key);

        keyring
            .create_item(
                "Nutmeg Secret",
                &attributes,
                value,
                true, // replace
            )
            .await?;

        debug!("Stored secret for key: {}", key);
        Ok(())
    }

    async fn get_secret(&self, key: &str) -> Result<Option<String>, SecretError> {
        let keyring = oo7::Keyring::new().await?;

        let mut attributes = HashMap::new();
        attributes.insert("application", "nutmeg");
        attributes.insert("key", key);

        let items = keyring.search_items(&attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            let secret_str =
                String::from_utf8(secret.to_vec()).map_err(|_| SecretError::Unknown)?;
            return Ok(Some(secret_str));
        }

        Ok(None)
    }

    async fn delete_secret(&self, key: &str) -> Result<(), SecretError> {
        let keyring = oo7::Keyring::new().await?;

        let mut attributes = HashMap::new();
        attributes.insert("application", "nutmeg");
        attributes.insert("key", key);

        keyring.delete(&attributes).await?;

        debug!("Deleted secret for key: {}", key);
        Ok(())
    }
}

#[cfg(test)]
pub struct MockSecretService {
    storage: std::sync::Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

#[cfg(test)]
impl MockSecretService {
    pub fn new() -> Self {
        Self {
            storage: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
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
