UPDATE guild_quest_claims
SET status = 'expired'
WHERE status = 'active'
  AND id NOT IN (
      SELECT MAX(id)
      FROM guild_quest_claims
      WHERE status = 'active'
      GROUP BY quest_id, uid
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_guild_claims_one_active_per_quest_user
ON guild_quest_claims(quest_id, uid)
WHERE status = 'active';
