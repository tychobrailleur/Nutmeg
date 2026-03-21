-- Add download_id to match_ratings so it follows the insert-only pattern.
-- The old PK was (match_id, team_id); the new PK is (match_id, team_id, download_id).

-- Ensure a legacy download record exists for data migrated from pre-download_id schema.
INSERT OR IGNORE INTO downloads (id, timestamp, status)
VALUES (0, '1970-01-01T00:00:00Z', 'legacy');
CREATE TABLE match_ratings_new (
    match_id         INTEGER NOT NULL,
    team_id          INTEGER NOT NULL,
    download_id      INTEGER NOT NULL,
    formation        TEXT,
    tactic_type      INTEGER,
    rating_midfield  REAL,
    rating_right_def REAL,
    rating_mid_def   REAL,
    rating_left_def  REAL,
    rating_right_att REAL,
    rating_mid_att   REAL,
    rating_left_att  REAL,
    PRIMARY KEY (match_id, team_id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Migrate existing rows; assign them download_id = 0 as a sentinel for legacy data.
-- In practice match_ratings are only written during active sessions, so this table
-- is almost certainly empty at migration time.
INSERT INTO match_ratings_new
    SELECT match_id, team_id, 0, formation, tactic_type,
           rating_midfield, rating_right_def, rating_mid_def, rating_left_def,
           rating_right_att, rating_mid_att, rating_left_att
    FROM match_ratings;

DROP TABLE match_ratings;
ALTER TABLE match_ratings_new RENAME TO match_ratings;
