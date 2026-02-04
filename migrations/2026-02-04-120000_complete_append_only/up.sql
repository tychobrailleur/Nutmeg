-- Recreate Users
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    login_name TEXT NOT NULL,
    supporter_tier TEXT NOT NULL,
    signup_date TEXT,
    activation_date TEXT,
    last_login_date TEXT,
    has_manager_license BOOLEAN,
    language_id INTEGER,
    language_name TEXT,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Recreate Currencies
DROP TABLE IF EXISTS currencies;
CREATE TABLE currencies (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    rate REAL,
    symbol TEXT,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Recreate Countries
DROP TABLE IF EXISTS countries;
CREATE TABLE countries (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    currency_id INTEGER,
    country_code TEXT,
    date_format TEXT,
    time_format TEXT,
    flag TEXT,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Recreate Regions
DROP TABLE IF EXISTS regions;
CREATE TABLE regions (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    country_id INTEGER NOT NULL,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Recreate Leagues
DROP TABLE IF EXISTS leagues;
CREATE TABLE leagues (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    country_id INTEGER,
    short_name TEXT,
    continent TEXT,
    season INTEGER,
    season_offset INTEGER,
    match_round INTEGER,
    zone_name TEXT,
    english_name TEXT,
    language_id INTEGER,
    national_team_id INTEGER,
    u20_team_id INTEGER,
    active_teams INTEGER,
    active_users INTEGER,
    number_of_levels INTEGER,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
