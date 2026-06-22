-- 资讯投稿接入统一账号：文章补登录用户外键（逻辑引用 core.users.id）。
-- 旧投稿 author_user_id 为 NULL（历史游客投稿，前端标识为只读历史）。复用 author 文本列作署名快照。
ALTER TABLE articles ADD COLUMN author_user_id INTEGER;
CREATE INDEX IF NOT EXISTS idx_articles_author_user ON articles(author_user_id);
