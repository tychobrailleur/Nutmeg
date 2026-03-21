/* request.rs
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

use http_types::{Method, Url};
use log::{debug, info};
use oauth_1a::*;
use serde_xml_rs::from_str;
use std::collections::BTreeMap;

use crate::error::NutmegError;
use crate::chpp::metadata::ChppEndpoints;
use crate::chpp::model::{
    AvatarsData, ChppErrorResponse, HattrickData, LeagueDetailsData, MatchDetailsData,
    MatchLineupData, MatchesArchiveData, MatchesData, Player, PlayerDetailsData, PlayersData,
    StaffListData, WorldDetails,
};
use crate::chpp::{CHPP_URL, NUTMEG_USER_AGENT};

use serde::de::DeserializeOwned;

pub async fn chpp_request<T: DeserializeOwned>(
    file: &str,
    version: &str,
    extra_params: Option<&Vec<(&str, &str)>>,
    mut data: OAuthData,
    key: SigningKey,
) -> Result<T, NutmegError> {
    use crate::chpp::retry::{should_retry, RetryConfig};

    let config = RetryConfig::default();
    let mut backoff_ms = config.initial_backoff_ms;

    for attempt in 0..=config.max_retries {
        let result =
            perform_single_request::<T>(file, version, extra_params, &mut data, &key).await;

        match result {
            Ok(data) => return Ok(data),
            Err(e) => {
                if attempt == config.max_retries {
                    log::error!(
                        "CHPP request to {} v{} failed after {} retries: {}",
                        file,
                        version,
                        config.max_retries,
                        e
                    );
                    return Err(e);
                }

                if should_retry(&e) {
                    log::warn!(
                        "CHPP request to {} v{} attempt {}/{} failed: {}. Retrying in {}ms...",
                        file,
                        version,
                        attempt + 1,
                        config.max_retries + 1,
                        e,
                        backoff_ms
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
                } else {
                    log::error!(
                        "CHPP request to {} v{} encountered non-retryable error: {}",
                        file,
                        version,
                        e
                    );
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}

async fn perform_single_request<T: DeserializeOwned>(
    file: &str,
    version: &str,
    extra_params: Option<&Vec<(&str, &str)>>,
    data: &mut OAuthData,
    key: &SigningKey,
) -> Result<T, NutmegError> {
    let chpp_str_url = CHPP_URL.replace(":file", file).replace(":version", version);
    let chpp_url = Url::parse(chpp_str_url.as_str())
        .map_err(|e| NutmegError::Network(format!("Invalid URL: {}", e)))?;

    let mut params = BTreeMap::new();
    params.insert(String::from("file"), String::from(file));
    params.insert(String::from("version"), String::from(version));

    // Build URL for request with query parameters
    let mut send_url_builder = chpp_url.clone();
    {
        let mut pairs = send_url_builder.query_pairs_mut();
        pairs.append_pair("file", file);
        pairs.append_pair("version", version);

        if let Some(extras) = extra_params {
            for (k, v) in extras {
                pairs.append_pair(k, v);
                params.insert(k.to_string(), v.to_string());
            }
        }
    }
    let send_url = Url::parse(send_url_builder.as_ref())
        .map_err(|e| NutmegError::Network(format!("Invalid send URL: {}", e)))?;

    data.regen_nonce();
    for (k, v) in data.parameters() {
        if k != "oauth_verifier" {
            params.insert(k, v);
        }
    }

    let req = SignableRequest::new(Method::Get, chpp_url.clone(), params);
    debug!(
        "Signable request: {}",
        std::str::from_utf8(&req.to_bytes()).unwrap_or("Invalid UTF-8")
    );
    let authorization = data.authorization(req, AuthorizationType::Request, key);
    debug!("---\nAuthorization: {}", authorization);

    let client = reqwest::Client::new();
    let response = client
        .get(send_url)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .header("User-Agent", NUTMEG_USER_AGENT)
        .header("Accept-Language", "en")
        .header(
            "Accept",
            "image/gif, image/x-xbitmap, image/jpeg, image/pjpeg, */*",
        )
        .send()
        .await;

    match response {
        Ok(resp) => {
            let data_str = resp
                .text()
                .await
                .map_err(|e| NutmegError::Network(format!("Failed to read response: {}", e)))?;
            info!("Output: {}", data_str);

            // Check if this is an error response before attempting deserialization
            if data_str.contains("<ErrorCode>") {
                let error_response: ChppErrorResponse = from_str(data_str.as_str())
                    .map_err(|e| NutmegError::Xml(format!("Failed to parse error response: {}", e)))?;

                log::error!(
                    "CHPP API error {}: {} (Request: {}, GUID: {})",
                    error_response.ErrorCode,
                    error_response.Error,
                    error_response.Request.as_deref().unwrap_or("unknown"),
                    error_response.ErrorGUID.as_deref().unwrap_or("none")
                );

                return Err(NutmegError::ChppApi {
                    code: error_response.ErrorCode,
                    message: error_response.Error,
                    error_guid: error_response.ErrorGUID,
                    request: error_response.Request,
                });
            }

            // Special debug: save league details XML to file for inspection
            if file.contains("leaguedetails") {
                if let Err(e) = std::fs::write("/tmp/league_details_response.xml", &data_str) {
                    log::error!("Failed to write league details XML to file: {}", e);
                } else {
                    log::info!("League details XML saved to /tmp/league_details_response.xml");
                }
            }

            let hattrick_data: T = match from_str(data_str.as_str()) {
                Ok(data) => data,
                Err(e) => {
                    let preview = if data_str.len() > 100 {
                        format!("{}...", &data_str[..100])
                    } else {
                        data_str.clone()
                    };

                    log::error!(
                        "Failed to deserialize XML from {} v{}. Error: {}. Raw response (first 100 chars): {}",
                        file, version, e, preview
                    );

                    // Handle common non-XML error responses from CHPP/OAuth
                    if data_str.contains("expired_token") {
                        return Err(NutmegError::Auth("expired_token".to_string()));
                    } else if data_str.contains("invalid_token") {
                        return Err(NutmegError::Auth("invalid_token".to_string()));
                    } else if data_str.contains("version_not_supported") {
                        return Err(NutmegError::ChppApi {
                            code: 400,
                            message: "version_not_supported".to_string(),
                            error_guid: None,
                            request: Some(file.to_string()),
                        });
                    }

                    return Err(NutmegError::Xml(format!("Failed to deserialize XML: {}", e)));
                }
            };
            Ok(hattrick_data)
        }
        Err(e) => Err(NutmegError::Network(e.to_string())),
    }
}

pub async fn world_details_request(
    data: OAuthData,
    key: SigningKey,
) -> Result<WorldDetails, NutmegError> {
    chpp_request::<WorldDetails>(
        ChppEndpoints::WORLD_DETAILS.name,
        ChppEndpoints::WORLD_DETAILS.version,
        None,
        data,
        key,
    )
    .await
}

pub async fn team_details_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<HattrickData, NutmegError> {
    if let Some(tid) = team_id {
        let tid_str = tid.to_string();
        let p = vec![("teamID", tid_str.as_str())];
        chpp_request::<HattrickData>(
            ChppEndpoints::TEAM_DETAILS.name,
            ChppEndpoints::TEAM_DETAILS.version,
            Some(&p),
            data,
            key,
        )
        .await
    } else {
        chpp_request::<HattrickData>(
            ChppEndpoints::TEAM_DETAILS.name,
            ChppEndpoints::TEAM_DETAILS.version,
            None,
            data,
            key,
        )
        .await
    }
}

pub async fn players_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<PlayersData, NutmegError> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("teamID", tid_str.as_str()));
    }
    params.push(("actionType", "view"));
    params.push(("includeMatchInfo", "true"));
    chpp_request::<PlayersData>(
        ChppEndpoints::PLAYERS.name,
        ChppEndpoints::PLAYERS.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn player_details_request(
    data: OAuthData,
    key: SigningKey,
    player_id: u32,
) -> Result<Player, NutmegError> {
    let pid_str = player_id.to_string();
    let params = vec![("playerID", pid_str.as_str())];

    let response = chpp_request::<PlayerDetailsData>(
        ChppEndpoints::PLAYER_DETAILS.name,
        ChppEndpoints::PLAYER_DETAILS.version,
        Some(&params),
        data,
        key,
    )
    .await?;
    Ok(response.Player)
}

pub async fn avatars_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<AvatarsData, NutmegError> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("actionType", "players"));
        params.push(("teamId", tid_str.as_str()));
    }

    chpp_request::<AvatarsData>(
        ChppEndpoints::AVATARS.name,
        ChppEndpoints::AVATARS.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn league_details_request(
    data: OAuthData,
    key: SigningKey,
    league_level_unit_id: u32,
) -> Result<LeagueDetailsData, NutmegError> {
    let id_str = league_level_unit_id.to_string();
    let params = vec![("leagueLevelUnitID", id_str.as_str())];

    chpp_request::<LeagueDetailsData>(
        ChppEndpoints::LEAGUE_DETAILS.name,
        ChppEndpoints::LEAGUE_DETAILS.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn matches_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<MatchesData, NutmegError> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("teamID", tid_str.as_str()));
    }

    chpp_request::<MatchesData>(
        ChppEndpoints::MATCHES.name,
        ChppEndpoints::MATCHES.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn matches_archive_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
    first_match_date: Option<String>,
    last_match_date: Option<String>,
) -> Result<MatchesArchiveData, NutmegError> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("teamID", tid_str.as_str()));
    }

    // We need to keep strings alive if we borrow them
    // But since function args are owned Option<String>, we can borrow from them
    if let Some(ref fmd) = first_match_date {
        params.push(("FirstMatchDate", fmd.as_str()));
    }
    if let Some(ref lmd) = last_match_date {
        params.push(("LastMatchDate", lmd.as_str()));
    }

    chpp_request::<MatchesArchiveData>(
        ChppEndpoints::MATCHES_ARCHIVE.name,
        ChppEndpoints::MATCHES_ARCHIVE.version,
        Some(&params),
        data,
        key,
    )
    .await
}
pub async fn staff_list_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<StaffListData, NutmegError> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("teamId", tid_str.as_str()));
    }

    chpp_request::<StaffListData>(
        ChppEndpoints::STAFF_LIST.name,
        ChppEndpoints::STAFF_LIST.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn match_details_request(
    data: OAuthData,
    key: SigningKey,
    match_id: u32,
    source_system: &str,
) -> Result<MatchDetailsData, NutmegError> {
    let mid_str = match_id.to_string();
    let params = vec![
        ("matchID", mid_str.as_str()),
        ("sourceSystem", source_system),
        ("matchEvents", "true"),
    ];

    chpp_request::<MatchDetailsData>(
        ChppEndpoints::MATCH_DETAILS.name,
        ChppEndpoints::MATCH_DETAILS.version,
        Some(&params),
        data,
        key,
    )
    .await
}

pub async fn match_lineup_request(
    data: OAuthData,
    key: SigningKey,
    match_id: u32,
    team_id: u32,
    source_system: &str,
) -> Result<MatchLineupData, NutmegError> {
    let mid_str = match_id.to_string();
    let tid_str = team_id.to_string();
    let params = vec![
        ("matchID", mid_str.as_str()),
        ("teamID", tid_str.as_str()),
        ("sourceSystem", source_system),
    ];

    chpp_request::<MatchLineupData>(
        ChppEndpoints::MATCH_LINEUP.name,
        ChppEndpoints::MATCH_LINEUP.version,
        Some(&params),
        data,
        key,
    )
    .await
}
