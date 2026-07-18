-- 地方支部采用能力 + 实例作用域授权；既有 user_app_roles 保持不变。
CREATE TABLE IF NOT EXISTS capability_grants (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    capability   TEXT    NOT NULL,
    scope_type   TEXT    NOT NULL CHECK (scope_type IN ('platform', 'branch')),
    scope_id     TEXT    NOT NULL,
    granted_by   INTEGER REFERENCES users(id) ON DELETE SET NULL,
    expires_at   TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (user_id, capability, scope_type, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_capability_grants_user
    ON capability_grants(user_id, scope_type, scope_id);
CREATE INDEX IF NOT EXISTS idx_capability_grants_scope
    ON capability_grants(scope_type, scope_id, capability);
