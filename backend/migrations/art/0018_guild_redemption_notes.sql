ALTER TABLE guild_reward_redemptions ADD COLUMN review_note TEXT;
ALTER TABLE guild_reward_redemptions ADD COLUMN fulfilled_note TEXT;

UPDATE guild_reward_redemptions
SET review_note = admin_note
WHERE review_note IS NULL
  AND admin_note IS NOT NULL
  AND reviewed_at IS NOT NULL
  AND status IN ('approved', 'rejected', 'cancelled');

UPDATE guild_reward_redemptions
SET fulfilled_note = admin_note
WHERE fulfilled_note IS NULL
  AND admin_note IS NOT NULL
  AND fulfilled_at IS NOT NULL
  AND status = 'fulfilled';
