//! 顶层路由装配：/api/* 统一挂载，/uploads 静态服务，CORS 与日志层。

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{auth_routes, modules};

pub fn router(state: AppState) -> Router {
    let uploads_dir = state.cfg.uploads_dir.clone();

    // /api 下：health + 鉴权 + 各业务模块
    let api = Router::new()
        .route("/health", get(health))
        .merge(auth_routes::router());
    let api = modules::mount(api).with_state(state);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .nest("/api", api)
        .nest_service("/uploads", ServeDir::new(uploads_dir))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "haruhifanclub" }))
}
