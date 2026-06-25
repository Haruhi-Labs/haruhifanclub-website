-- 统一用户级登录系统：把 users 从「纯后台账号」升级为「终端用户 + 管理员」统一身份。
-- 管理员 = 拥有 user_app_roles 角色的 user；终端用户 = 无后台角色的 user。
-- 终端用户注册时 username = email（email 唯一 → username 仍唯一，免去重建 NOT NULL 的 username 列）。
-- 红线：只新增列/表/索引，绝不修改已有迁移文件（sqlx SHA-384 校验）。

-- users 扩展：终端用户资料 + 邮箱验证 + 软删除
ALTER TABLE users ADD COLUMN email_verified INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN nickname   TEXT;
ALTER TABLE users ADD COLUMN avatar     TEXT;
ALTER TABLE users ADD COLUMN bio        TEXT;
ALTER TABLE users ADD COLUMN deleted_at TEXT;

-- email 唯一（部分索引：允许多个 NULL，给无邮箱的历史管理员留活路）
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique ON users(email) WHERE email IS NOT NULL;

-- 服务端会话：cookie 内是不透明随机串，此处 id 存其 sha256（DB 泄露不暴露活跃会话）。
-- 登出/改密 = 删行 → 立即失效（无状态 JWT 做不到）。
CREATE TABLE IF NOT EXISTS sessions (
    id           TEXT    PRIMARY KEY,                         -- sha256(cookie 原值) 的 base64url
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    csrf_token   TEXT    NOT NULL,                            -- 双提交 CSRF 令牌
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    expires_at   TEXT    NOT NULL,
    last_seen_at TEXT,
    user_agent   TEXT,
    ip           TEXT
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

-- 邮箱验证 / 找回密码 一次性令牌：邮件里发原值，库存其 sha256。
CREATE TABLE IF NOT EXISTS user_tokens (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind       TEXT    NOT NULL,                              -- 'verify_email' | 'reset_password'
    token_hash TEXT    NOT NULL,
    expires_at TEXT    NOT NULL,
    used_at    TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_user_tokens_hash ON user_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_user_tokens_user ON user_tokens(user_id, kind);

-- ===== 以下为 2FA / Passkey 预留表（本期建表不接端点，Phase 4 再实现）=====
CREATE TABLE IF NOT EXISTS user_totp (
    user_id      INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret       TEXT    NOT NULL,
    enabled      INTEGER NOT NULL DEFAULT 0,
    confirmed_at TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS user_backup_codes (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash TEXT    NOT NULL,
    used_at   TEXT
);
CREATE TABLE IF NOT EXISTS user_passkeys (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id TEXT    NOT NULL UNIQUE,
    public_key    TEXT    NOT NULL,
    name          TEXT,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
