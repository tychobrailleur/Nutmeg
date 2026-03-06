-- Migrate league_units, league_unit_teams, and matches to composite PKs.
-- All three tables previously used surrogate INTEGER PRIMARY KEY; we replace
-- them with business-key composite PKs aligned with the insert-only schema.

-- ── league_units: composite PK (unit_id, download_id) ──────────────────────
CREATE TABLE league_units_new (
    unit_id             INTEGER NOT NULL,
    download_id         INTEGER NOT NULL,
    unit_name           TEXT    NOT NULL,
    league_level        INTEGER NOT NULL,
    max_number_of_teams INTEGER,
    current_match_round INTEGER,
    PRIMARY KEY (unit_id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
INSERT INTO league_units_new
    SELECT unit_id, download_id, unit_name, league_level,
           max_number_of_teams, current_match_round
    FROM league_units;

-- ── league_unit_teams: composite PK (unit_id, team_id, download_id) ─────────
-- Recover unit_id from old league_units via the surrogate league_unit_id FK
-- before the old league_units table is dropped.
CREATE TABLE league_unit_teams_new (
    unit_id        INTEGER NOT NULL,
    team_id        INTEGER NOT NULL,
    download_id    INTEGER NOT NULL,
    team_name      TEXT    NOT NULL,
    position       INTEGER NOT NULL,
    points         INTEGER NOT NULL,
    matches_played INTEGER NOT NULL,
    goals_for      INTEGER NOT NULL,
    goals_against  INTEGER NOT NULL,
    won            INTEGER NOT NULL,
    draws          INTEGER NOT NULL,
    lost           INTEGER NOT NULL,
    PRIMARY KEY (unit_id, team_id, download_id),
    FOREIGN KEY (unit_id, download_id) REFERENCES league_units_new(unit_id, download_id) ON DELETE CASCADE,
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
INSERT INTO league_unit_teams_new
    SELECT lu.unit_id, lut.team_id, lut.download_id, lut.team_name, lut.position,
           lut.points, lut.matches_played, lut.goals_for, lut.goals_against,
           lut.won, lut.draws, lut.lost
    FROM league_unit_teams lut
    JOIN league_units lu ON lu.id = lut.league_unit_id;

-- ── matches: composite PK (match_id, download_id) ───────────────────────────
CREATE TABLE matches_new (
    match_id         INTEGER NOT NULL,
    download_id      INTEGER NOT NULL,
    home_team_id     INTEGER NOT NULL,
    home_team_name   TEXT    NOT NULL,
    away_team_id     INTEGER NOT NULL,
    away_team_name   TEXT    NOT NULL,
    match_date       TEXT    NOT NULL,
    match_type       INTEGER NOT NULL,
    status           TEXT    NOT NULL,
    home_goals       INTEGER,
    away_goals       INTEGER,
    match_context_id INTEGER,
    PRIMARY KEY (match_id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
INSERT INTO matches_new
    SELECT match_id, download_id, home_team_id, home_team_name, away_team_id,
           away_team_name, match_date, match_type, status, home_goals, away_goals,
           match_context_id
    FROM matches
    WHERE id IN (
        -- Keep only the row with the highest surrogate id (= most recent download)
        -- for each (match_id, download_id) pair to satisfy the new composite PK.
        SELECT MAX(id)
        FROM matches
        GROUP BY match_id, download_id
    );

-- ── Drop old tables (child tables first) and rename new ones ─────────────────
DROP TABLE league_unit_teams;
ALTER TABLE league_unit_teams_new RENAME TO league_unit_teams;

DROP TABLE matches;
ALTER TABLE matches_new RENAME TO matches;

DROP TABLE league_units;
ALTER TABLE league_units_new RENAME TO league_units;
