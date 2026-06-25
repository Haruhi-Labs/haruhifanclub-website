-- 团报积分兑换记录：用户用团报积分兑换 prizes 的订单流水。
-- user_id 形如 "u{core_user_id}"，与 users.id / points_history.user_id 对齐（跨库无 DB 级 FK）。
-- 旧站无此表，纯新增，遵守「迁移文件只增不改」红线。
CREATE TABLE IF NOT EXISTS redemptions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     TEXT NOT NULL,
    prize_id    INTEGER NOT NULL,
    prize_name  TEXT,
    points_cost INTEGER NOT NULL DEFAULT 0,
    status      TEXT NOT NULL DEFAULT 'pending',  -- pending | fulfilled | cancelled
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_redemptions_user ON redemptions(user_id);
CREATE INDEX IF NOT EXISTS idx_redemptions_prize ON redemptions(prize_id);
