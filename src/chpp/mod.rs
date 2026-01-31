static CHPP_OAUTH_REQUEST_TOKEN_URL: &str = "https://chpp.hattrick.org/oauth/request_token.ashx";
static CHPP_OAUTH_AUTH_URL: &str = "https://chpp.hattrick.org/oauth/authorize.aspx";
static CHPP_OAUTH_ACCESS_TOKEN_URL: &str = "https://chpp.hattrick.org/oauth/access_token.ashx";
static CHPP_URL: &str = "https://chpp.hattrick.org/chppxml.ashx";
static HOCTANE_USER_AGENT: &str = "HOv8.0";

pub mod error;
pub mod model;
pub mod oauth;
pub mod request;

pub use oauth::OAuthSettings;
pub use request::request_token;
