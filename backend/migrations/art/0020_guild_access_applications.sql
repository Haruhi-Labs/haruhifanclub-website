CREATE TABLE IF NOT EXISTS guild_access_applications (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    uid              TEXT NOT NULL,
    from_access      TEXT,
    target_access    TEXT NOT NULL,
    artwork_ids_json TEXT NOT NULL DEFAULT '[]',
    status           TEXT NOT NULL DEFAULT 'pending',
    user_note        TEXT,
    admin_note       TEXT,
    created_at       TEXT,
    reviewed_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_guild_access_apps_status_created
ON guild_access_applications(status, datetime(created_at));

CREATE UNIQUE INDEX IF NOT EXISTS idx_guild_access_pending_uid
ON guild_access_applications(uid)
WHERE status = 'pending';
