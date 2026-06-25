-- 试卷创建接入统一账号：exams 补登录用户外键（逻辑引用 core.users.id）。
-- 旧试卷 author_user_id 为 NULL；持有 edit_token 者可经 /api/exam/claim 认领到本人名下。
ALTER TABLE exams ADD COLUMN author_user_id INTEGER;
CREATE INDEX IF NOT EXISTS idx_exams_author_user ON exams(author_user_id);
