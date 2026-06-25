-- 画廊 UGC 接入统一账号：作品/评论补登录用户外键（逻辑引用 core.users.id，跨库无 DB 级 FK）。
-- 旧匿名行 author_user_id 为 NULL（前端标“历史游客内容”），可经 /api/art/claim 用旧匿名 cookie 认领。
-- 复用既有 uploader_name/user_name 文本列作署名快照；uploader_uid 对登录用户取 "u{id}"，沿用积分/创作者/点赞体系。
ALTER TABLE artworks ADD COLUMN author_user_id INTEGER;
ALTER TABLE comments ADD COLUMN author_user_id INTEGER;
CREATE INDEX IF NOT EXISTS idx_artworks_author_user ON artworks(author_user_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_user ON comments(author_user_id);
