CREATE TABLE league_units (
  id INTEGER PRIMARY KEY NOT NULL,
  download_id INTEGER NOT NULL,
  unit_id INTEGER NOT NULL,
  unit_name TEXT NOT NULL,
  league_level INTEGER NOT NULL,
  max_number_of_teams INTEGER,
  current_match_round INTEGER,
  FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

CREATE TABLE league_unit_teams (
  id INTEGER PRIMARY KEY NOT NULL,
  download_id INTEGER NOT NULL,
  league_unit_id INTEGER NOT NULL,
  team_id INTEGER NOT NULL,
  team_name TEXT NOT NULL,
  position INTEGER NOT NULL,
  points INTEGER NOT NULL,
  matches_played INTEGER NOT NULL,
  goals_for INTEGER NOT NULL,
  goals_against INTEGER NOT NULL,
  won INTEGER NOT NULL,
  draws INTEGER NOT NULL,
  lost INTEGER NOT NULL,
  FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE,
  FOREIGN KEY (league_unit_id) REFERENCES league_units(id) ON DELETE CASCADE
);

CREATE TABLE matches (
  id INTEGER PRIMARY KEY NOT NULL,
  download_id INTEGER NOT NULL,
  match_id INTEGER NOT NULL,
  home_team_id INTEGER NOT NULL,
  home_team_name TEXT NOT NULL,
  away_team_id INTEGER NOT NULL,
  away_team_name TEXT NOT NULL,
  match_date TEXT NOT NULL,
  match_type INTEGER NOT NULL,
  status TEXT NOT NULL,
  home_goals INTEGER,
  away_goals INTEGER,
  FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
