-- Change currencies.rate from REAL (f32) to DOUBLE PRECISION (f64) so that
-- Diesel's print-schema emits `Nullable<Double>` instead of `Nullable<Float>`,
-- matching the f64 used throughout the application layer.
CREATE TABLE currencies_new (
    id          INTEGER NOT NULL,
    download_id INTEGER NOT NULL,
    name        TEXT    NOT NULL,
    rate        DOUBLE PRECISION,
    symbol      TEXT,
    PRIMARY KEY (id, download_id),
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

INSERT INTO currencies_new (id, download_id, name, rate, symbol)
    SELECT id, download_id, name, rate, symbol
    FROM currencies;

DROP TABLE currencies;
ALTER TABLE currencies_new RENAME TO currencies;
