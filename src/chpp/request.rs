use http_types::{Method, Url};
use log::{debug, info};
use oauth_1a::*;
use serde_xml_rs::from_str;
use std::collections::BTreeMap;

use crate::chpp::error::Error;
use crate::chpp::model::HattrickData;
use crate::chpp::{CHPP_URL, HOCTANE_USER_AGENT};

use serde::de::DeserializeOwned;

pub async fn chpp_request<T: DeserializeOwned>(
    file: &str,
    version: &str,
    extra_params: Option<&Vec<(&str, &str)>>,
    mut data: OAuthData,
    key: SigningKey,
) -> Result<T, Error> {
    let chpp_str_url = CHPP_URL.replace(":file", file).replace(":version", version);
    let chpp_url = Url::parse(chpp_str_url.as_str()).unwrap();

    let mut params = BTreeMap::new();
    params.insert(String::from("file"), String::from(file));
    params.insert(String::from("version"), String::from(version));

    // Build URL for request with query parameters
    let mut send_url = chpp_url.clone();
    {
        let mut pairs = send_url.query_pairs_mut();
        pairs.append_pair("file", file);
        pairs.append_pair("version", version);

        if let Some(extras) = extra_params {
            for (k, v) in extras {
                pairs.append_pair(k, v);
                params.insert(k.to_string(), v.to_string());
            }
        }
    }

    data.regen_nonce();
    for (k, v) in data.parameters() {
        if k != "oauth_verifier" {
            params.insert(k, v);
        }
    }

    let req = SignableRequest::new(Method::Get, chpp_url.clone(), params);
    debug!(
        "Signable request: {}",
        std::str::from_utf8(&req.to_bytes()).unwrap()
    );
    let authorization = data.authorization(req, AuthorizationType::Request, &key);
    debug!("---\nAuthorization: {}", authorization);

    let client = reqwest::Client::new();
    let response = client
        .get(send_url)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .header("User-Agent", HOCTANE_USER_AGENT)
        .header("Accept-Language", "en")
        .header(
            "Accept",
            "image/gif, image/x-xbitmap, image/jpeg, image/pjpeg, */*",
        )
        .send()
        .await;

    match response {
        Ok(resp) => {
            let data_str = resp.text().await.unwrap();
            info!("Output: {}", data_str);
            let hattrick_data: T = from_str(data_str.as_str()).unwrap();
            Ok(hattrick_data)
        }
        Err(e) => Err(Error::Network(e.to_string())),
    }
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

use crate::chpp::model::PlayersData;

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

    chpp_request::<PlayersData>("players", "2.4", Some(&params), data, key).await
}
