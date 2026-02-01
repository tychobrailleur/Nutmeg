/* client.rs
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

use crate::chpp::error::Error;
use crate::chpp::model::{HattrickData, PlayersData, WorldDetails};
use crate::chpp::oauth::{OAuthData, SigningKey};
use crate::chpp::request::{players_request, team_details_request, world_details_request};
use async_trait::async_trait;

#[async_trait]
pub trait ChppClient: Send + Sync {
    async fn world_details(&self, data: OAuthData, key: SigningKey) -> Result<WorldDetails, Error>;

    async fn team_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<HattrickData, Error>;

    async fn players(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<PlayersData, Error>;
}

pub struct HattrickClient;

impl HattrickClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ChppClient for HattrickClient {
    async fn world_details(&self, data: OAuthData, key: SigningKey) -> Result<WorldDetails, Error> {
        world_details_request(data, key).await
    }

    async fn team_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<HattrickData, Error> {
        team_details_request(data, key, team_id).await
    }

    async fn players(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<PlayersData, Error> {
        players_request(data, key, team_id).await
    }
}
