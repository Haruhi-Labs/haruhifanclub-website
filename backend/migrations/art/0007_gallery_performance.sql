-- 画廊高频路径性能索引与访客统计缓存。
-- 这些索引覆盖公开列表、作者档案、公会任务进度、评论和后台账本的常用查询形态。
CREATE INDEX IF NOT EXISTS idx_artworks_status_created_id
ON artworks(status, datetime(created_at) DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_artworks_status_content_source_created_id
ON artworks(status, content_type, source_type, datetime(created_at) DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_artworks_uploader_status_created_id
ON artworks(uploader_uid, status, datetime(created_at) DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_artworks_uploader_source_content_status_reviewed
ON artworks(uploader_uid, source_type, content_type, status, datetime(COALESCE(reviewed_at, created_at)));

CREATE INDEX IF NOT EXISTS idx_artworks_status_like_created_id
ON artworks(
    status,
    COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0) DESC,
    datetime(created_at) DESC,
    id DESC
);

CREATE INDEX IF NOT EXISTS idx_comments_art_status_created
ON comments(artwork_id, status, datetime(created_at));

CREATE INDEX IF NOT EXISTS idx_comments_status_created
ON comments(status, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_points_ledger_uid_granted
ON points_ledger(uid, datetime(granted_at) DESC);

CREATE INDEX IF NOT EXISTS idx_points_ledger_granted
ON points_ledger(datetime(granted_at) DESC);

CREATE INDEX IF NOT EXISTS idx_creators_created
ON creators(datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_guild_quests_auto_active
ON guild_quests(status, auto_claim, sort_order);

CREATE TABLE IF NOT EXISTS art_visitor_stats (
    id              INTEGER PRIMARY KEY CHECK(id = 1),
    total_visits    INTEGER NOT NULL DEFAULT 0,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    updated_at      TEXT
);

INSERT OR REPLACE INTO art_visitor_stats(id, total_visits, unique_visitors, updated_at)
SELECT
    1,
    COALESCE(SUM(visit_count), 0),
    COUNT(*),
    CURRENT_TIMESTAMP
FROM art_visitors;
