DROP TABLE IF EXISTS cups;
CREATE TABLE cups (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    league_level INTEGER,
    level INTEGER,
    level_index INTEGER,
    match_round INTEGER,
    match_rounds_left INTEGER
);

DROP TABLE IF EXISTS languages;
CREATE TABLE languages (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);
