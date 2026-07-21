-- 作品浏览与画廊访问统一使用签名匿名 Cookie，并按 10 分钟滚动会话去重。
-- artwork_views 继续只保存实际计数事件；本表保存每个设备访问每件作品的会话状态。
CREATE TABLE IF NOT EXISTS artwork_view_visitors (
    artwork_id    INTEGER NOT NULL,
    anon_id       TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at  TEXT NOT NULL,
    visit_count   INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (artwork_id, anon_id),
    FOREIGN KEY (artwork_id) REFERENCES artworks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_artwork_view_visitors_last_seen
ON artwork_view_visitors(datetime(last_seen_at));

-- 尽可能承接现有匿名浏览的最后访问时间，避免升级后立即重复计数。
INSERT OR IGNORE INTO artwork_view_visitors(
    artwork_id,
    anon_id,
    first_seen_at,
    last_seen_at,
    visit_count
)
SELECT
    artwork_id,
    substr(actor_key, 6),
    MIN(viewed_at),
    MAX(viewed_at),
    COUNT(1)
FROM artwork_views
WHERE actor_key LIKE 'anon:%' AND length(actor_key) > 5
GROUP BY artwork_id, actor_key;
