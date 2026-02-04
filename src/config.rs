use std::env;

pub static VERSION: &str = "0.1.0";
pub static GETTEXT_PACKAGE: &str = "nutmeg";
pub static LOCALEDIR: &str = "/app/share/locale";
pub static PKGDATADIR: &str = "/app/share/nutmeg";

pub fn consumer_key() -> String {
    env::var("HT_CONSUMER_KEY").unwrap_or_default()
}

pub fn consumer_secret() -> String {
    env::var("HT_CONSUMER_SECRET").unwrap_or_default()
}
