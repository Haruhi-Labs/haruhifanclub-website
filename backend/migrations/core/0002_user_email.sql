-- 给用户加邮箱：AI 审核拦截内容时，给管理员发邮件通知用（收件人 = 有邮箱的超管 ∪ 在该模块有角色且有邮箱的管理员）。
ALTER TABLE users ADD COLUMN email TEXT;
