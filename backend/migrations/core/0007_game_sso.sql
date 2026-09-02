-- 游戏统一身份：Authorization Code + PKCE 的一次性授权码。
-- 只保存授权码 SHA-256；原值仅出现在浏览器跳转中，60 秒后失效且只能消费一次。
CREATE TABLE game_authorization_codes (
    token_hash     TEXT PRIMARY KEY,
    user_id        INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id      TEXT NOT NULL,
    redirect_uri   TEXT NOT NULL,
    code_challenge TEXT NOT NULL,
    expires_at     TEXT NOT NULL,
    consumed_at    TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_game_authorization_codes_expires
    ON game_authorization_codes(expires_at);
CREATE INDEX idx_game_authorization_codes_user
    ON game_authorization_codes(user_id, created_at);
