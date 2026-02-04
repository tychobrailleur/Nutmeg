/* mod.rs
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

static CHPP_OAUTH_REQUEST_TOKEN_URL: &str = "https://chpp.hattrick.org/oauth/request_token.ashx";
static CHPP_OAUTH_AUTH_URL: &str = "https://chpp.hattrick.org/oauth/authorize.aspx";
static CHPP_OAUTH_ACCESS_TOKEN_URL: &str = "https://chpp.hattrick.org/oauth/access_token.ashx";
static CHPP_URL: &str = "https://chpp.hattrick.org/chppxml.ashx";
static NUTMEG_USER_AGENT: &str = "Nutmeg/v1.0";

//pub mod authenticator;
pub mod client;
pub mod error;
pub mod model;
mod oauth;
mod request;
pub mod retry;

pub use client::{ChppClient, HattrickClient};
pub use error::Error;
pub use oauth::create_oauth_context;
pub use oauth::exchange_verification_code;
pub use oauth::get_request_token_url;
pub use oauth::request_token;
pub use oauth::OauthSettings;
pub use retry::{retry_with_backoff, retry_with_default_config, should_retry, RetryConfig};
