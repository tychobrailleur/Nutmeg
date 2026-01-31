use gtk::glib;
use std::env;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use oauth_1a::{OAuthData, SigningKey, ClientSecret, Token, ClientId};
use serde::{Deserialize, Serialize};

use crate::chpp::oauth::{OauthSettings, request_token, exchange_verification_code, create_oauth_context};
use crate::chpp::request::team_details_request;

// This file is useful to do a full end to end test of the CHPP OAuth flow.

const CREDENTIALS_FILE: &str = ".hoctane_token";

#[derive(Serialize, Deserialize, Debug)]
struct StoredCredentials {
    access_token: String,
    access_secret: String,
}

fn prompt_browser(url: &str) -> i32 {
    println!("Opening browser...");
    println!("If it doesn't open, please visit this URL manually:");
    println!("{}", url);
    let _ = open::that(url);
    0
}

fn save_credentials(access_token: &str, access_secret: &str) -> io::Result<()> {
    let creds = StoredCredentials {
        access_token: access_token.to_string(),
        access_secret: access_secret.to_string(),
    };
    let json = serde_json::to_string_pretty(&creds)?;
    let mut file = fs::File::create(CREDENTIALS_FILE)?;
    file.write_all(json.as_bytes())?;
    println!("Credentials saved to {}", CREDENTIALS_FILE);
    Ok(())
}

fn load_credentials() -> Option<(String, String)> {
    if Path::new(CREDENTIALS_FILE).exists() {
        match fs::read_to_string(CREDENTIALS_FILE) {
            Ok(content) => {
                match serde_json::from_str::<StoredCredentials>(&content) {
                    Ok(creds) => {
                        println!("Loaded credentials from {}", CREDENTIALS_FILE);
                        return Some((creds.access_token, creds.access_secret));
                    },
                    Err(_) => eprintln!("Failed to parse credentials file."),
                }
            },
            Err(_) => eprintln!("Failed to read credentials file."),
        }
    }
    None
}

pub fn perform_cli_auth() -> glib::ExitCode {
    let consumer_key = env::var("HT_CONSUMER_KEY").expect("HT_CONSUMER_KEY not set env or .env/.zshrc");
    let consumer_secret = env::var("HT_CONSUMER_SECRET").expect("HT_CONSUMER_SECRET not set env or .env/.zshrc");

    println!("Starting CHPP Authentication Flow...");

    let (access_token, access_secret) = if let Some((token, secret)) = load_credentials() {
        (token, secret)
    } else {
        println!("No inputs credentials found. Starting browser authentication.");
        let oauth_settings = request_token(
            OauthSettings::default(),
            &consumer_key,
            &consumer_secret,
            prompt_browser,
        );

        // Check request token is available.
        if oauth_settings.request_token.borrow().is_empty() {
            eprintln!("Error: Request token is empty. Auth flow failed.");
            return glib::ExitCode::FAILURE;
        }

        println!("Please enter the verification code from the browser:");
        let mut verification_code = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut verification_code).expect("Failed to read line");
        let verification_code = verification_code.trim();

        // Exchange for Access Token
        println!("Exchanging verification code: {}", verification_code);
        match exchange_verification_code(verification_code, &oauth_settings) {
            Ok((t, s)) => {
                println!("Access Token: {}", t);
                println!("Access Secret: {}", s);
                if let Err(e) = save_credentials(&t, &s) {
                    eprintln!("Warning: Failed to save credentials: {}", e);
                }
                (t, s)
            },
            Err(e) => {
                eprintln!("Error exchanging verification code: {:?}", e);
                return glib::ExitCode::FAILURE;
            }
        }
    };

    // Prepare for team details request
    let (data, key) = create_oauth_context(&consumer_key, &consumer_secret, &access_token, &access_secret);

    // Execute async request
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

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
                 let (data2, key2) = create_oauth_context(&consumer_key, &consumer_secret, &access_token, &access_secret);
                 
                 match rt.block_on(crate::chpp::request::players_request(data2, key2, Some(team_id))) {
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
                     },
                     Err(e) => eprintln!("Error fetching players: {:?}", e),
                 }
            }
        },
        Err(e) => {
            eprintln!("Error fetching team details: {:?}", e);
            // If error is 401, maybe delete credentials? But for now user asked to store it.
            return glib::ExitCode::FAILURE;
        }
    }

    glib::ExitCode::SUCCESS
}
