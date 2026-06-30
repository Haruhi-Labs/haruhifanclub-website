UPDATE guild_rating_applications
SET
    status = 'rejected',
    admin_note = COALESCE(admin_note, '系统清理重复待审核评级申请'),
    reviewed_at = COALESCE(reviewed_at, CURRENT_TIMESTAMP)
WHERE status = 'pending'
  AND id NOT IN (
      SELECT MAX(id)
      FROM guild_rating_applications
      WHERE status = 'pending'
      GROUP BY uid
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_guild_rating_pending_uid
ON guild_rating_applications(uid)
WHERE status = 'pending';
