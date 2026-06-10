//! 应用共享状态。

use std::sync::Arc;

use axum::extract::FromRef;
use haruhi_auth::AuthSecret;
use haruhi_core::Config;
use haruhi_db::Pools;

use crate::ratelimit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub pools: Pools,
    pub login_limiter: Arc<RateLimiter>,
    /// 匿名上传（art/exam）按 IP 限流，防资源滥用。
    pub upload_limiter: Arc<RateLimiter>,
}

/// 让 auth 提取器能从 AppState 取到 JWT 密钥。
impl FromRef<AppState> for AuthSecret {
    fn from_ref(state: &AppState) -> Self {
        AuthSecret(Arc::new(state.cfg.jwt_secret.clone()))
    }
}
