/* client.rs
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

use crate::chpp::error::Error;
use crate::chpp::model::{
    AvatarsData, HattrickData, LeagueDetailsData, MatchDetailsData, MatchLineupData,
    MatchesArchiveData, MatchesData, Player, PlayersData, StaffListData, WorldDetails,
};
use crate::chpp::oauth::{OAuthData, SigningKey};
use crate::chpp::request::{
    league_details_request, match_details_request, match_lineup_request, matches_archive_request,
    matches_request, player_details_request, players_request, team_details_request,
    world_details_request,
};
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

    async fn player_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        player_id: u32,
    ) -> Result<Player, Error>;

    async fn avatars(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<AvatarsData, Error>;

    async fn league_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        league_level_unit_id: u32,
    ) -> Result<LeagueDetailsData, Error>;

    async fn matches(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<MatchesData, Error>;

    async fn staff_list(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<StaffListData, Error>;

    async fn matches_archive(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
        first_match_date: Option<String>,
        last_match_date: Option<String>,
    ) -> Result<MatchesArchiveData, Error>;

    async fn match_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        match_id: u32,
        source_system: &str,
    ) -> Result<MatchDetailsData, Error>;

    async fn match_lineup(
        &self,
        data: OAuthData,
        key: SigningKey,
        match_id: u32,
        team_id: u32,
        source_system: &str,
    ) -> Result<MatchLineupData, Error>;
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

    async fn player_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        player_id: u32,
    ) -> Result<Player, Error> {
        player_details_request(data, key, player_id).await
    }

    async fn avatars(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<AvatarsData, Error> {
        crate::chpp::request::avatars_request(data, key, team_id).await
    }

    async fn league_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        league_level_unit_id: u32,
    ) -> Result<LeagueDetailsData, Error> {
        league_details_request(data, key, league_level_unit_id).await
    }

    async fn matches(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<MatchesData, Error> {
        matches_request(data, key, team_id).await
    }

    async fn staff_list(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
    ) -> Result<StaffListData, Error> {
        crate::chpp::request::staff_list_request(data, key, team_id).await
    }

    async fn matches_archive(
        &self,
        data: OAuthData,
        key: SigningKey,
        team_id: Option<u32>,
        first_match_date: Option<String>,
        last_match_date: Option<String>,
    ) -> Result<MatchesArchiveData, Error> {
        matches_archive_request(data, key, team_id, first_match_date, last_match_date).await
    }

    async fn match_details(
        &self,
        data: OAuthData,
        key: SigningKey,
        match_id: u32,
        source_system: &str,
    ) -> Result<MatchDetailsData, Error> {
        match_details_request(data, key, match_id, source_system).await
    }

    async fn match_lineup(
        &self,
        data: OAuthData,
        key: SigningKey,
        match_id: u32,
        team_id: u32,
        source_system: &str,
    ) -> Result<MatchLineupData, Error> {
        match_lineup_request(data, key, match_id, team_id, source_system).await
    }
}
