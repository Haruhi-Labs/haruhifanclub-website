-- art guild: coins, reputation, quests, rewards, public profiles.

CREATE TABLE IF NOT EXISTS coins_ledger (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    uid         TEXT NOT NULL,
    artwork_id  INTEGER,
    coins       INTEGER NOT NULL,
    note        TEXT,
    source_type TEXT DEFAULT 'legacy',
    created_at  TEXT,
    granted_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_coins_ledger_uid ON coins_ledger(uid, datetime(created_at));

INSERT INTO coins_ledger(uid, artwork_id, coins, note, source_type, created_at, granted_at)
SELECT uid, artwork_id, points, note, 'legacy_points', created_at, granted_at
FROM points_ledger
WHERE NOT EXISTS (SELECT 1 FROM coins_ledger WHERE source_type = 'legacy_points');

CREATE TABLE IF NOT EXISTS reputation_ledger (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    uid         TEXT NOT NULL,
    quest_id    INTEGER,
    artwork_id  INTEGER,
    reputation  INTEGER NOT NULL,
    note        TEXT,
    source_type TEXT DEFAULT 'manual',
    created_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_reputation_ledger_uid ON reputation_ledger(uid, datetime(created_at));

CREATE TABLE IF NOT EXISTS guild_profiles (
    uid         TEXT PRIMARY KEY,
    user_id     INTEGER,
    reputation  INTEGER NOT NULL DEFAULT 0,
    rating      TEXT NOT NULL DEFAULT 'F',
    access_tier TEXT NOT NULL DEFAULT 'observer_clearance',
    created_at  TEXT,
    updated_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_guild_profiles_rating ON guild_profiles(rating, reputation);

INSERT OR IGNORE INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at)
SELECT uid, 0, 'F', 'observer_clearance', COALESCE(created_at, CURRENT_TIMESTAMP), COALESCE(created_at, CURRENT_TIMESTAMP)
FROM creators
WHERE uid IS NOT NULL AND TRIM(uid) <> '';

CREATE TABLE IF NOT EXISTS guild_quests (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    title               TEXT NOT NULL,
    description         TEXT,
    quest_type          TEXT NOT NULL DEFAULT 'daily',
    difficulty          TEXT NOT NULL DEFAULT 'normal',
    required_rating     TEXT NOT NULL DEFAULT 'F',
    required_access     TEXT NOT NULL DEFAULT 'observer_clearance',
    condition_kind      TEXT NOT NULL DEFAULT 'upload_personal_haruhi',
    target_count        INTEGER NOT NULL DEFAULT 1,
    reward_reputation   INTEGER NOT NULL DEFAULT 0,
    reward_coins        INTEGER NOT NULL DEFAULT 0,
    deadline_hours      INTEGER,
    status              TEXT NOT NULL DEFAULT 'active',
    sort_order          INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT,
    updated_at          TEXT
);
CREATE INDEX IF NOT EXISTS idx_guild_quests_status ON guild_quests(status, sort_order);

INSERT OR IGNORE INTO guild_quests(
    id, title, description, quest_type, difficulty, required_rating, required_access,
    condition_kind, target_count, reward_reputation, reward_coins, deadline_hours,
    status, sort_order, created_at, updated_at
) VALUES
    (1, '每日观测：浏览3个画廊作品', '接取后浏览任意 3 个公开画廊作品。', 'daily', 'normal', 'F', 'observer_clearance', 'browse_artworks', 3, 15, 0, NULL, 'active', 10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (2, '每日回声：评论1个作品', '接取后在任意作品详情留下 1 条评论。', 'daily', 'normal', 'F', 'observer_clearance', 'comment_artworks', 1, 20, 0, NULL, 'active', 20, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (3, 'SOS团特别委托：上传凉宫作品', '接取后投稿并通过审核 1 张凉宫个人作品。', 'limited', 'hard', 'E', 'observer_clearance', 'upload_personal_haruhi', 1, 180, 30, 72, 'active', 30, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS guild_quest_claims (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    quest_id        INTEGER NOT NULL,
    uid             TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',
    progress        INTEGER NOT NULL DEFAULT 0,
    target_count    INTEGER NOT NULL DEFAULT 1,
    claimed_at      TEXT,
    completed_at    TEXT,
    rewarded_at     TEXT,
    UNIQUE(quest_id, uid)
);
CREATE INDEX IF NOT EXISTS idx_guild_claims_uid ON guild_quest_claims(uid, status);

CREATE TABLE IF NOT EXISTS guild_quest_events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    uid         TEXT NOT NULL,
    event_kind  TEXT NOT NULL,
    target_id   INTEGER,
    created_at  TEXT,
    UNIQUE(uid, event_kind, target_id)
);
CREATE INDEX IF NOT EXISTS idx_guild_events_uid ON guild_quest_events(uid, event_kind, datetime(created_at));

CREATE TABLE IF NOT EXISTS guild_rating_applications (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    uid                  TEXT NOT NULL,
    from_rating          TEXT,
    target_rating        TEXT NOT NULL,
    reputation_snapshot  INTEGER NOT NULL DEFAULT 0,
    haruhi_count_snapshot INTEGER NOT NULL DEFAULT 0,
    status               TEXT NOT NULL DEFAULT 'pending',
    user_note            TEXT,
    admin_note           TEXT,
    created_at           TEXT,
    reviewed_at          TEXT
);
CREATE INDEX IF NOT EXISTS idx_guild_rating_apps ON guild_rating_applications(status, datetime(created_at));

CREATE TABLE IF NOT EXISTS guild_rewards (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    description     TEXT,
    reward_type     TEXT NOT NULL DEFAULT 'virtual',
    price_coins     INTEGER NOT NULL DEFAULT 0,
    stock           INTEGER,
    required_rating TEXT NOT NULL DEFAULT 'F',
    required_access TEXT NOT NULL DEFAULT 'observer_clearance',
    image_url       TEXT,
    status          TEXT NOT NULL DEFAULT 'active',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT,
    updated_at      TEXT
);
CREATE INDEX IF NOT EXISTS idx_guild_rewards_status ON guild_rewards(status, sort_order);

INSERT OR IGNORE INTO guild_rewards(
    id, name, description, reward_type, price_coins, stock, required_rating, required_access,
    image_url, status, sort_order, created_at, updated_at
) VALUES
    (1, 'SOS团电子徽章', '个人主页展示用的虚拟徽章。', 'badge', 80, NULL, 'F', 'observer_clearance', '', 'active', 10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (2, '画廊应援明信片', '实体奖励，需在备注中填写联系方式或领取方式。', 'physical', 300, 20, 'E', 'observer_clearance', '', 'active', 20, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS guild_reward_redemptions (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    reward_id      INTEGER NOT NULL,
    uid            TEXT NOT NULL,
    frozen_coins   INTEGER NOT NULL DEFAULT 0,
    status         TEXT NOT NULL DEFAULT 'pending',
    user_note      TEXT,
    admin_note     TEXT,
    created_at     TEXT,
    reviewed_at    TEXT,
    fulfilled_at   TEXT
);
CREATE INDEX IF NOT EXISTS idx_guild_redemptions_uid ON guild_reward_redemptions(uid, status, datetime(created_at));
