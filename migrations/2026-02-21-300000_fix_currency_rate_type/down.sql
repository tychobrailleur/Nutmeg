-- Restore currencies.rate to REAL (f32).
CREATE TABLE currencies_old (
    id          INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name        TEXT    NOT NULL,
    rate        REAL,
    symbol      TEXT,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

INSERT INTO currencies_old (id, download_id, name, rate, symbol)
    SELECT id, download_id, name, rate, symbol
    FROM currencies;

DROP TABLE currencies;
ALTER TABLE currencies_old RENAME TO currencies;
