//! 统一鉴权路由：/api/auth/login、/api/auth/me。
//! 登出由前端丢弃 token 实现（无状态 JWT）。

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{issue_token, verify_password, AuthUser};
use haruhi_core::{AppError, AppResult};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
}

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginReq>) -> AppResult<Json<Value>> {
    let row: Option<(i64, String, bool, String)> = sqlx::query_as(
        "SELECT id, password_hash, is_super_admin, status FROM users WHERE username = ?",
    )
    .bind(req.username.trim())
    .fetch_optional(&state.pools.core)
    .await?;

    let (id, hash, is_super, status) = row.ok_or(AppError::Unauthorized)?;
    if status != "active" {
        return Err(AppError::Forbidden);
    }
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized);
    }

    let token = issue_token(
        &state.cfg.jwt_secret,
        id,
        is_super,
        state.cfg.jwt_ttl_seconds,
    )?;
    sqlx::query("UPDATE users SET last_login_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(&state.pools.core)
        .await?;

    let profile = load_profile(&state, id).await?;
    Ok(Json(json!({ "token": token, "user": profile })))
}

async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let profile = load_profile(&state, user.id).await?;
    Ok(Json(profile))
}

/// 组装用户档案 + 各 app 权限矩阵。
async fn load_profile(state: &AppState, user_id: i64) -> AppResult<Value> {
    let urow: Option<(String, Option<String>, bool)> = sqlx::query_as(
        "SELECT username, display_name, is_super_admin FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(&state.pools.core)
    .await?;
    let (username, display_name, is_super) = urow.ok_or(AppError::Unauthorized)?;

    let roles: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT ua.app, r.key, r.name, r.level FROM user_app_roles ua \
         JOIN roles r ON r.id = ua.role_id WHERE ua.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&state.pools.core)
    .await?;

    let apps: Value = Value::Object(
        roles
            .into_iter()
            .map(|(app, key, name, level)| {
                (
                    app,
                    json!({ "role": key, "roleName": name, "level": level }),
                )
            })
            .collect(),
    );

    Ok(json!({
        "id": user_id,
        "username": username,
        "displayName": display_name,
        "isSuperAdmin": is_super,
        "apps": apps,
    }))
}
