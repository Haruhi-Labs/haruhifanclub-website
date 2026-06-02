-- art.db：绘画部画廊（忠实照搬旧 haruhi-art-club/server/db.js，幂等）
CREATE TABLE IF NOT EXISTS creators (
    uid        TEXT PRIMARY KEY,
    avatar_url TEXT,
    created_at TEXT,
    qq         TEXT
);

CREATE TABLE IF NOT EXISTS artworks (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    title              TEXT,
    description        TEXT,
    uploader_name      TEXT,
    uploader_uid       TEXT,
    source_type        TEXT,
    content_type       TEXT,
    tags_json          TEXT,
    tags_norm          TEXT,
    origin_url         TEXT,
    file_path          TEXT,
    file_path_original TEXT,
    status             TEXT,
    review_note        TEXT,
    reviewed_at        TEXT,
    created_at         TEXT,
    licenses_json      TEXT,
    like_total         INTEGER DEFAULT 0,
    images_json        TEXT,
    ai_reason          TEXT
);

CREATE TABLE IF NOT EXISTS points_ledger (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    uid        TEXT,
    artwork_id INTEGER,
    points     INTEGER,
    note       TEXT,
    created_at TEXT,
    granted_at TEXT
);

CREATE TABLE IF NOT EXISTS comments (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    artwork_id INTEGER,
    anon_id    TEXT,
    user_name  TEXT,
    avatar_key INTEGER,
    body       TEXT,
    like_total INTEGER DEFAULT 0,
    created_at TEXT,
    status     TEXT DEFAULT 'public',
    ai_reason  TEXT
);
CREATE INDEX IF NOT EXISTS idx_comments_art ON comments(artwork_id, datetime(created_at));

CREATE TABLE IF NOT EXISTS likes_daily (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    anon_id     TEXT,
    target_type TEXT,
    target_id   INTEGER,
    day         TEXT,
    count       INTEGER,
    created_at  TEXT,
    updated_at  TEXT,
    UNIQUE(anon_id, target_type, target_id, day)
);
CREATE INDEX IF NOT EXISTS idx_likes_target ON likes_daily(target_type, target_id, day);

CREATE TABLE IF NOT EXISTS members (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    member_code_hash TEXT UNIQUE,
    is_active        INTEGER DEFAULT 1,
    created_at       TEXT
);
