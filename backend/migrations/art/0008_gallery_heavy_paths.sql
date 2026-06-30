-- 随机排序不再对每次请求按 id+seed 计算全量表达式排序。
-- 每个作品持久化一个随机键，列表请求只用 seed 选择环形起点并走范围索引分页。
ALTER TABLE artworks ADD COLUMN random_key INTEGER NOT NULL DEFAULT 0;

UPDATE artworks
SET random_key = ABS(((id * 1103515245 + 12345) % 2147483647)) + 1
WHERE random_key = 0;

CREATE INDEX IF NOT EXISTS idx_artworks_status_random_key
ON artworks(status, random_key, id);

CREATE INDEX IF NOT EXISTS idx_artworks_status_content_source_random_key
ON artworks(status, content_type, source_type, random_key, id);

CREATE INDEX IF NOT EXISTS idx_artworks_uploader_status_random_key
ON artworks(uploader_uid, status, random_key, id);

CREATE INDEX IF NOT EXISTS idx_artworks_random_key
ON artworks(random_key, id);
