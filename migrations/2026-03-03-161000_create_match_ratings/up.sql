CREATE TABLE IF NOT EXISTS match_ratings (
    match_id   INTEGER NOT NULL,
    team_id    INTEGER NOT NULL,
    formation  TEXT,
    tactic_type INTEGER,
    rating_midfield  REAL,
    rating_right_def REAL,
    rating_mid_def   REAL,
    rating_left_def  REAL,
    rating_right_att REAL,
    rating_mid_att   REAL,
    rating_left_att  REAL,
    PRIMARY KEY (match_id, team_id)
);
