/* auth.rs
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

use crate::chpp::{exchange_verification_code, get_request_token_url, Error, OauthSettings};
use crate::config::{consumer_key, consumer_secret};

// Trait for dependency injection and mocking
pub trait AuthenticationService {
    /// Returns the authorization URL and the temporary request tokens (token, secret)
    fn get_authorization_url(&self) -> Result<(String, String, String), Error>;

    /// Exchanges verification code for access tokens (token, secret)
    fn verify_user(
        &self,
        verification_code: &str,
        request_token: &str,
        request_token_secret: &str,
    ) -> Result<(String, String), Error>;
}

pub struct HattrickAuthService;

impl HattrickAuthService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HattrickAuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthenticationService for HattrickAuthService {
    fn get_authorization_url(&self) -> Result<(String, String, String), Error> {
        let settings = OauthSettings::default();
        let key = consumer_key();
        let secret = consumer_secret();
        let url = get_request_token_url(&settings, &key, &secret)?;

        let token = settings.request_token.take();
        let secret = settings.oauth_secret_token.take();

        Ok((url, token, secret))
    }

    fn verify_user(
        &self,
        verification_code: &str,
        request_token: &str,
        request_token_secret: &str,
    ) -> Result<(String, String), Error> {
        // Reconstruct settings for the exchange
        let settings = OauthSettings::default();
        settings.client_id.replace(consumer_key());
        settings.client_secret.replace(consumer_secret());
        settings.request_token.replace(request_token.to_string());
        settings
            .oauth_secret_token
            .replace(request_token_secret.to_string());

        exchange_verification_code(verification_code, &settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAuthService;
    impl AuthenticationService for MockAuthService {
        fn get_authorization_url(&self) -> Result<(String, String, String), Error> {
            Ok((
                "http://mock.url".to_string(),
                "mock_tok".to_string(),
                "mock_sec".to_string(),
            ))
        }
        fn verify_user(
            &self,
            _code: &str,
            _rt: &str,
            _rs: &str,
        ) -> Result<(String, String), Error> {
            Ok(("access_tok".to_string(), "access_sec".to_string()))
        }
    }

    #[test]
    fn test_auth_flow_interface() {
        let auth = MockAuthService;
        let (url, _, _) = auth.get_authorization_url().unwrap();
        assert_eq!(url, "http://mock.url");

        let (acc, _) = auth.verify_user("123", "tok", "sec").unwrap();
        assert_eq!(acc, "access_tok");
    }
}
