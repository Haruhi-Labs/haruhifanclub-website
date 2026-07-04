//! 应用共享状态。

use std::sync::Arc;

use axum::extract::FromRef;
use haruhi_auth::{AuthSecret, CoreDb};
use haruhi_core::Config;
use haruhi_db::Pools;
use haruhi_mail::Mailer;

use crate::ratelimit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub pools: Pools,
    pub login_limiter: Arc<RateLimiter>,
    /// 匿名上传（art/exam）按 IP 限流，防资源滥用。
    pub upload_limiter: Arc<RateLimiter>,
    /// 注册 / 找回密码 / 重发验证邮件按 IP 限流，防刷邮件与账号枚举。
    pub account_limiter: Arc<RateLimiter>,
    /// 统一邮件发送器；未配置邮件时为 None（业务层据此把链接打日志降级）。
    pub mailer: Option<Mailer>,
    /// 资源站（download）：语雀知识库索引的内存缓存，后台定时同步、请求直接读。
    pub download: crate::modules::download::DownloadCache,
    /// SEO 注入：各 app dist/index.html 模板缓存（mtime 失效，部署免重启）。
    pub seo_templates: crate::modules::seo::template::SeoTemplates,
}

/// 让 auth 提取器能从 AppState 取到 JWT 密钥。
impl FromRef<AppState> for AuthSecret {
    fn from_ref(state: &AppState) -> Self {
        AuthSecret(Arc::new(state.cfg.jwt_secret.clone()))
    }
}

/// 让会话提取器能从 AppState 取到 core.db 连接池。
impl FromRef<AppState> for CoreDb {
    fn from_ref(state: &AppState) -> Self {
        CoreDb(state.pools.core.clone())
    }
}
