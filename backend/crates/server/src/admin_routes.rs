//! 超管控制台后端：/api/admin/*（仅超级管理员可访问）。
//! 用户的增删改、改密、启用/停用、按 app 分配角色。

use std::collections::BTreeMap;

use axum::extract::{Path, Query, State};
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
    "fiction",
    "voice",
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
        // 旧 ID 归属迁移：把历史匿名内容（author_user_id IS NULL）绑定到现有账户
        .route("/admin/migration/orphans", get(migration_orphans))
        .route("/admin/migration/bind", post(migration_bind))
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
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, username, display_name, is_super_admin, status, created_at, last_login_at, email \
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
            |(id, username, display_name, is_super, status, created_at, last_login_at, email)| {
                let app_roles: BTreeMap<String, String> = roles
                    .iter()
                    .filter(|(uid, _, _)| *uid == id)
                    .map(|(_, app, key)| (app.clone(), key.clone()))
                    .collect();
                json!({
                    "id": id, "username": username, "displayName": display_name,
                    "isSuperAdmin": is_super, "status": status, "email": email,
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
    email: Option<String>,
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
    // 与注册一致：邮箱统一转小写；空串视为不填
    let email = req
        .email
        .as_deref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    if let Some(ref e) = email {
        let taken: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
            .bind(e)
            .fetch_optional(&state.pools.core)
            .await?;
        if taken.is_some() {
            return Err(AppError::conflict("该邮箱已被其他账号使用"));
        }
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, display_name, email, is_super_admin, status) \
         VALUES (?, ?, ?, ?, ?, 'active') RETURNING id",
    )
    .bind(username)
    .bind(&hash)
    .bind(&req.display_name)
    .bind(&email)
    .bind(req.is_super_admin)
    .fetch_one(&state.pools.core)
    .await
    .map_err(map_email_conflict)?;
    audit(&state, user.id, "console", "create_user", &id.to_string()).await;
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
struct UpdateUser {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    email: Option<String>,
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
    if let Some(email) = req.email {
        // 与注册一致：邮箱统一转小写；空串视为清空（置 NULL）
        let email = email.trim().to_lowercase();
        if email.is_empty() {
            sqlx::query("UPDATE users SET email = NULL WHERE id = ?")
                .bind(id)
                .execute(&state.pools.core)
                .await?;
        } else {
            // 唯一性预检（排除自己），命中给友好 409 而非 500
            let taken: Option<(i64,)> =
                sqlx::query_as("SELECT id FROM users WHERE email = ? AND id != ?")
                    .bind(&email)
                    .bind(id)
                    .fetch_optional(&state.pools.core)
                    .await?;
            if taken.is_some() {
                return Err(AppError::conflict("该邮箱已被其他账号使用"));
            }
            sqlx::query("UPDATE users SET email = ? WHERE id = ?")
                .bind(&email)
                .bind(id)
                .execute(&state.pools.core)
                .await
                .map_err(map_email_conflict)?;
        }
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

// ============================================================
// 旧 ID 归属迁移（require_super）：把历史匿名内容绑定到现有账户
// ============================================================

#[derive(Deserialize)]
struct BindReq {
    module: String,
    ids: Vec<String>,
    #[serde(rename = "userId")]
    user_id: i64,
}

/// GET /api/admin/migration/orphans?module=art|news|exam&q=&page=
/// 列出某模块中尚未归属（author_user_id IS NULL）的历史内容，供管理员挑选迁移。
async fn migration_orphans(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let module = q.get("module").map(|s| s.as_str()).unwrap_or("art");
    let search = q.get("q").map(|s| s.trim()).unwrap_or("").to_string();
    let page: i64 = q
        .get("page")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);
    let page_size: i64 = 20;
    let offset = (page - 1) * page_size;
    let like = format!("%{search}%");

    let (total, items): (i64, Vec<Value>) = match module {
        "art" => {
            let cond = if search.is_empty() {
                "author_user_id IS NULL".to_string()
            } else {
                "author_user_id IS NULL AND (title LIKE ? OR uploader_name LIKE ? OR uploader_uid LIKE ?)".to_string()
            };
            let count_sql = format!("SELECT COUNT(*) FROM artworks WHERE {cond}");
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            if !search.is_empty() {
                cq = cq.bind(&like).bind(&like).bind(&like);
            }
            let total = cq.fetch_one(&state.pools.art).await?;
            let list_sql = format!("SELECT id, title, uploader_name, uploader_uid, created_at FROM artworks WHERE {cond} ORDER BY id DESC LIMIT ? OFFSET ?");
            let mut lq = sqlx::query_as::<
                _,
                (
                    i64,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                ),
            >(&list_sql);
            if !search.is_empty() {
                lq = lq.bind(&like).bind(&like).bind(&like);
            }
            let rows = lq
                .bind(page_size)
                .bind(offset)
                .fetch_all(&state.pools.art)
                .await?;
            let items = rows
                .into_iter()
                .map(|(id, title, uname, uuid, at)| {
                    json!({ "id": id.to_string(), "title": title.unwrap_or_default(), "by": uname.or(uuid), "at": at })
                })
                .collect();
            (total, items)
        }
        "news" => {
            let cond = if search.is_empty() {
                "author_user_id IS NULL".to_string()
            } else {
                "author_user_id IS NULL AND (title LIKE ? OR author LIKE ?)".to_string()
            };
            let count_sql = format!("SELECT COUNT(*) FROM articles WHERE {cond}");
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            if !search.is_empty() {
                cq = cq.bind(&like).bind(&like);
            }
            let total = cq.fetch_one(&state.pools.news).await?;
            let list_sql = format!("SELECT id, title, author, date FROM articles WHERE {cond} ORDER BY id DESC LIMIT ? OFFSET ?");
            let mut lq = sqlx::query_as::<_, (i64, Option<String>, Option<String>, Option<String>)>(
                &list_sql,
            );
            if !search.is_empty() {
                lq = lq.bind(&like).bind(&like);
            }
            let rows = lq
                .bind(page_size)
                .bind(offset)
                .fetch_all(&state.pools.news)
                .await?;
            let items = rows
                .into_iter()
                .map(|(id, title, author, date)| {
                    json!({ "id": id.to_string(), "title": title.unwrap_or_default(), "by": author, "at": date })
                })
                .collect();
            (total, items)
        }
        "exam" => {
            let cond = if search.is_empty() {
                "author_user_id IS NULL".to_string()
            } else {
                "author_user_id IS NULL AND title LIKE ?".to_string()
            };
            let count_sql = format!("SELECT COUNT(*) FROM exams WHERE {cond}");
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            if !search.is_empty() {
                cq = cq.bind(&like);
            }
            let total = cq.fetch_one(&state.pools.exam).await?;
            let list_sql = format!("SELECT id, title, edit_token, created_at FROM exams WHERE {cond} ORDER BY created_at DESC LIMIT ? OFFSET ?");
            let mut lq = sqlx::query_as::<
                _,
                (String, Option<String>, Option<String>, Option<String>),
            >(&list_sql);
            if !search.is_empty() {
                lq = lq.bind(&like);
            }
            let rows = lq
                .bind(page_size)
                .bind(offset)
                .fetch_all(&state.pools.exam)
                .await?;
            let items = rows
                .into_iter()
                .map(|(id, title, token, at)| {
                    let by = token.map(|t| format!("token:{}", &t[..t.len().min(8)]));
                    json!({ "id": id, "title": title.unwrap_or_default(), "by": by, "at": at })
                })
                .collect();
            (total, items)
        }
        _ => return Err(AppError::bad_request("无效模块")),
    };

    Ok(Json(json!({
        "module": module, "total": total, "page": page, "pageSize": page_size, "items": items
    })))
}

/// POST /api/admin/migration/bind { module, ids:[], userId }
/// 把选中的历史内容批量绑定到指定账户（仅设 author_user_id，且仅作用于尚未归属的行）。
async fn migration_bind(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<BindReq>,
) -> AppResult<Json<Value>> {
    require_super(&user)?;
    let table = match req.module.as_str() {
        "art" => "artworks",
        "news" => "articles",
        "exam" => "exams",
        _ => return Err(AppError::bad_request("无效模块")),
    };
    if req.ids.is_empty() {
        return Err(AppError::bad_request("未选择内容"));
    }
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE id = ? AND deleted_at IS NULL")
            .bind(req.user_id)
            .fetch_optional(&state.pools.core)
            .await?;
    if exists.is_none() {
        return Err(AppError::bad_request("目标用户不存在"));
    }

    let pool = match req.module.as_str() {
        "art" => &state.pools.art,
        "news" => &state.pools.news,
        _ => &state.pools.exam,
    };
    let placeholders = req.ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "UPDATE {table} SET author_user_id = ? WHERE id IN ({placeholders}) AND author_user_id IS NULL"
    );
    let mut qy = sqlx::query(&sql).bind(req.user_id);
    for id in &req.ids {
        qy = qy.bind(id);
    }
    let bound = qy.execute(pool).await?.rows_affected();

    audit(
        &state,
        user.id,
        "console",
        "migrate_bind",
        &format!("{}: {} 项 → u{}", req.module, bound, req.user_id),
    )
    .await;

    Ok(Json(json!({ "ok": true, "bound": bound })))
}

/// 把 email 唯一约束（idx_users_email_unique / users.email）触发的 UNIQUE 错误
/// 转成友好的「邮箱已被占用」冲突，避免回笼统的 500。兜并发竞态（预检后仍可能撞约束）。
fn map_email_conflict(e: sqlx::Error) -> AppError {
    match e {
        sqlx::Error::Database(ref db)
            if db.message().contains("users.email")
                || db.message().contains("idx_users_email_unique") =>
        {
            AppError::conflict("该邮箱已被其他账号使用")
        }
        other => AppError::Database(other),
    }
}
