-- 公会实体补给预算改为管理员手动补给台账；消耗仍由实体兑换审核自动统计。
CREATE TABLE IF NOT EXISTS guild_budget_supplies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_type  TEXT NOT NULL DEFAULT 'quarterly',
    amount_coins INTEGER NOT NULL,
    amount_input INTEGER NOT NULL,
    input_unit   TEXT NOT NULL DEFAULT 'coins',
    created_by   INTEGER,
    created_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_guild_budget_supplies_created
ON guild_budget_supplies(datetime(created_at) DESC, id DESC);
