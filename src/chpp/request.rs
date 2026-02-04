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

use crate::chpp::error::Error;
use crate::chpp::model::{
    ChppErrorResponse, HattrickData, Player, PlayerDetailsData, PlayersData, WorldDetails,
};
use crate::chpp::{CHPP_URL, NUTMEG_USER_AGENT};

use serde::de::DeserializeOwned;

pub async fn chpp_request<T: DeserializeOwned>(
    file: &str,
    version: &str,
    extra_params: Option<&Vec<(&str, &str)>>,
    mut data: OAuthData,
    key: SigningKey,
) -> Result<T, Error> {
    let chpp_str_url = CHPP_URL.replace(":file", file).replace(":version", version);
    let chpp_url = Url::parse(chpp_str_url.as_str())
        .map_err(|e| Error::Network(format!("Invalid URL: {}", e)))?;

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
    let send_url = Url::parse(&send_url_builder.to_string())
        .map_err(|e| Error::Network(format!("Invalid send URL: {}", e)))?;

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
    let authorization = data.authorization(req, AuthorizationType::Request, &key);
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
                .map_err(|e| Error::Network(format!("Failed to read response: {}", e)))?;
            info!("Output: {}", data_str);

            // Check if this is an error response before attempting deserialization
            if data_str.contains("<ErrorCode>") {
                let error_response: ChppErrorResponse = from_str(data_str.as_str())
                    .map_err(|e| Error::Xml(format!("Failed to parse error response: {}", e)))?;

                log::error!(
                    "CHPP API error {}: {} (Request: {}, GUID: {})",
                    error_response.ErrorCode,
                    error_response.Error,
                    error_response.Request.as_deref().unwrap_or("unknown"),
                    error_response.ErrorGUID.as_deref().unwrap_or("none")
                );

                return Err(Error::ChppApi {
                    code: error_response.ErrorCode,
                    message: error_response.Error,
                    error_guid: error_response.ErrorGUID,
                    request: error_response.Request,
                });
            }

            let hattrick_data: T = from_str(data_str.as_str())
                .map_err(|e| Error::Xml(format!("Failed to deserialize XML: {}", e)))?;
            Ok(hattrick_data)
        }
        Err(e) => Err(Error::Network(e.to_string())),
    }
}

pub async fn world_details_request(
    data: OAuthData,
    key: SigningKey,
) -> Result<WorldDetails, Error> {
    chpp_request::<WorldDetails>("worlddetails", "1.9", None, data, key).await
}

pub async fn team_details_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<HattrickData, Error> {
    if let Some(tid) = team_id {
        let tid_str = tid.to_string();
        let p = vec![("teamID", tid_str.as_str())];
        chpp_request::<HattrickData>("teamdetails", "3.7", Some(&p), data, key).await
    } else {
        chpp_request::<HattrickData>("teamdetails", "3.7", None, data, key).await
    }
}

pub async fn players_request(
    data: OAuthData,
    key: SigningKey,
    team_id: Option<u32>,
) -> Result<PlayersData, Error> {
    let mut params = Vec::new();
    let tid_str;
    if let Some(tid) = team_id {
        tid_str = tid.to_string();
        params.push(("teamID", tid_str.as_str()));
    }
    params.push(("actionType", "view"));
    params.push(("includeMatchInfo", "true"));
    chpp_request::<PlayersData>("players", "2.4", Some(&params), data, key).await
}

pub async fn player_details_request(
    data: OAuthData,
    key: SigningKey,
    player_id: u32,
) -> Result<Player, Error> {
    let pid_str = player_id.to_string();
    let params = vec![("playerID", pid_str.as_str())];

    let response =
        chpp_request::<PlayerDetailsData>("playerdetails", "3.1", Some(&params), data, key).await?;
    Ok(response.Player)
}
