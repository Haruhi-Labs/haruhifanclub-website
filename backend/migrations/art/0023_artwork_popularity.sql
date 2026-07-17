-- 作品有效浏览：同一账号/匿名设备在同一 30 分钟桶内只计一次。
-- 长期保留，供一周、一年和历史人气榜使用；推荐行为日志仍按自身保留策略清理。
CREATE TABLE IF NOT EXISTS artwork_views (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    artwork_id  INTEGER NOT NULL,
    actor_key   TEXT NOT NULL,
    view_bucket INTEGER NOT NULL,
    viewed_at   TEXT NOT NULL,
    FOREIGN KEY (artwork_id) REFERENCES artworks(id) ON DELETE CASCADE,
    UNIQUE(artwork_id, actor_key, view_bucket)
);

CREATE INDEX IF NOT EXISTS idx_artwork_views_artwork_viewed
ON artwork_views(artwork_id, datetime(viewed_at) DESC);

CREATE INDEX IF NOT EXISTS idx_artwork_views_viewed_artwork
ON artwork_views(datetime(viewed_at) DESC, artwork_id);
