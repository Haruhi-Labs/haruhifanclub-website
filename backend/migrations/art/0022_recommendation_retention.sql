CREATE INDEX IF NOT EXISTS idx_recommendation_events_created
ON recommendation_events(datetime(created_at));

CREATE TABLE IF NOT EXISTS recommendation_maintenance (
    id              INTEGER PRIMARY KEY CHECK(id = 1),
    last_cleanup_at TEXT
);

INSERT OR IGNORE INTO recommendation_maintenance(id, last_cleanup_at) VALUES(1, NULL);
