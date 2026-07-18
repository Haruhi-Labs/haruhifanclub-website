-- 画廊社交关系：用户关注与作品收藏。

CREATE TABLE IF NOT EXISTS user_follows (
    follower_uid TEXT NOT NULL,
    followed_uid TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (follower_uid, followed_uid),
    CHECK (follower_uid <> followed_uid)
);

CREATE INDEX IF NOT EXISTS idx_user_follows_followed
ON user_follows(followed_uid, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_user_follows_follower
ON user_follows(follower_uid, datetime(created_at) DESC);

CREATE TABLE IF NOT EXISTS artwork_favorites (
    user_id    INTEGER NOT NULL,
    artwork_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, artwork_id),
    FOREIGN KEY (artwork_id) REFERENCES artworks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_artwork_favorites_artwork
ON artwork_favorites(artwork_id, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_artwork_favorites_user
ON artwork_favorites(user_id, datetime(created_at) DESC);
