-- 创作者公开档案留言板：与作品评论分表，避免留言被计入作品热度与评论统计。
CREATE TABLE IF NOT EXISTS creator_profile_messages (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    creator_uid    TEXT    NOT NULL,
    author_user_id INTEGER NOT NULL,
    user_name      TEXT    NOT NULL,
    body           TEXT    NOT NULL,
    created_at     TEXT    NOT NULL,
    status         TEXT    NOT NULL DEFAULT 'public',
    ai_reason      TEXT
);

CREATE INDEX IF NOT EXISTS idx_creator_profile_messages_profile
ON creator_profile_messages(creator_uid, status, datetime(created_at) DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_creator_profile_messages_author
ON creator_profile_messages(author_user_id, datetime(created_at) DESC);
