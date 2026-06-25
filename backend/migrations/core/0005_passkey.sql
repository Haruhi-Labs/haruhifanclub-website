-- Passkey / WebAuthn 落地：扩展 user_passkeys（沿用 0003 预留表），新增 ceremony 状态表。
-- 约定：user_passkeys.public_key 列存放「序列化后的 webauthn-rs Passkey（JSON）」，
--      即完整凭据（含公钥与签名计数器），登录验证后回写以更新计数器。
ALTER TABLE user_passkeys ADD COLUMN last_used_at TEXT;
ALTER TABLE user_passkeys ADD COLUMN transports TEXT;
ALTER TABLE user_passkeys ADD COLUMN backed_up INTEGER NOT NULL DEFAULT 0;

-- WebAuthn 注册/登录 ceremony 的中间状态（start 与 finish 之间持久化），短期有效。
CREATE TABLE IF NOT EXISTS webauthn_states (
    id         TEXT    PRIMARY KEY,  -- 随机 flow id，返回前端，finish 时带回
    kind       TEXT    NOT NULL,     -- 'reg' | 'auth'
    user_id    INTEGER,              -- reg 时为发起用户；auth（无用户名）时为 NULL
    state      TEXT    NOT NULL,     -- 序列化的 webauthn-rs 状态（注册/鉴别）
    expires_at TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webauthn_states_expires ON webauthn_states(expires_at);
