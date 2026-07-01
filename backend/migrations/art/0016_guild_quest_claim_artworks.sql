CREATE TABLE IF NOT EXISTS guild_quest_claim_artworks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  claim_id INTEGER NOT NULL,
  quest_id INTEGER NOT NULL,
  uid TEXT NOT NULL,
  artwork_id INTEGER NOT NULL,
  submitted_at TEXT NOT NULL,
  UNIQUE(claim_id, artwork_id)
);

CREATE INDEX IF NOT EXISTS idx_guild_quest_claim_artworks_claim
  ON guild_quest_claim_artworks(claim_id);

CREATE INDEX IF NOT EXISTS idx_guild_quest_claim_artworks_quest_uid
  ON guild_quest_claim_artworks(quest_id, uid);

CREATE INDEX IF NOT EXISTS idx_guild_quest_claim_artworks_artwork
  ON guild_quest_claim_artworks(artwork_id);
