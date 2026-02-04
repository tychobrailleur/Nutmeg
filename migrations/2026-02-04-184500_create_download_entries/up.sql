CREATE TABLE download_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    download_id INTEGER NOT NULL,
    endpoint TEXT NOT NULL,
    version TEXT NOT NULL,
    user_id INTEGER,
    status TEXT NOT NULL,
    fetched_date TEXT NOT NULL,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (download_id) REFERENCES downloads(id)
);

CREATE INDEX idx_download_entries_download_id ON download_entries(download_id);
CREATE INDEX idx_download_entries_status ON download_entries(status);
