DROP TABLE teams;
-- Restore old teams table schema (simplified for down migration)
CREATE TABLE teams (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  raw_data TEXT NOT NULL,
  short_name TEXT,
  is_primary_club BOOLEAN,
  founded_date TEXT,
  arena_id INTEGER,
  arena_name TEXT,
  league_id INTEGER,
  league_name TEXT,
  country_id INTEGER,
  country_name TEXT,
  region_id INTEGER,
  region_name TEXT,
  homepage TEXT,
  dress_uri TEXT,
  dress_alternate_uri TEXT,
  logo_url TEXT,
  trainer_id INTEGER,
  cup_still_in BOOLEAN,
  cup_id INTEGER,
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

DROP TABLE cups;
DROP TABLE leagues;
DROP TABLE regions;
DROP TABLE countries;
DROP TABLE users;
