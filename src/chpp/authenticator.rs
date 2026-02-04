/* authenticator.rs
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

use crate::service::secret::{GnomeSecretService, SecretStorageService};
use gtk::glib;
use oauth_1a::{ClientId, ClientSecret, OAuthData, SigningKey, Token};
use std::env;
use std::io::{self, Write};

use crate::chpp::oauth::{
    create_oauth_context, exchange_verification_code, request_token, OauthSettings,
};
use crate::chpp::request::team_details_request;

// This file is useful to do a full end to end test of the CHPP OAuth flow.

fn prompt_browser(url: &str) -> i32 {
    println!("Opening browser...");
    println!("If it doesn't open, please visit this URL manually:");
    println!("{}", url);
    let _ = open::that(url);
    0
}

pub fn perform_cli_auth() -> glib::ExitCode {
    let consumer_key =
        env::var("HT_CONSUMER_KEY").expect("HT_CONSUMER_KEY not set env or .env/.zshrc");
    let consumer_secret =
        env::var("HT_CONSUMER_SECRET").expect("HT_CONSUMER_SECRET not set env or .env/.zshrc");

    println!("Starting CHPP Authentication Flow...");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let secret_service = GnomeSecretService::new();

    let maybe_creds = rt.block_on(async {
        let token = secret_service
            .get_secret("access_token")
            .await
            .ok()
            .flatten();
        let secret = secret_service
            .get_secret("access_secret")
            .await
            .ok()
            .flatten();

        match (token, secret) {
            (Some(t), Some(s)) => {
                println!("Credentials found in Keyring.");
                Some((t, s))
            }
            _ => None,
        }
    });

    let (access_token, access_secret) = match maybe_creds {
        Some(creds) => creds,
        None => {
            println!("No credentials found in keyring. Starting browser authentication.");
            // Get Request Token and authorize
            let settings = match request_token(
                OauthSettings::default(),
                &consumer_key,
                &consumer_secret,
                |url| prompt_browser(url),
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error requesting token: {:?}", e);
                    return glib::ExitCode::FAILURE;
                }
            };

            println!("Please enter the verification code from the browser:");
            let mut verification_code = String::new();
            if let Err(e) = io::stdout().flush() {
                eprintln!("Failed to flush stdout: {}", e);
            }
            match io::stdin().read_line(&mut verification_code) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to read line: {}", e);
                    return glib::ExitCode::FAILURE;
                }
            }
            let verification_code = verification_code.trim();

            // Exchange for Access Token
            println!("Exchanging verification code: {}", verification_code);
            match exchange_verification_code(verification_code, &settings) {
                Ok((t, s)) => {
                    println!("Access Token: {}", t);
                    println!("Access Secret: {}", s);

                    // Store credentials
                    rt.block_on(async {
                        let ss = GnomeSecretService::new();
                        if let Err(e) = ss.store_secret("access_token", &t).await {
                            eprintln!("Warning: Failed to save access token: {}", e);
                        }
                        if let Err(e) = ss.store_secret("access_secret", &s).await {
                            eprintln!("Warning: Failed to save access secret: {}", e);
                        } else {
                            println!("Credentials saved to Keyring.");
                        }
                    });

                    (t, s)
                }
                Err(e) => {
                    eprintln!("Error exchanging verification code: {:?}", e);
                    return glib::ExitCode::FAILURE;
                }
            }
        }
    };

    // Prepare for team details request
    let (data, key) = create_oauth_context(
        &consumer_key,
        &consumer_secret,
        &access_token,
        &access_secret,
    );

    // Execute async request (reuse runtime)

    match rt.block_on(team_details_request(data, key, Some(281726))) {
        Ok(data) => {
            println!("Successfully retrieved team details!");
            // println!("{:#?}", data);

            let team = &data.Teams.Teams[0];
            let team_id_str = &team.TeamID;
            println!("Fetched Team: {} ({})", team.TeamName, team_id_str);

            if let Ok(team_id) = team_id_str.parse::<u32>() {
                println!("Fetching players for TeamID: {}", team_id);

                // Re-create context for the second request since OAuthData is not Clone
                let (data2, key2) = create_oauth_context(
                    &consumer_key,
                    &consumer_secret,
                    &access_token,
                    &access_secret,
                );

                match rt.block_on(crate::chpp::request::players_request(
                    data2,
                    key2,
                    Some(team_id),
                )) {
                    Ok(players_data) => {
                        println!("Successfully retrieved players!");
                        let team_w_players = &players_data.Team;
                        if let Some(player_list) = &team_w_players.PlayerList {
                            println!("Found {} players.", player_list.players.len());
                            for p in &player_list.players {
                                println!("- {} {} (ID: {})", p.FirstName, p.LastName, p.PlayerID);
                            }
                        } else {
                            println!("No PlayerList found in response.");
                        }
                    }
                    Err(e) => eprintln!("Error fetching players: {:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching team details: {:?}", e);
            // If error is 401, maybe delete credentials? But for now user asked to store it.
            return glib::ExitCode::FAILURE;
        }
    }

    glib::ExitCode::SUCCESS
}
