CREATE TABLE staff (
    staff_id INTEGER PRIMARY KEY NOT NULL,
    team_id INTEGER NOT NULL,
    staff_type INTEGER NOT NULL, -- 0=Assistant, 1=Doctor, 2=Psychologist, 3=Form, 4=Financial, 5=Tactician
    staff_level INTEGER NOT NULL,
    hired_date TEXT NOT NULL,
    cost INTEGER NOT NULL,
    name TEXT NOT NULL,
    download_id INTEGER NOT NULL,
    FOREIGN KEY(download_id) REFERENCES downloads(id)
);
