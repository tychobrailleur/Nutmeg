CREATE TABLE languages (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL
);

CREATE TABLE currencies (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  rate REAL,
  symbol TEXT
);

-- Recreate countries to add currency_id FK
CREATE TABLE countries_new (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  currency_id INTEGER REFERENCES currencies(id)
);

INSERT INTO countries_new (id, name) SELECT id, name FROM countries;

DROP TABLE countries;
ALTER TABLE countries_new RENAME TO countries;

-- Recreate users to add language_id FK properly
CREATE TABLE users_new (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  login_name TEXT NOT NULL,
  supporter_tier TEXT NOT NULL,
  signup_date TEXT,
  activation_date TEXT,
  last_login_date TEXT,
  has_manager_license BOOLEAN,
  language_id INTEGER REFERENCES languages(id),
  language_name TEXT
);

INSERT INTO users_new SELECT * FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
