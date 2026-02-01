-- Create Users table
CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  login_name TEXT NOT NULL,
  supporter_tier TEXT NOT NULL,
  signup_date TEXT,
  activation_date TEXT,
  last_login_date TEXT,
  has_manager_license BOOLEAN,
  language_id INTEGER,
  language_name TEXT
);

-- Create Reference Tables
CREATE TABLE countries (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL
);

CREATE TABLE regions (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  country_id INTEGER NOT NULL REFERENCES countries(id)
);

CREATE TABLE leagues (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  country_id INTEGER REFERENCES countries(id)
);

CREATE TABLE cups (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  league_level INTEGER,
  level INTEGER,
  level_index INTEGER,
  match_round INTEGER,
  match_rounds_left INTEGER
);

-- Recreate Teams table with Foreign Keys
-- We rename the old one to migrate data if needed, but for now we DROP since it's dev.
DROP TABLE teams;

CREATE TABLE teams (
  id INTEGER PRIMARY KEY NOT NULL,
  user_id INTEGER REFERENCES users(id),
  name TEXT NOT NULL,
  raw_data TEXT NOT NULL,
  short_name TEXT,
  is_primary_club BOOLEAN,
  founded_date TEXT,
  arena_id INTEGER, 
  arena_name TEXT,
  league_id INTEGER REFERENCES leagues(id),
  league_name TEXT,
  country_id INTEGER REFERENCES countries(id),
  country_name TEXT,
  region_id INTEGER REFERENCES regions(id),
  region_name TEXT,
  homepage TEXT,
  dress_uri TEXT,
  dress_alternate_uri TEXT,
  logo_url TEXT,
  trainer_id INTEGER,
  cup_still_in BOOLEAN,
  cup_id INTEGER REFERENCES cups(id),
  cup_name TEXT,
  cup_league_level INTEGER,
  cup_level INTEGER,
  cup_level_index INTEGER,
  cup_match_round INTEGER,
  cup_match_rounds_left INTEGER,
  power_rating_global INTEGER,
  power_rating_league INTEGER,
  power_rating_region INTEGER,
  power_rating_indiv INTEGER,
  friendly_team_id INTEGER,
  league_level_unit_id INTEGER,
  league_level_unit_name TEXT,
  league_level INTEGER,
  number_of_victories INTEGER,
  number_of_undefeated INTEGER,
  number_of_visits INTEGER,
  team_rank INTEGER,
  fanclub_id INTEGER,
  fanclub_name TEXT,
  fanclub_size INTEGER,
  color_background TEXT,
  color_primary TEXT,
  is_bot BOOLEAN,
  bot_since TEXT,
  youth_team_id INTEGER,
  youth_team_name TEXT
);
