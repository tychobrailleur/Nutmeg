-- Revert users
CREATE TABLE users_old (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  login_name TEXT NOT NULL,
  supporter_tier TEXT NOT NULL,
  signup_date TEXT,
  activation_date TEXT,
  last_login_date TEXT,
  has_manager_license BOOLEAN,
  language_id INTEGER,
  language_name TEXT
);
INSERT INTO users_old SELECT * FROM users;
DROP TABLE users;
ALTER TABLE users_old RENAME TO users;

-- Revert countries
CREATE TABLE countries_old (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL
);
INSERT INTO countries_old (id, name) SELECT id, name FROM countries;
DROP TABLE countries;
ALTER TABLE countries_old RENAME TO countries;

DROP TABLE currencies;
DROP TABLE languages;
