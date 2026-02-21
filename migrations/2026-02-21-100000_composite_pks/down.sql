-- Restore surrogate INTEGER PRIMARY KEY columns on league_units,
-- league_unit_teams, and matches.
--
-- NOTE: The surrogate league_unit_id FK values in league_unit_teams cannot be
-- perfectly recovered. We re-derive them by joining with the restored
-- league_units table (which gets new auto-increment IDs). This is sufficient
-- for a development rollback; a full re-sync restores correctness.

-- ── Restore league_units with surrogate PK ───────────────────────────────────
CREATE TABLE league_units_old (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    download_id         INTEGER NOT NULL,
    unit_id             INTEGER NOT NULL,
    unit_name           TEXT    NOT NULL,
    league_level        INTEGER NOT NULL,
    max_number_of_teams INTEGER,
    current_match_round INTEGER,
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
INSERT INTO league_units_old (download_id, unit_id, unit_name, league_level,
                               max_number_of_teams, current_match_round)
    SELECT download_id, unit_id, unit_name, league_level,
           max_number_of_teams, current_match_round
    FROM league_units;
DROP TABLE league_units;
ALTER TABLE league_units_old RENAME TO league_units;

-- ── Restore league_unit_teams with surrogate PK ──────────────────────────────
-- Re-derive league_unit_id by joining with the just-restored league_units.
CREATE TABLE league_unit_teams_old (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    download_id    INTEGER NOT NULL,
    league_unit_id INTEGER NOT NULL,
    team_id        INTEGER NOT NULL,
    team_name      TEXT    NOT NULL,
    position       INTEGER NOT NULL,
    points         INTEGER NOT NULL,
    matches_played INTEGER NOT NULL,
    goals_for      INTEGER NOT NULL,
    goals_against  INTEGER NOT NULL,
    won            INTEGER NOT NULL,
    draws          INTEGER NOT NULL,
    lost           INTEGER NOT NULL,
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE,
    FOREIGN KEY (league_unit_id) REFERENCES league_units(id) ON DELETE CASCADE
);
INSERT INTO league_unit_teams_old
    (download_id, league_unit_id, team_id, team_name, position, points,
     matches_played, goals_for, goals_against, won, draws, lost)
    SELECT lut.download_id, lu.id, lut.team_id, lut.team_name, lut.position,
           lut.points, lut.matches_played, lut.goals_for, lut.goals_against,
           lut.won, lut.draws, lut.lost
    FROM league_unit_teams lut
    JOIN league_units lu ON lu.unit_id = lut.unit_id
                        AND lu.download_id = lut.download_id;
DROP TABLE league_unit_teams;
ALTER TABLE league_unit_teams_old RENAME TO league_unit_teams;

-- ── Restore matches with surrogate PK ───────────────────────────────────────
CREATE TABLE matches_old (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    download_id      INTEGER NOT NULL,
    match_id         INTEGER NOT NULL,
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
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
INSERT INTO matches_old
    (download_id, match_id, home_team_id, home_team_name, away_team_id,
     away_team_name, match_date, match_type, status, home_goals, away_goals,
     match_context_id)
    SELECT download_id, match_id, home_team_id, home_team_name, away_team_id,
           away_team_name, match_date, match_type, status, home_goals, away_goals,
           match_context_id
    FROM matches;
DROP TABLE matches;
ALTER TABLE matches_old RENAME TO matches;
