-- 降低注册门槛：取消「邮箱验证」环节（验证邮件易被误判为垃圾邮件）。
-- 存量用户一律视为已验证；新注册用户在 auth_routes::register 中直接置 email_verified=1。
-- email_verified 列保留（兼容旧查询与未来可能恢复），但不再作为发布门槛。
UPDATE users SET email_verified = 1 WHERE email_verified = 0;
