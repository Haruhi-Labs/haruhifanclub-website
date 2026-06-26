-- 昵称地位等同用户名：必填、唯一、不可重名（全站对外署名的权威标识）。
-- 红线：只新增，绝不修改已应用迁移（sqlx SHA-384 校验）。

-- 1) 回填历史空昵称。
--    迁移前建立的老账号 nickname 可能为 NULL/空（彼时昵称非必填）。用 username 兜底：
--    username 全表唯一，且这些账号的 username 不与任何现有 nickname 冲突（已核对生产数据），
--    保证回填后不破坏下面的唯一索引。用户登录后可在「个人中心 → 个人资料」自行改名。
UPDATE users
   SET nickname = username
 WHERE trim(coalesce(nickname, '')) = '';

-- 2) 大小写不敏感的昵称唯一索引（lower(nickname)）。
--    沿用 email 索引同款「部分索引」风格，仅对非空昵称生效；回填后已无空昵称，
--    该条件主要是防御性，避免极端情况下空串/NULL 触发约束。
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_nickname_unique
    ON users(lower(nickname))
 WHERE nickname IS NOT NULL AND nickname <> '';
