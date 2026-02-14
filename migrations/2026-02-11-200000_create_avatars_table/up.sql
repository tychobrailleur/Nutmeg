CREATE TABLE avatars (
  player_id INTEGER NOT NULL,
  download_id INTEGER NOT NULL REFERENCES downloads(id),
  image BLOB NOT NULL,
  PRIMARY KEY (player_id, download_id)
);

ALTER TABLE players DROP COLUMN avatar;
