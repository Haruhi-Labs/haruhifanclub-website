ALTER TABLE guild_quests ADD COLUMN cycle_days INTEGER;
ALTER TABLE guild_quests ADD COLUMN cycle_reset_hour INTEGER NOT NULL DEFAULT 4;
ALTER TABLE guild_quests ADD COLUMN deadline_days INTEGER;
ALTER TABLE guild_quests ADD COLUMN fixed_deadline_at TEXT;

UPDATE guild_quests
SET
    cycle_days = CASE WHEN quest_type = 'daily' THEN 1 ELSE NULL END,
    deadline_days = CASE
        WHEN deadline_hours IS NULL THEN NULL
        ELSE CAST((deadline_hours + 23) / 24 AS INTEGER)
    END
WHERE cycle_days IS NULL AND deadline_days IS NULL;

CREATE TABLE guild_quest_claims_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    quest_id        INTEGER NOT NULL,
    uid             TEXT NOT NULL,
    cycle_key       TEXT NOT NULL DEFAULT 'once',
    status          TEXT NOT NULL DEFAULT 'active',
    progress        INTEGER NOT NULL DEFAULT 0,
    target_count    INTEGER NOT NULL DEFAULT 1,
    claimed_at      TEXT,
    cycle_start_at  TEXT,
    cycle_end_at    TEXT,
    completed_at    TEXT,
    rewarded_at     TEXT,
    UNIQUE(quest_id, uid, cycle_key)
);

INSERT INTO guild_quest_claims_new(
    id, quest_id, uid, cycle_key, status, progress, target_count,
    claimed_at, cycle_start_at, cycle_end_at, completed_at, rewarded_at
)
SELECT
    c.id,
    c.quest_id,
    c.uid,
    CASE WHEN q.quest_type = 'daily' THEN 'legacy' ELSE 'once' END,
    CASE WHEN q.quest_type = 'daily' AND c.status = 'active' THEN 'expired' ELSE c.status END,
    c.progress,
    c.target_count,
    c.claimed_at,
    c.claimed_at,
    NULL,
    c.completed_at,
    c.rewarded_at
FROM guild_quest_claims c
LEFT JOIN guild_quests q ON q.id = c.quest_id;

DROP TABLE guild_quest_claims;
ALTER TABLE guild_quest_claims_new RENAME TO guild_quest_claims;
CREATE INDEX IF NOT EXISTS idx_guild_claims_uid ON guild_quest_claims(uid, status);
CREATE INDEX IF NOT EXISTS idx_guild_claims_cycle ON guild_quest_claims(quest_id, uid, cycle_key);

CREATE TABLE guild_quest_events_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    uid         TEXT NOT NULL,
    event_kind  TEXT NOT NULL,
    target_id   INTEGER,
    event_scope TEXT NOT NULL DEFAULT 'legacy',
    created_at  TEXT,
    UNIQUE(uid, event_kind, target_id, event_scope)
);

INSERT OR IGNORE INTO guild_quest_events_new(
    id, uid, event_kind, target_id, event_scope, created_at
)
SELECT
    id,
    uid,
    event_kind,
    target_id,
    'legacy',
    created_at
FROM guild_quest_events;

DROP TABLE guild_quest_events;
ALTER TABLE guild_quest_events_new RENAME TO guild_quest_events;
CREATE INDEX IF NOT EXISTS idx_guild_events_uid ON guild_quest_events(uid, event_kind, datetime(created_at));
CREATE INDEX IF NOT EXISTS idx_guild_events_scope ON guild_quest_events(uid, event_kind, event_scope);
