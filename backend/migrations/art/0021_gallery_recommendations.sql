-- 画廊推荐行为日志。匿名身份与登录账号同时保留，使登录后的当前设备可以沿用登录前偏好。
CREATE TABLE IF NOT EXISTS recommendation_events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER,
    anon_id     TEXT NOT NULL,
    session_id  TEXT,
    artwork_id  INTEGER NOT NULL,
    batch_id    TEXT,
    event_type  TEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT 'gallery',
    position    INTEGER,
    dwell_ms    INTEGER,
    created_at  TEXT NOT NULL,
    FOREIGN KEY (artwork_id) REFERENCES artworks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recommendation_events_user_created
ON recommendation_events(user_id, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_recommendation_events_anon_created
ON recommendation_events(anon_id, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_recommendation_events_artwork_type_created
ON recommendation_events(artwork_id, event_type, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_recommendation_events_batch
ON recommendation_events(batch_id, artwork_id);

-- 同一推荐批次中的同类行为只记一次，防止重复观察回调和连点污染画像。
CREATE UNIQUE INDEX IF NOT EXISTS idx_recommendation_events_batch_unique
ON recommendation_events(batch_id, artwork_id, event_type)
WHERE batch_id IS NOT NULL;
