-- 投稿审核奖励规则：后台可配置积分/声望基础值与活动倍率。
CREATE TABLE IF NOT EXISTS art_reward_settings (
    id                           INTEGER PRIMARY KEY CHECK (id = 1),
    personal_haruhi_points       INTEGER NOT NULL DEFAULT 120,
    personal_other_points        INTEGER NOT NULL DEFAULT 30,
    personal_haruhi_reputation   INTEGER NOT NULL DEFAULT 120,
    personal_other_reputation    INTEGER NOT NULL DEFAULT 30,
    points_multiplier_bps        INTEGER NOT NULL DEFAULT 10000,
    reputation_multiplier_bps    INTEGER NOT NULL DEFAULT 10000,
    updated_at                   TEXT
);

INSERT OR IGNORE INTO art_reward_settings(
    id,
    personal_haruhi_points,
    personal_other_points,
    personal_haruhi_reputation,
    personal_other_reputation,
    points_multiplier_bps,
    reputation_multiplier_bps,
    updated_at
) VALUES (1, 120, 30, 120, 30, 10000, 10000, CURRENT_TIMESTAMP);

-- 作品首次通过时锁定当时的奖励规则，避免后续活动倍率追溯影响历史作品。
CREATE TABLE IF NOT EXISTS artwork_reward_snapshots (
    artwork_id                  INTEGER PRIMARY KEY,
    uid                         TEXT NOT NULL,
    source_type                 TEXT NOT NULL,
    content_type                TEXT NOT NULL,
    points_base                 INTEGER NOT NULL DEFAULT 0,
    points_multiplier_bps       INTEGER NOT NULL DEFAULT 10000,
    points_award                INTEGER NOT NULL DEFAULT 0,
    reputation_base             INTEGER NOT NULL DEFAULT 0,
    reputation_multiplier_bps   INTEGER NOT NULL DEFAULT 10000,
    reputation_award            INTEGER NOT NULL DEFAULT 0,
    created_at                  TEXT NOT NULL,
    updated_at                  TEXT
);

CREATE INDEX IF NOT EXISTS idx_artwork_reward_snapshots_uid
ON artwork_reward_snapshots(uid, datetime(created_at));

INSERT OR IGNORE INTO artwork_reward_snapshots(
    artwork_id,
    uid,
    source_type,
    content_type,
    points_base,
    points_multiplier_bps,
    points_award,
    reputation_base,
    reputation_multiplier_bps,
    reputation_award,
    created_at,
    updated_at
)
SELECT
    a.id,
    COALESCE(a.uploader_uid, ''),
    COALESCE(a.source_type, ''),
    COALESCE(a.content_type, ''),
    CASE
        WHEN COALESCE(a.source_type, '') = 'personal' AND COALESCE(a.content_type, '') = 'haruhi' THEN 120
        WHEN COALESCE(a.source_type, '') = 'personal' THEN 30
        ELSE 0
    END,
    10000,
    CASE
        WHEN COALESCE(a.source_type, '') = 'personal' AND COALESCE(a.content_type, '') = 'haruhi' THEN 120
        WHEN COALESCE(a.source_type, '') = 'personal' THEN 30
        ELSE 0
    END,
    CASE
        WHEN COALESCE(a.source_type, '') = 'personal' AND COALESCE(a.content_type, '') = 'haruhi' THEN 120
        WHEN COALESCE(a.source_type, '') = 'personal' THEN 30
        ELSE 0
    END,
    10000,
    CASE
        WHEN COALESCE(a.source_type, '') = 'personal' AND COALESCE(a.content_type, '') = 'haruhi' THEN 120
        WHEN COALESCE(a.source_type, '') = 'personal' THEN 30
        ELSE 0
    END,
    COALESCE(
        (
            SELECT MIN(COALESCE(pl.granted_at, pl.created_at))
            FROM points_ledger pl
            WHERE pl.artwork_id = a.id
        ),
        (
            SELECT MIN(rl.created_at)
            FROM reputation_ledger rl
            WHERE rl.artwork_id = a.id AND COALESCE(rl.source_type, '') = 'upload_artwork'
        ),
        a.created_at,
        CURRENT_TIMESTAMP
    ),
    CURRENT_TIMESTAMP
FROM artworks a
WHERE COALESCE(a.uploader_uid, '') <> ''
  AND (
      a.status = 'approved'
      OR EXISTS (
          SELECT 1 FROM points_ledger pl
          WHERE pl.artwork_id = a.id
      )
      OR EXISTS (
          SELECT 1 FROM reputation_ledger rl
          WHERE rl.artwork_id = a.id AND COALESCE(rl.source_type, '') = 'upload_artwork'
      )
  );
