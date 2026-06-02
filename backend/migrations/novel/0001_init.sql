-- novel.db：书库（与旧 haruhi-novel-reader 同构，幂等）
CREATE TABLE IF NOT EXISTS books (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    author      TEXT DEFAULT '佚名',
    cover_path  TEXT,
    file_path   TEXT,
    upload_date TEXT,
    category    TEXT,
    sort_order  REAL
);
