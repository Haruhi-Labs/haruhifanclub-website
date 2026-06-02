//! 顶层路由装配：/api/* 统一挂载，/uploads 静态服务，CORS 与日志层。

use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum::routing::get;
use axum::{Json, Router};
use haruhi_core::Config;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// 上传体积上限（EPUB / 图片 / 音频），覆盖 axum 默认 2MB。
const MAX_BODY_BYTES: usize = 256 * 1024 * 1024;

use crate::state::AppState;
use crate::{admin_routes, auth_routes, modules};

pub fn router(state: AppState) -> Router {
    let uploads_dir = state.cfg.uploads_dir.clone();
    let cors = build_cors(&state.cfg);

    // /api 下：health + 鉴权 + 各业务模块
    let api = Router::new()
        .route("/health", get(health))
        .merge(auth_routes::router())
        .merge(admin_routes::router());
    let api = modules::mount(api).with_state(state);

    Router::new()
        .nest("/api", api)
        .nest_service("/uploads", ServeDir::new(uploads_dir))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "haruhifanclub" }))
}

/// CORS：调试构建放宽(Any，方便本地)；release 限定来源（HARUHI_CORS_ORIGINS，默认仅 public_site_url）。
fn build_cors(cfg: &Config) -> CorsLayer {
    let base = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    if cfg!(debug_assertions) {
        return base.allow_origin(Any);
    }
    let mut origins = cfg.cors_origins.clone();
    if origins.is_empty() {
        origins.push(cfg.public_site_url.clone());
    }
    let parsed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    if parsed.is_empty() {
        base.allow_origin(Any)
    } else {
        base.allow_origin(parsed)
    }
}
