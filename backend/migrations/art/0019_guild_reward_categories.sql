CREATE TABLE IF NOT EXISTS guild_reward_categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT,
  updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_guild_reward_categories_status_sort
ON guild_reward_categories(status, sort_order, id);

ALTER TABLE guild_rewards ADD COLUMN category_id INTEGER;

CREATE INDEX IF NOT EXISTS idx_guild_rewards_category
ON guild_rewards(category_id);
