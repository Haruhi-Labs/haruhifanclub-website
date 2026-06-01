//! 应用共享状态。

use std::sync::Arc;

use axum::extract::FromRef;
use haruhi_auth::AuthSecret;
use haruhi_core::Config;
use haruhi_db::Pools;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub pools: Pools,
}

/// 让 auth 提取器能从 AppState 取到 JWT 密钥。
impl FromRef<AppState> for AuthSecret {
    fn from_ref(state: &AppState) -> Self {
        AuthSecret(Arc::new(state.cfg.jwt_secret.clone()))
    }
}
