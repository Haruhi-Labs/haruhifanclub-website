ALTER TABLE artworks ADD COLUMN exhibit_enabled INTEGER;

CREATE INDEX IF NOT EXISTS idx_artworks_creator_exhibit
ON artworks(author_user_id, exhibit_enabled, status, source_type);
