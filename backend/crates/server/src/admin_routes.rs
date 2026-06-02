//! 超管控制台后端：/api/admin/*（仅超级管理员可访问）。
//! 用户的增删改、改密、启用/停用、按 app 分配角色。

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{hash_password, require_super, AuthUser};
use haruhi_core::{AppError, AppResult};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

// 顶层应用 + news 的细粒度子作用域（拥有父级 "news" 角色即覆盖所有 news.* 子作用域）
const APPS: &[&str] = &[
    "news",
    "news.blog",
    "news.activity",
    "news.store",
    "news.points",
    "art",
    "exam",
    "novel",
    "shop",
    "console",
];

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users).post(create_user))
        .route(
            "/admin/users/{id}",
            axum::routing::patch(update_user).delete(delete_user),
        )
        .route("/admin/users/{id}/password", post(reset_password))
        .route("/admin/users/{id}/roles", axum::routing::put(set_roles))
        .route("/admin/roles", get(list_roles))
        .route("/admin/audit", get(list_audit))
}

async fn list_roles(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let rows: Vec<(i64, String, String, i64, Option<String>)> =
        sqlx::query_as("SELECT id, key, name, level, description FROM roles ORDER BY level")
            .fetch_all(&state.pools.core)
            .await?;
    let roles: Vec<Value> = rows
        .into_iter()
        .map(|(id, key, name, level, description)| {
            json!({ "id": id, "key": key, "name": name, "level": level, "description": description })
        })
        .collect();
    Ok(Json(json!({ "roles": roles, "apps": APPS })))
}

async fn list_users(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let urows: Vec<(
        i64,
        String,
        Option<String>,
        bool,
        String,
        String,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, username, display_name, is_super_admin, status, created_at, last_login_at \
             FROM users ORDER BY id",
    )
    .fetch_all(&state.pools.core)
    .await?;

    let roles: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT ua.user_id, ua.app, r.key FROM user_app_roles ua \
         JOIN roles r ON r.id = ua.role_id",
    )
    .fetch_all(&state.pools.core)
    .await?;

    let users: Vec<Value> = urows
        .into_iter()
        .map(
            |(id, username, display_name, is_super, status, created_at, last_login_at)| {
                let app_roles: BTreeMap<String, String> = roles
                    .iter()
                    .filter(|(uid, _, _)| *uid == id)
                    .map(|(_, app, key)| (app.clone(), key.clone()))
                    .collect();
                json!({
                    "id": id, "username": username, "displayName": display_name,
                    "isSuperAdmin": is_super, "status": status,
                    "createdAt": created_at, "lastLoginAt": last_login_at,
                    "roles": app_roles,
                })
            },
        )
        .collect();
    Ok(Json(json!({ "users": users })))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    password: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    is_super_admin: bool,
}

async fn create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateUser>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let username = req.username.trim();
    if username.is_empty() || req.password.len() < 6 {
        return Err(AppError::bad_request("用户名不能为空且密码至少 6 位"));
    }
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.pools.core)
        .await?;
    if exists.is_some() {
        return Err(AppError::conflict("用户名已存在"));
    }
    let hash = hash_password(&req.password)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, display_name, is_super_admin, status) \
         VALUES (?, ?, ?, ?, 'active') RETURNING id",
    )
    .bind(username)
    .bind(&hash)
    .bind(&req.display_name)
    .bind(req.is_super_admin)
    .fetch_one(&state.pools.core)
    .await?;
    audit(&state, user.id, "console", "create_user", &id.to_string()).await;
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
struct UpdateUser {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    is_super_admin: Option<bool>,
}

async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUser>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    if let Some(name) = req.display_name {
        sqlx::query("UPDATE users SET display_name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&state.pools.core)
            .await?;
    }
    if let Some(status) = req.status {
        if status != "active" && status != "disabled" {
            return Err(AppError::bad_request("status 仅支持 active|disabled"));
        }
        if id == user.id && status == "disabled" {
            return Err(AppError::bad_request("不能停用自己的账号"));
        }
        sqlx::query("UPDATE users SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&state.pools.core)
            .await?;
    }
    if let Some(is_super) = req.is_super_admin {
        if id == user.id && !is_super {
            return Err(AppError::bad_request("不能撤销自己的超管身份"));
        }
        sqlx::query("UPDATE users SET is_super_admin = ? WHERE id = ?")
            .bind(is_super)
            .bind(id)
            .execute(&state.pools.core)
            .await?;
    }
    audit(&state, user.id, "console", "update_user", &id.to_string()).await;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ResetPassword {
    password: String,
}

async fn reset_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<ResetPassword>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    if req.password.len() < 6 {
        return Err(AppError::bad_request("密码至少 6 位"));
    }
    let hash = hash_password(&req.password)?;
    let affected = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&hash)
        .bind(id)
        .execute(&state.pools.core)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::not_found("用户不存在"));
    }
    audit(
        &state,
        user.id,
        "console",
        "reset_password",
        &id.to_string(),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

async fn delete_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    if id == user.id {
        return Err(AppError::bad_request("不能删除自己的账号"));
    }
    let affected = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.pools.core)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::not_found("用户不存在"));
    }
    audit(&state, user.id, "console", "delete_user", &id.to_string()).await;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct SetRoles {
    /// app -> roleKey；缺省的 app 表示无角色。
    roles: BTreeMap<String, String>,
}

async fn set_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<SetRoles>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    // 校验 app 与 role 合法
    for (app, role_key) in &req.roles {
        if !APPS.contains(&app.as_str()) {
            return Err(AppError::bad_request(format!("未知应用: {app}")));
        }
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM roles WHERE key = ?")
            .bind(role_key)
            .fetch_optional(&state.pools.core)
            .await?;
        if exists.is_none() {
            return Err(AppError::bad_request(format!("未知角色: {role_key}")));
        }
    }
    let mut tx = state.pools.core.begin().await?;
    sqlx::query("DELETE FROM user_app_roles WHERE user_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (app, role_key) in &req.roles {
        sqlx::query(
            "INSERT INTO user_app_roles (user_id, app, role_id) \
             VALUES (?, ?, (SELECT id FROM roles WHERE key = ?))",
        )
        .bind(id)
        .bind(app)
        .bind(role_key)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    audit(&state, user.id, "console", "set_roles", &id.to_string()).await;
    Ok(Json(json!({ "ok": true })))
}

async fn list_audit(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let rows: Vec<(
        i64,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, user_id, app, action, target, created_at FROM audit_log \
             ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(&state.pools.core)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, user_id, app, action, target, created_at)| {
            json!({ "id": id, "userId": user_id, "app": app, "action": action, "target": target, "createdAt": created_at })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

/// 写审计日志（失败仅告警，不阻断主流程）。
async fn audit(state: &AppState, user_id: i64, app: &str, action: &str, target: &str) {
    let _ = sqlx::query("INSERT INTO audit_log (user_id, app, action, target) VALUES (?, ?, ?, ?)")
        .bind(user_id)
        .bind(app)
        .bind(action)
        .bind(target)
        .execute(&state.pools.core)
        .await
        .map_err(|e| tracing::warn!("写审计日志失败: {e}"));
}
