-- Add indexes to accelerate "latest by business key" queries.
-- These indexes are purely for read performance; they do not change any schema.

CREATE INDEX IF NOT EXISTS idx_league_units_unit_id
    ON league_units (unit_id, download_id DESC);

CREATE INDEX IF NOT EXISTS idx_matches_match_id
    ON matches (match_id, download_id DESC);

CREATE INDEX IF NOT EXISTS idx_matches_home_team
    ON matches (home_team_id, download_id DESC);

CREATE INDEX IF NOT EXISTS idx_matches_away_team
    ON matches (away_team_id, download_id DESC);

CREATE INDEX IF NOT EXISTS idx_players_download
    ON players (download_id);

CREATE INDEX IF NOT EXISTS idx_teams_download
    ON teams (download_id);
