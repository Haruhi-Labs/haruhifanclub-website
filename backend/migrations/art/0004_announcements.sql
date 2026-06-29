-- 画廊社团公告：后台可发布/编辑/下线的真实公告，替代前端硬编码 mock。
-- 公开接口只读 status='published'；草稿仅后台可见。tags 复用 JSON 数组存储范式。
CREATE TABLE IF NOT EXISTS announcements (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    category     TEXT NOT NULL DEFAULT 'activity',   -- activity | maintenance
    title        TEXT NOT NULL,
    summary      TEXT NOT NULL DEFAULT '',
    body         TEXT NOT NULL DEFAULT '',
    tags_json    TEXT NOT NULL DEFAULT '[]',
    pinned       INTEGER NOT NULL DEFAULT 0,         -- 置顶
    status       TEXT NOT NULL DEFAULT 'published',  -- published | draft
    published_at TEXT,                               -- 对外展示日期（排序用）
    created_at   TEXT,
    updated_at   TEXT
);

-- 列表排序：置顶优先、再按发布时间倒序；公开/后台共用。
CREATE INDEX IF NOT EXISTS idx_announcements_list
    ON announcements(status, pinned, datetime(published_at));
