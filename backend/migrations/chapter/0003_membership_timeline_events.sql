-- Chapter 正式会员、活动现场时间线与活动运营数据。
-- 所有变更均为追加式；旧 branch_posts 保留用于兼容和回滚。

ALTER TABLE branch_events ADD COLUMN format TEXT NOT NULL DEFAULT 'in_person'
    CHECK (format IN ('in_person','online','hybrid'));
ALTER TABLE branch_events ADD COLUMN registration_mode TEXT NOT NULL DEFAULT 'none'
    CHECK (registration_mode IN ('none','internal','external','both'));
ALTER TABLE branch_events ADD COLUMN admission_mode TEXT NOT NULL DEFAULT 'automatic'
    CHECK (admission_mode IN ('automatic','review'));
ALTER TABLE branch_events ADD COLUMN capacity INTEGER CHECK (capacity IS NULL OR capacity > 0);
ALTER TABLE branch_events ADD COLUMN registration_opens_at TEXT;
ALTER TABLE branch_events ADD COLUMN registration_closes_at TEXT;

UPDATE branch_events
SET registration_mode = CASE
    WHEN registration_url IS NOT NULL AND TRIM(registration_url) <> '' THEN 'external'
    ELSE 'none'
END;

CREATE TABLE IF NOT EXISTS branch_memberships (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id  INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    user_id    INTEGER NOT NULL,
    state      TEXT    NOT NULL DEFAULT 'active'
                       CHECK (state IN ('active','leave_requested','ended','removed')),
    joined_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    ended_at   TEXT,
    ended_by   INTEGER,
    end_reason TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ended_at 为空即占用全 Chapter 唯一会员槽；数据库约束兜住并发加入。
CREATE UNIQUE INDEX IF NOT EXISTS uq_branch_memberships_active_user
    ON branch_memberships(user_id) WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_branch_memberships_branch
    ON branch_memberships(branch_id, ended_at, joined_at);

CREATE TABLE IF NOT EXISTS branch_membership_leave_requests (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    membership_id INTEGER NOT NULL REFERENCES branch_memberships(id) ON DELETE CASCADE,
    reason        TEXT    NOT NULL,
    state         TEXT    NOT NULL DEFAULT 'pending'
                          CHECK (state IN ('pending','approved','rejected','cancelled')),
    requested_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    reviewed_by   INTEGER,
    review_note   TEXT,
    reviewed_at   TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_branch_leave_request_pending
    ON branch_membership_leave_requests(membership_id) WHERE state = 'pending';

CREATE UNIQUE INDEX IF NOT EXISTS uq_branch_members_user
    ON branch_members(branch_id, user_id) WHERE user_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS branch_event_timeline_entries (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id        INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    event_id         INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    title            TEXT    NOT NULL,
    content          TEXT,
    image_path       TEXT,
    happened_at      TEXT    NOT NULL,
    location_name    TEXT,
    status           TEXT    NOT NULL DEFAULT 'draft'
                              CHECK (status IN ('draft','published','deleted')),
    moderation_state TEXT    NOT NULL DEFAULT 'normal'
                              CHECK (moderation_state IN ('normal','withheld')),
    created_by       INTEGER,
    updated_by       INTEGER,
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_branch_timeline_public
    ON branch_event_timeline_entries(status, moderation_state, happened_at, id);
CREATE INDEX IF NOT EXISTS idx_branch_timeline_event
    ON branch_event_timeline_entries(event_id, happened_at, id);

CREATE TABLE IF NOT EXISTS branch_event_topics (
    event_id   INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    topic      TEXT    NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (event_id, topic)
);

CREATE TABLE IF NOT EXISTS branch_event_people (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id    INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    role        TEXT    NOT NULL DEFAULT 'speaker'
                        CHECK (role IN ('speaker','host','facilitator','volunteer')),
    name        TEXT    NOT NULL,
    title       TEXT,
    organization TEXT,
    avatar_path TEXT,
    bio         TEXT,
    url         TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS branch_event_partners (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id    INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    partner_type TEXT   NOT NULL DEFAULT 'community'
                         CHECK (partner_type IN ('community','venue','sponsor','media','other')),
    name        TEXT    NOT NULL,
    logo_path   TEXT,
    url         TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS branch_event_cohosts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id     INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    branch_id    INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    state        TEXT    NOT NULL DEFAULT 'pending'
                         CHECK (state IN ('pending','accepted','rejected','revoked')),
    invited_by   INTEGER NOT NULL,
    invited_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    responded_by INTEGER,
    responded_at TEXT,
    UNIQUE (event_id, branch_id)
);
CREATE INDEX IF NOT EXISTS idx_branch_event_cohosts_branch
    ON branch_event_cohosts(branch_id, state, event_id);

CREATE TABLE IF NOT EXISTS branch_event_registration_questions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id    INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    question_type TEXT  NOT NULL CHECK (question_type IN ('short_text','single','multiple')),
    label       TEXT    NOT NULL,
    required    INTEGER NOT NULL DEFAULT 0,
    options_json TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS branch_event_registrations (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id         INTEGER NOT NULL REFERENCES branch_events(id) ON DELETE CASCADE,
    user_id          INTEGER NOT NULL,
    state            TEXT    NOT NULL
                              CHECK (state IN ('pending','confirmed','waitlisted','rejected','cancelled')),
    answers_json     TEXT,
    public_mode      TEXT    NOT NULL DEFAULT 'named'
                              CHECK (public_mode IN ('named','anonymous')),
    anonymous_number INTEGER NOT NULL,
    review_note      TEXT,
    reviewed_by      INTEGER,
    reviewed_at      TEXT,
    checked_in_at    TEXT,
    checked_in_by    INTEGER,
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (event_id, user_id),
    UNIQUE (event_id, anonymous_number)
);
CREATE INDEX IF NOT EXISTS idx_branch_event_registrations_state
    ON branch_event_registrations(event_id, state, created_at);
