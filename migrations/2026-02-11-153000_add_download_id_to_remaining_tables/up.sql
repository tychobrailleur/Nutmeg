-- Recreate cups with download_id
DROP TABLE IF EXISTS cups;
CREATE TABLE cups (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    league_level INTEGER,
    level INTEGER,
    level_index INTEGER,
    match_round INTEGER,
    match_rounds_left INTEGER,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

-- Recreate languages with download_id
DROP TABLE IF EXISTS languages;
CREATE TABLE languages (
    id INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);
