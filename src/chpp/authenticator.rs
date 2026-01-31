use gtk::glib;
use std::env;
use std::io::{self, Write};
use oauth_1a::{OAuthData, SigningKey, ClientSecret, Token, ClientId, SignatureMethod, Nonce, TokenSecret};

use crate::chpp::oauth::{OauthSettings, request_token, exchange_verification_code, create_oauth_context};
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
    let consumer_key = env::var("HT_CONSUMER_KEY").expect("HT_CONSUMER_KEY not set env or .env/.zshrc");
    let consumer_secret = env::var("HT_CONSUMER_SECRET").expect("HT_CONSUMER_SECRET not set env or .env/.zshrc");

    println!("Starting CHPP Authentication Flow...");

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
        Ok((access_token, access_secret)) => {
            println!("Access Token: {}", access_token);
            println!("Access Secret: {}", access_secret);

            // Prepare for team details request
            let (data, key) = create_oauth_context(&consumer_key, &consumer_secret, &access_token, &access_secret);

            // Execute async request
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            match rt.block_on(team_details_request(data, key)) {
                Ok(data) => {
                    println!("Successfully retrieved team details!");
                    println!("{:#?}", data);
                },
                Err(e) => {
                    eprintln!("Error fetching team details: {:?}", e);
                    return glib::ExitCode::FAILURE;
                }
            }
        },
        Err(e) => {
            eprintln!("Error exchanging verification code: {:?}", e);
            return glib::ExitCode::FAILURE;
        }
    }

    glib::ExitCode::SUCCESS
}
