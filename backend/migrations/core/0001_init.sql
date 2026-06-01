-- core.db：统一鉴权与 RBAC
-- 模型：每个用户在每个 app 至多拥有一个角色；角色带数值等级。
-- 权限判定（在代码中）：action 需要的最小等级 <= 用户在该 app 的角色等级；超管全通过。

CREATE TABLE IF NOT EXISTS users (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    username       TEXT    NOT NULL UNIQUE,
    password_hash  TEXT    NOT NULL,
    display_name   TEXT,
    is_super_admin INTEGER NOT NULL DEFAULT 0,
    status         TEXT    NOT NULL DEFAULT 'active',  -- active | disabled
    created_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    last_login_at  TEXT
);

-- 角色：viewer(1) < editor(2) < moderator(3) < admin(4)
CREATE TABLE IF NOT EXISTS roles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key         TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    level       INTEGER NOT NULL,
    description TEXT
);

-- 用户在某 app 的角色（按应用分配权限的核心表）
CREATE TABLE IF NOT EXISTS user_app_roles (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app     TEXT    NOT NULL,  -- news | art | exam | novel | shop | console
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, app)
);

-- 敏感操作审计
CREATE TABLE IF NOT EXISTS audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER,
    app        TEXT,
    action     TEXT,
    target     TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_app_roles_user ON user_app_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id, created_at);

-- 预置角色
INSERT OR IGNORE INTO roles (key, name, level, description) VALUES
    ('viewer',    '只读',   1, '仅查看后台数据'),
    ('editor',    '编辑',   2, '内容增删改'),
    ('moderator', '审核员', 3, '审核/通过/拒绝内容'),
    ('admin',     '管理员', 4, '模块内全部管理权限');
