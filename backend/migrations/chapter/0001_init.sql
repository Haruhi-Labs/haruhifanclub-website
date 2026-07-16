-- Chapter 地方支部子站领域库。
CREATE TABLE IF NOT EXISTS branches (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    slug                    TEXT    NOT NULL UNIQUE,
    name                    TEXT    NOT NULL,
    short_name              TEXT,
    summary                 TEXT,
    about_text              TEXT,
    join_text               TEXT,
    country_code            TEXT    NOT NULL DEFAULT 'CN',
    region_code             TEXT,
    locality_name           TEXT,
    timezone                TEXT    NOT NULL DEFAULT 'Asia/Shanghai',
    founded_on              TEXT,
    status                  TEXT    NOT NULL DEFAULT 'draft'
                                     CHECK (status IN ('draft','active','paused','archived')),
    default_post_aggregate  INTEGER NOT NULL DEFAULT 1,
    default_event_aggregate INTEGER NOT NULL DEFAULT 1,
    created_at              TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_slug_aliases (
    slug       TEXT PRIMARY KEY,
    branch_id  INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_brand (
    branch_id       INTEGER PRIMARY KEY REFERENCES branches(id) ON DELETE CASCADE,
    logo_light_path TEXT,
    logo_dark_path  TEXT,
    logo_mark_path  TEXT,
    logo_alt        TEXT,
    cover_path      TEXT,
    cover_focal_x   REAL NOT NULL DEFAULT 0.5,
    cover_focal_y   REAL NOT NULL DEFAULT 0.5,
    tagline         TEXT,
    accent_key      TEXT NOT NULL DEFAULT 'blue'
                           CHECK (accent_key IN ('blue','red','amber','green','purple','cyan')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_sections (
    branch_id   INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    section_key TEXT    NOT NULL CHECK (section_key IN
        ('about','organization','members','posts','events','join','contact')),
    label       TEXT,
    enabled     INTEGER NOT NULL DEFAULT 1,
    visibility  TEXT    NOT NULL DEFAULT 'public' CHECK (visibility IN ('public','members','hidden')),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (branch_id, section_key)
);

CREATE TABLE IF NOT EXISTS branch_members (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id    INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    user_id      INTEGER,
    display_name TEXT    NOT NULL,
    avatar_path  TEXT,
    bio          TEXT,
    status       TEXT    NOT NULL DEFAULT 'active' CHECK (status IN ('active','alumni','inactive')),
    joined_on    TEXT,
    left_on      TEXT,
    is_public    INTEGER NOT NULL DEFAULT 1,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_qq_groups (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id         INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name              TEXT    NOT NULL,
    group_number      TEXT    NOT NULL,
    description       TEXT,
    audience_label    TEXT,
    join_url          TEXT,
    qr_image_path     TEXT,
    join_instructions TEXT,
    is_primary        INTEGER NOT NULL DEFAULT 0,
    status            TEXT    NOT NULL DEFAULT 'active' CHECK (status IN ('active','paused','archived')),
    sort_order        INTEGER NOT NULL DEFAULT 0,
    last_verified_at  TEXT,
    created_at        TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_contact_people (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id            INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    member_id            INTEGER REFERENCES branch_members(id) ON DELETE SET NULL,
    display_name         TEXT    NOT NULL,
    role_title           TEXT,
    responsibility       TEXT,
    is_primary           INTEGER NOT NULL DEFAULT 0,
    is_public            INTEGER NOT NULL DEFAULT 1,
    consent_confirmed_at TEXT,
    status               TEXT    NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    sort_order           INTEGER NOT NULL DEFAULT 0,
    created_at           TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_contact_methods (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id   INTEGER NOT NULL REFERENCES branch_contact_people(id) ON DELETE CASCADE,
    method_type TEXT    NOT NULL CHECK (method_type IN ('qq','email','phone','wechat','website','other')),
    label       TEXT,
    value       TEXT    NOT NULL,
    url         TEXT,
    is_public   INTEGER NOT NULL DEFAULT 1,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS organization_versions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id    INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    effective_on TEXT,
    ended_on     TEXT,
    state        TEXT    NOT NULL DEFAULT 'draft' CHECK (state IN ('draft','current','historical')),
    display_mode TEXT    NOT NULL DEFAULT 'tree' CHECK (display_mode IN ('tree','flat','summary','hidden')),
    summary      TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS organization_units (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    version_id  INTEGER NOT NULL REFERENCES organization_versions(id) ON DELETE CASCADE,
    parent_id   INTEGER REFERENCES organization_units(id) ON DELETE SET NULL,
    name        TEXT    NOT NULL,
    kind        TEXT    NOT NULL DEFAULT 'group',
    description TEXT,
    is_public   INTEGER NOT NULL DEFAULT 1,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS organization_assignments (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    version_id INTEGER NOT NULL REFERENCES organization_versions(id) ON DELETE CASCADE,
    unit_id    INTEGER REFERENCES organization_units(id) ON DELETE CASCADE,
    member_id  INTEGER NOT NULL REFERENCES branch_members(id) ON DELETE CASCADE,
    title      TEXT,
    term_start TEXT,
    term_end   TEXT,
    is_public  INTEGER NOT NULL DEFAULT 1,
    is_contact INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS branch_posts (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id        INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    slug             TEXT    NOT NULL,
    title            TEXT    NOT NULL,
    summary          TEXT,
    content          TEXT,
    cover_path       TEXT,
    status           TEXT    NOT NULL DEFAULT 'draft'
                              CHECK (status IN ('draft','scheduled','published','archived','deleted')),
    visibility       TEXT    NOT NULL DEFAULT 'public' CHECK (visibility IN ('public','members')),
    aggregate_mode   TEXT    NOT NULL DEFAULT 'inherit' CHECK (aggregate_mode IN ('inherit','include','exclude')),
    moderation_state TEXT    NOT NULL DEFAULT 'normal' CHECK (moderation_state IN ('normal','withheld')),
    published_at     TEXT,
    scheduled_at     TEXT,
    created_by       INTEGER,
    updated_by       INTEGER,
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (branch_id, slug)
);

CREATE TABLE IF NOT EXISTS branch_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id        INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    slug             TEXT    NOT NULL,
    title            TEXT    NOT NULL,
    summary          TEXT,
    content          TEXT,
    cover_path       TEXT,
    event_type       TEXT,
    venue_name       TEXT,
    address          TEXT,
    online_url       TEXT,
    starts_at        TEXT    NOT NULL,
    ends_at          TEXT,
    registration_url TEXT,
    status           TEXT    NOT NULL DEFAULT 'draft'
                              CHECK (status IN ('draft','scheduled','published','archived','deleted')),
    visibility       TEXT    NOT NULL DEFAULT 'public' CHECK (visibility IN ('public','members')),
    aggregate_mode   TEXT    NOT NULL DEFAULT 'inherit' CHECK (aggregate_mode IN ('inherit','include','exclude')),
    moderation_state TEXT    NOT NULL DEFAULT 'normal' CHECK (moderation_state IN ('normal','withheld')),
    published_at     TEXT,
    created_by       INTEGER,
    updated_by       INTEGER,
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (branch_id, slug)
);

CREATE TABLE IF NOT EXISTS branch_admin_handovers (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id    INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    from_user_id INTEGER,
    to_user_id   INTEGER NOT NULL,
    initiated_by INTEGER NOT NULL,
    note         TEXT,
    state        TEXT NOT NULL DEFAULT 'completed' CHECK (state IN ('pending','completed','cancelled')),
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS branch_audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id   INTEGER REFERENCES branches(id) ON DELETE SET NULL,
    actor_id    INTEGER,
    action      TEXT NOT NULL,
    entity_type TEXT,
    entity_id   TEXT,
    detail_json TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_branches_public ON branches(status, region_code, locality_name);
CREATE INDEX IF NOT EXISTS idx_branch_members_public ON branch_members(branch_id, is_public, sort_order);
CREATE INDEX IF NOT EXISTS idx_branch_contacts_public ON branch_contact_people(branch_id, status, is_public);
CREATE INDEX IF NOT EXISTS idx_branch_posts_public ON branch_posts(status, visibility, moderation_state, published_at);
CREATE INDEX IF NOT EXISTS idx_branch_events_public ON branch_events(status, visibility, moderation_state, starts_at);
CREATE INDEX IF NOT EXISTS idx_branch_audit ON branch_audit_log(branch_id, created_at);
