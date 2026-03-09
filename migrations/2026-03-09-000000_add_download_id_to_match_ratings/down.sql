CREATE TABLE match_ratings_old (
    match_id         INTEGER NOT NULL,
    team_id          INTEGER NOT NULL,
    formation        TEXT,
    tactic_type      INTEGER,
    rating_midfield  REAL,
    rating_right_def REAL,
    rating_mid_def   REAL,
    rating_left_def  REAL,
    rating_right_att REAL,
    rating_mid_att   REAL,
    rating_left_att  REAL,
    PRIMARY KEY (match_id, team_id)
);

INSERT INTO match_ratings_old
    SELECT match_id, team_id, formation, tactic_type,
           rating_midfield, rating_right_def, rating_mid_def, rating_left_def,
           rating_right_att, rating_mid_att, rating_left_att
    FROM match_ratings
    WHERE (match_id, team_id, download_id) IN (
        SELECT match_id, team_id, MAX(download_id)
        FROM match_ratings
        GROUP BY match_id, team_id
    );

DROP TABLE match_ratings;
ALTER TABLE match_ratings_old RENAME TO match_ratings;
