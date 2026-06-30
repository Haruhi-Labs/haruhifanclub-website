ALTER TABLE guild_quests ADD COLUMN repeat_on_complete INTEGER NOT NULL DEFAULT 0;

UPDATE guild_quests
SET repeat_on_complete = 1
WHERE deadline_days IS NOT NULL
  AND fixed_deadline_at IS NULL
  AND cycle_days IS NOT NULL;
