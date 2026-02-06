/* oauth.rs
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

use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
};

use http_types::{Method, Url};
use log::info;
use oauth_1a::*;
pub use oauth_1a::{OAuthData, SigningKey};

use crate::chpp::error::Error;
use crate::chpp::CHPP_OAUTH_AUTH_URL;
use crate::chpp::{CHPP_OAUTH_ACCESS_TOKEN_URL, CHPP_OAUTH_REQUEST_TOKEN_URL};

#[derive(Clone, Default)]
pub struct OauthSettings {
    pub request_token: RefCell<String>,
    pub oauth_secret_token: RefCell<String>,
    pub nonce: RefCell<String>,
    pub client_id: RefCell<String>,
    pub client_secret: RefCell<String>,
}

impl Debug for OauthSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let binding = self.request_token.borrow_mut();
        let tok: &str = binding.as_str();

        f.debug_tuple(tok).finish()
    }
}

pub fn get_request_token_url(
    settings: &OauthSettings,
    consumer_key: &str,
    consumer_secret: &str,
) -> Result<String, Error> {
    let client_id = ClientId(consumer_key.to_string());
    let client_secret = ClientSecret(consumer_secret.to_string());

    let mut key = SigningKey::without_token(client_secret);
    let mut data = OAuthData {
        client_id,
        token: None,
        signature_method: SignatureMethod::HmacSha1,
        nonce: Nonce::generate(),
    };

    // Request temporary credentials (Request Token)
    let initiate = Url::parse(CHPP_OAUTH_REQUEST_TOKEN_URL)
        .map_err(|e| Error::Parse(format!("Invalid request token URL: {}", e)))?;

    settings.client_secret.replace(consumer_secret.to_string());
    settings.client_id.replace(consumer_key.to_string());

    let callback = "oob".to_owned();
    let req = SignableRequest::new(Method::Post, initiate.clone(), Default::default());
    let authorization = data.authorization(req, AuthorizationType::RequestToken { callback }, &key);
    info!("authorization: {}", authorization);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(initiate)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .send()
        .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?
        .text()
        .map_err(|e| Error::Network(format!("Failed to read text: {}", e)))?;

    info!("---\n{}", resp);
    data.regen_nonce();

    let token = receive_token(&mut data, &mut key, &resp).map_err(|e| {
        Error::Auth(format!(
            "Failed to receive token: {}. Response: {}",
            e, resp
        ))
    })?;

    settings.request_token.replace(token.0.clone());
    let token_secret = key
        .token_secret
        .ok_or_else(|| Error::Auth("No token secret in key".to_string()))?;
    match token_secret {
        TokenSecret(s) => {
            settings.oauth_secret_token.replace(s.clone());
        }
    }

    Ok(format!(
        "{}?oauth_token={}&scope=set_matchorder",
        CHPP_OAUTH_AUTH_URL, &token.0
    ))
}

#[allow(dead_code)]
pub fn request_token(
    settings: OauthSettings,
    consumer_key: &str,
    consumer_secret: &str,
    verif_callback: fn(&str) -> i32,
) -> Result<OauthSettings, Error> {
    let client_id = ClientId(consumer_key.to_string());
    let client_secret = ClientSecret(consumer_secret.to_string());

    let mut key = SigningKey::without_token(client_secret);
    let mut data = OAuthData {
        client_id,
        token: None,
        signature_method: SignatureMethod::HmacSha1,
        nonce: Nonce::generate(),
    };

    // Request temporary credentials (Request Token)
    let initiate = Url::parse(CHPP_OAUTH_REQUEST_TOKEN_URL)
        .map_err(|e| Error::Parse(format!("Invalid request token URL: {}", e)))?;

    settings.client_secret.replace(consumer_secret.to_string());
    settings.client_id.replace(consumer_key.to_string());

    let callback = "oob".to_owned();
    let req = SignableRequest::new(Method::Post, initiate.clone(), Default::default());
    let authorization = data.authorization(req, AuthorizationType::RequestToken { callback }, &key);
    info!("authorization: {}", authorization);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(initiate)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .send()
        .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?
        .text()
        .map_err(|e| Error::Network(format!("Failed to read text: {}", e)))?;

    info!("---\n{}", resp);
    data.regen_nonce();

    // authorize: https://chpp.hattrick.org/oauth/authorize.aspx
    let token = receive_token(&mut data, &mut key, &resp)
        .map_err(|e| Error::Auth(format!("Failed to receive token: {}", e)))?;
    info!("---\n{}", token.0);

    settings.request_token.replace(token.0.clone());
    let token_secret = key
        .token_secret
        .ok_or_else(|| Error::Auth("No token secret in key".to_string()))?;
    match token_secret {
        TokenSecret(s) => {
            settings.oauth_secret_token.replace(s.clone());
        }
    }

    // The callback needs to open the URL passed as an argument,
    // authenticate in Hattrick, and obtain the verification code.
    verif_callback(&format!(
        "{}?oauth_token={}&scope=set_matchorder",
        CHPP_OAUTH_AUTH_URL, &token.0
    ));

    Ok(settings)
}

/// Exchange verification code for access token
pub fn exchange_verification_code(
    verification_code: &str,
    settings: &OauthSettings,
) -> Result<(String, String), Error> {
    let consumer_key = settings.client_id.borrow().clone();
    let consumer_secret = settings.client_secret.borrow().clone();
    let request_token = settings.request_token.borrow().clone();
    let oauth_secret_token = settings.oauth_secret_token.borrow().clone();

    if consumer_key.is_empty() || consumer_secret.is_empty() {
        return Err(Error::Auth(
            "Consumer key or secret missing in settings".to_string(),
        ));
    }

    let mut data = OAuthData {
        client_id: ClientId(consumer_key.clone()),
        token: Some(Token(request_token)),
        signature_method: SignatureMethod::HmacSha1,
        nonce: Nonce::generate(),
    };

    let mut key = SigningKey::with_token(
        ClientSecret(consumer_secret),
        TokenSecret(oauth_secret_token),
    );

    let access_url = Url::parse(CHPP_OAUTH_ACCESS_TOKEN_URL)
        .map_err(|e| Error::Parse(format!("Invalid URL: {}", e)))?;

    let req = SignableRequest::new(Method::Post, access_url.clone(), Default::default());
    let access_type = AuthorizationType::AccessToken {
        verifier: verification_code.to_string(),
    };
    let authorization = data.authorization(req, access_type, &key);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(access_url)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .send()
        .map_err(|e| Error::Network(format!("Failed to request access token: {}", e)))?
        .text()
        .map_err(|e| Error::Network(format!("Failed to read response: {}", e)))?;

    data.regen_nonce();

    // Parse response to extract access token and secret
    let token = receive_token(&mut data, &mut key, &resp)
        .map_err(|e| Error::Auth(format!("Failed to receive token: {}", e)))?;

    // Extract the token secret from the signing key
    let access_token = token.0.clone();
    let access_secret = match &key.token_secret {
        Some(secret) => secret.0.clone(),
        None => return Err(Error::Auth("No token secret received".to_string())),
    };

    Ok((access_token, access_secret))
}

pub fn create_oauth_context(
    consumer_key: &str,
    consumer_secret: &str,
    access_token: &str,
    access_secret: &str,
) -> (OAuthData, SigningKey) {
    let client_id = ClientId(consumer_key.to_string());
    let client_secret = ClientSecret(consumer_secret.to_string());
    let token = Token(access_token.to_string());
    let token_secret = TokenSecret(access_secret.to_string());

    let data = OAuthData {
        client_id,
        token: Some(token),
        signature_method: SignatureMethod::HmacSha1,
        nonce: Nonce::generate(),
    };

    let key = SigningKey::with_token(client_secret, token_secret);
    (data, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_oauth_context() {
        let consumer_key = "ckey";
        let consumer_secret = "csecret";
        let access_token = "atoken";
        let access_secret = "asecret";

        let (data, key) =
            create_oauth_context(consumer_key, consumer_secret, access_token, access_secret);

        assert_eq!(data.client_id.0, "ckey");
        assert_eq!(data.token.unwrap().0, "atoken");

        // SigningKey fields are private or hard to check directly depending on visibility,
        // but we can check if it was created successfully (it didn't panic).
        // If we really need to check internals we might need debug impls or accessors,
        // but for now ensuring it runs is good validation of the factory function.
        // HmacSha1 is the hardcoded signature method.
        matches!(data.signature_method, SignatureMethod::HmacSha1);
    }
    #[test]
    fn test_oauth_settings_default() {
        let settings = OauthSettings::default();
        assert!(settings.request_token.borrow().is_empty());
        assert!(settings.oauth_secret_token.borrow().is_empty());
        assert!(settings.client_id.borrow().is_empty());
        assert!(settings.client_secret.borrow().is_empty());
    }
}
