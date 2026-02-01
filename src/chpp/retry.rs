/* retry.rs
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

//! Retry utilities for CHPP API requests
//!
//! This module provides helper functions for retrying operations with
//! exponential backoff, handling transient failures transparently.

use crate::chpp::error::Error;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000, // 1 second
            max_backoff_ms: 32000,    // 32 seconds
        }
    }
}

/// Determine if an error should trigger a retry
pub fn should_retry(error: &Error) -> bool {
    match error {
        Error::Network(_) => true,
        Error::ChppApi { code, .. } => {
            // Retry on common transient error codes
            // 503 = Service unavailable, 429 = Rate limit
            matches!(code, 503 | 429)
        }
        _ => false,
    }
}

/// Execute an async operation with retry logic and exponential backoff
///
/// The operation function receives fresh credentials from `get_credentials`
/// on each attempt (including retries). This allows the OAuth nonce to be
/// regenerated for each request.
///
/// # Arguments
/// * `operation_name` - Name of the operation for logging
/// * `get_credentials` - Function that provides fresh OAuth credentials
/// * `operation` - The async operation to retry, receives OAuthData and SigningKey
/// * `config` - Retry configuration
pub async fn retry_with_backoff<T, F, G, Fut>(
    operation_name: &str,
    get_credentials: G,
    operation: F,
    config: &RetryConfig,
) -> Result<T, Error>
where
    F: Fn(oauth_1a::OAuthData, oauth_1a::SigningKey) -> Fut,
    G: Fn() -> (oauth_1a::OAuthData, oauth_1a::SigningKey),
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let mut backoff_ms = config.initial_backoff_ms;

    for attempt in 0..=config.max_retries {
        let (data, key) = get_credentials();

        match operation(data, key).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == config.max_retries {
                    log::error!(
                        "{} failed after {} retries: {}",
                        operation_name,
                        config.max_retries,
                        e
                    );
                    return Err(e);
                }

                if should_retry(&e) {
                    log::warn!(
                        "{} attempt {}/{} failed: {}. Retrying in {}ms...",
                        operation_name,
                        attempt + 1,
                        config.max_retries + 1,
                        e,
                        backoff_ms
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
                } else {
                    log::error!("{} encountered non-retryable error: {}", operation_name, e);
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}

/// Convenience wrapper for retry with default configuration
pub async fn retry_with_default_config<T, F, G, Fut>(
    operation_name: &str,
    get_credentials: G,
    operation: F,
) -> Result<T, Error>
where
    F: Fn(oauth_1a::OAuthData, oauth_1a::SigningKey) -> Fut,
    G: Fn() -> (oauth_1a::OAuthData, oauth_1a::SigningKey),
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    retry_with_backoff(
        operation_name,
        get_credentials,
        operation,
        &RetryConfig::default(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_on_network_error() {
        let attempts = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let attempts_clone = attempts.clone();

        let get_creds = || {
            (
                oauth_1a::OAuthData {
                    client_id: oauth_1a::ClientId("test".to_string()),
                    token: None,
                    signature_method: oauth_1a::SignatureMethod::HmacSha1,
                    nonce: oauth_1a::Nonce::generate(),
                },
                oauth_1a::SigningKey::without_token(oauth_1a::ClientSecret("test".to_string())),
            )
        };

        let operation = move |_data: oauth_1a::OAuthData, _key: oauth_1a::SigningKey| {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                if *count < 3 {
                    Err(Error::Network("Connection failed".to_string()))
                } else {
                    Ok("success")
                }
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };

        let result = retry_with_backoff("test_op", get_creds, operation, &config).await;
        assert!(result.is_ok());
        assert_eq!(*attempts.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_no_retry_on_permanent_error() {
        let attempts = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let attempts_clone = attempts.clone();

        let get_creds = || {
            (
                oauth_1a::OAuthData {
                    client_id: oauth_1a::ClientId("test".to_string()),
                    token: None,
                    signature_method: oauth_1a::SignatureMethod::HmacSha1,
                    nonce: oauth_1a::Nonce::generate(),
                },
                oauth_1a::SigningKey::without_token(oauth_1a::ClientSecret("test".to_string())),
            )
        };

        let operation = move |_data: oauth_1a::OAuthData, _key: oauth_1a::SigningKey| {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                Err::<&str, _>(Error::Auth("Unauthorized".to_string()))
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };

        let result = retry_with_backoff("test_op", get_creds, operation, &config).await;
        assert!(result.is_err());
        // Should only attempt once since error is not retryable
        assert_eq!(*attempts.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_max_retries_exhausted() {
        let attempts = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let attempts_clone = attempts.clone();

        let get_creds = || {
            (
                oauth_1a::OAuthData {
                    client_id: oauth_1a::ClientId("test".to_string()),
                    token: None,
                    signature_method: oauth_1a::SignatureMethod::HmacSha1,
                    nonce: oauth_1a::Nonce::generate(),
                },
                oauth_1a::SigningKey::without_token(oauth_1a::ClientSecret("test".to_string())),
            )
        };

        let operation = move |_data: oauth_1a::OAuthData, _key: oauth_1a::SigningKey| {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                Err::<&str, _>(Error::Network("Persistent failure".to_string()))
            }
        };

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };

        let result = retry_with_backoff("test_op", get_creds, operation, &config).await;
        assert!(result.is_err());
        // Should attempt 3 times total (initial + 2 retries)
        assert_eq!(*attempts.lock().unwrap(), 3);
    }
}
