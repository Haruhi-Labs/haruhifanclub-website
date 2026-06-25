//! 顶层路由装配：/api/* 统一挂载，/uploads 静态服务，CORS 与日志层。

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderName, HeaderValue, Method};
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
use crate::{admin_routes, auth_routes, modules, passkey_routes, totp_routes};

pub fn router(state: AppState) -> Router {
    let uploads_dir = state.cfg.uploads_dir.clone();
    let cors = build_cors(&state.cfg);

    // /api 下：health + 鉴权 + 各业务模块
    let api = Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(ready))
        .merge(auth_routes::router())
        .merge(passkey_routes::router())
        .merge(totp_routes::router())
        .merge(admin_routes::router());
    // CSRF 中间件只罩 /api：写方法 + 带会话 cookie 时校验双提交 token。
    let api = modules::mount(api)
        .with_state(state)
        .layer(axum::middleware::from_fn(crate::middleware::csrf_guard));

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

/// readiness：探测核心库连通（SELECT 1），供负载/巡检判断是否真正可服务。
async fn ready(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    match sqlx::query("SELECT 1").execute(&state.pools.core).await {
        Ok(_) => Ok(Json(json!({ "status": "ready" }))),
        Err(_) => Err(axum::http::StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// CORS 来源策略：
/// - 显式配置 HARUHI_CORS_ORIGINS → 按白名单 **且开启 credentials**（跨域携带会话 cookie）；
/// - 未配置 + debug → 放开 Any（不带 credentials；本地前端经 Vite proxy 与 API 同源，无需跨域 cookie）；
/// - 未配置 + release → 收敛到 public_site_url（开启 credentials）。
///
/// 注意：CORS 规范下 `allow_credentials(true)` 不能与 `Any` 来源/头/方法并存，故白名单分支用显式集合。
fn build_cors(cfg: &Config) -> CorsLayer {
    let open = || {
        CorsLayer::new()
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_origin(Any)
    };

    let origins: Vec<String> = if !cfg.cors_origins.is_empty() {
        cfg.cors_origins.clone()
    } else if cfg!(debug_assertions) {
        return open();
    } else {
        vec![cfg.public_site_url.clone()]
    };

    let parsed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    if parsed.is_empty() {
        return open();
    }

    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_origin(parsed)
        .allow_credentials(true)
}
