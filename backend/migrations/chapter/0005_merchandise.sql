-- 支部特色周边：仅用于公开陈列，不承载价格、库存或交易。
CREATE TABLE IF NOT EXISTS branch_merchandise (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id   INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    description TEXT,
    image_path  TEXT,
    tags_json   TEXT    NOT NULL DEFAULT '[]',
    status      TEXT    NOT NULL DEFAULT 'draft'
                        CHECK (status IN ('draft','published')),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_branch_merchandise_public
    ON branch_merchandise(branch_id, status, sort_order, id);
