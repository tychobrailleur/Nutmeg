use serde_xml_rs::from_str;
use std::collections::BTreeMap;
use http_types::{Url, Method};
use oauth_1a::*;
use log::{info, debug};

use crate::chpp::{CHPP_URL, HOCTANE_USER_AGENT};
use crate::chpp::model::HattrickData;
use crate::chpp::error::Error;

pub async fn chpp_request(
    file: &str,
    version: &str,
    mut data: OAuthData,
    key: SigningKey
) -> Result<HattrickData, Error> {

    let chpp_str_url = CHPP_URL.replace(":file", file)
        .replace(":version", version);
    let chpp_url = Url::parse(chpp_str_url.as_str()).unwrap();

    let mut params = BTreeMap::new();
    params.insert(String::from("file"), String::from(file));
    params.insert(String::from("version"), String::from(version));

    // TODO Insert the other parameters, for example specific TeamID

    data.regen_nonce();
    for (k, v) in data.parameters() {
        if k != "oauth_verifier" {
            params.insert(k, v);
        }
    }

    let req = SignableRequest::new(Method::Get, chpp_url.clone(), params);
    debug!("Signable request: {}", std::str::from_utf8(&req.to_bytes()).unwrap());
    let authorization = data.authorization(req, AuthorizationType::Request, &key);
    debug!("---\nAuthorization: {}", authorization);

    let client = reqwest::Client::new();
    let response = client.get(chpp_url)
        .header("Authorization", authorization)
        .header("Content-Length", "0")
        .header("User-Agent", HOCTANE_USER_AGENT)
        .header("Accept-Language", "en")
        .header("Accept", "image/gif, image/x-xbitmap, image/jpeg, image/pjpeg, */*")
        .send().await;

    match response {
        Ok(resp) => {
            let data_str = resp.text().await.unwrap();
            info!("Output: {}", data_str);
            let hattrick_data: HattrickData = from_str(data_str.as_str()).unwrap();
            Ok(hattrick_data)
        },
        Err(e) => Err(Error::Network(e.to_string()))
    }
}

pub async fn team_details_request(
    data: OAuthData,
    key: SigningKey
) -> Result<HattrickData, Error> {
    chpp_request("teamdetails", "3.7", data, key).await
}

pub async fn players_request(
    data: OAuthData,
    key: SigningKey
) -> Result<HattrickData, Error> {
    chpp_request("players", "2.7", data, key).await
}
