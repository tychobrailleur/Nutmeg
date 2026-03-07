-- SQLite does not support ALTER TABLE DROP PRIMARY KEY.
-- We must create a new table, copy data, drop the old table, and rename.

CREATE TABLE staff_new (
    staff_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    staff_type INTEGER NOT NULL,
    staff_level INTEGER NOT NULL,
    hired_date TEXT NOT NULL,
    cost INTEGER NOT NULL,
    name TEXT NOT NULL,
    download_id INTEGER NOT NULL,
    PRIMARY KEY (staff_id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

INSERT INTO staff_new SELECT * FROM staff;

DROP TABLE staff;

ALTER TABLE staff_new RENAME TO staff;
