/* error.rs
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

use gtk::glib;
use thiserror::Error;

#[derive(Clone, Error, Debug, glib::Boxed)]
#[boxed_type(name = "NutmegError")]
pub enum NutmegError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("XML error: {0}")]
    Xml(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("CHPP API error {code}: {message}")]
    ChppApi {
        code: u32,
        message: String,
        error_guid: Option<String>,
        request: Option<String>,
    },

    #[error("IO error: {0}")]
    Io(String),

    #[error("Database error: {0}")]
    Db(String),

    #[error("Application error: {0}")]
    Application(String),
}

impl From<reqwest::Error> for NutmegError {
    fn from(err: reqwest::Error) -> Self {
        NutmegError::Network(err.to_string())
    }
}

impl From<serde_xml_rs::Error> for NutmegError {
    fn from(err: serde_xml_rs::Error) -> Self {
        NutmegError::Parse(err.to_string())
    }
}

impl From<std::io::Error> for NutmegError {
    fn from(err: std::io::Error) -> Self {
        NutmegError::Io(err.to_string())
    }
}

impl From<diesel::result::Error> for NutmegError {
    fn from(err: diesel::result::Error) -> Self {
        NutmegError::Db(err.to_string())
    }
}

impl From<String> for NutmegError {
    fn from(s: String) -> Self {
        NutmegError::Application(s)
    }
}

impl From<&str> for NutmegError {
    fn from(s: &str) -> Self {
        NutmegError::Application(s.to_string())
    }
}
