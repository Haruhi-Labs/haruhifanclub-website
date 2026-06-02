//! news 模块：京都学报 / 新闻站（文章、活动、奖品、积分）。
//! 忠实移植旧 harunews 后端（server/index.js + db.js + imageUtils.js），统一挂载于 /api/news。
//!
//! 与旧实现的差异（详见交付报告）：
//! - 旧路径在根 /api/* 下，这里统一前缀 /api/news/*（router nest 在 /news）。
//! - 后台鉴权由旧的 X-Admin-Token 头比对 ADMIN_TOKEN，改为统一 JWT + RBAC（authorize/AuthUser，app="news"）。
//! - lowdb 与所有 *.json 备份（db.json/activities.json/...）是死代码，未移植，只移植 sqlite 逻辑。
//! - 图片：旧 imageUtils 把 data URL 转存为 public/uploads/<md5>.<ext> 并入库 `/uploads/<md5>.<ext>`；
//!   新约定用 haruhi_media::save_base64_image(&cfg.uploads_dir,"news",..) → 相对 `news/<md5>.<ext>`，
//!   入库存绝对路径 `/uploads/news/<md5>.<ext>`（非 data URL 的字符串原样透传，对齐旧 saveImage）。

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Map, Value};

use crate::state::AppState;

// ============================================================
// 路由装配
// ============================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // ---- 活动（Activities）----
        .route("/activities", get(list_activities))
        .route("/admin/activities", post(create_activity))
        .route("/admin/activities/reorder", post(reorder_activities))
        .route(
            "/admin/activities/{id}",
            put(update_activity).delete(delete_activity),
        )
        // ---- 奖品（Prizes）----
        .route("/prizes", get(list_prizes))
        .route("/admin/prizes", post(create_prize))
        .route("/admin/prizes/reorder", post(reorder_prizes))
        .route("/admin/prizes/{id}", put(update_prize).delete(delete_prize))
        // ---- 文章（Articles）----
        .route("/articles", get(list_articles).post(create_article))
        .route(
            "/articles/{id}",
            get(get_article).put(update_article).delete(delete_article),
        )
        .route("/admin/articles", get(admin_list_articles))
        // ---- 积分（Points）----
        .route("/admin/points/users", get(points_users))
        .route("/points/search", get(points_search))
        .route("/points/update", post(points_update))
        .route("/points/{id}", get(points_get))
}

// ============================================================
// 工具：图片转存（对齐旧 imageUtils.saveImage）
// ============================================================

/// 把 data URL 图片转存到 uploads/news/，返回入库用绝对路径 `/uploads/news/<md5>.<ext>`；
/// 非 data URL 的字符串原样返回（对齐旧 saveImage 的 “不是 base64 就当作已有路径” 行为）。
async fn save_image(state: &AppState, input: &str) -> String {
    if !input.starts_with("data:image/") {
        return input.to_string();
    }
    match haruhi_media::save_base64_image(&state.cfg.uploads_dir, "news", input).await {
        // 返回值是相对 `news/<md5>.<ext>`，入库前缀 `/uploads/`。
        Ok(rel) => format!("/uploads/{rel}"),
        Err(e) => {
            // 旧实现 parseBase64 失败时原样返回 inputStr，这里同样降级。
            tracing::warn!("news 图片转存失败，原样入库: {e}");
            input.to_string()
        }
    }
}

/// 处理 content（富文本块数组）中内嵌的 image block（block.type=="image" && block.src 为 data URL）。
/// 忠实旧逻辑：仅替换 src，其余字段原样保留；非数组则原样返回。
async fn process_content_images(state: &AppState, content: &Value) -> Value {
    let arr = match content.as_array() {
        Some(a) => a,
        None => return content.clone(),
    };
    let mut out = Vec::with_capacity(arr.len());
    for block in arr {
        let mut b = block.clone();
        let is_image = b.get("type").and_then(|v| v.as_str()) == Some("image");
        let src = b.get("src").and_then(|v| v.as_str()).map(|s| s.to_string());
        if is_image {
            if let Some(src) = src {
                if !src.is_empty() {
                    let new_src = save_image(state, &src).await;
                    if let Some(obj) = b.as_object_mut() {
                        obj.insert("src".to_string(), Value::String(new_src));
                    }
                }
            }
        }
        out.push(b);
    }
    Value::Array(out)
}

// ============================================================
// 工具：JSON 字段解析 / 动态 UPDATE（对齐旧 db.js）
// ============================================================

/// 解析存库 JSON 字符串为 Value；失败回退空数组（对齐旧 try/catch → []）。
fn parse_json_arr(s: Option<&str>) -> Value {
    match s {
        Some(text) => serde_json::from_str::<Value>(text).unwrap_or_else(|_| json!([])),
        None => json!([]),
    }
}

/// 取 i64（兼容数字或数字字符串，对齐旧 Number()）。
fn num_i64(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        Some(n)
    } else if let Some(f) = v.as_f64() {
        Some(f as i64)
    } else {
        v.as_str()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|f| f as i64)
    }
}

/// 把请求体中的字段值绑定到 sqlx query（对象/数组 → JSON 字符串，对齐旧 db.js insert/update）。
fn bind_value<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    v: &Value,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        Value::Null => q.bind(None::<String>),
        Value::Bool(b) => q.bind(if *b { 1_i64 } else { 0_i64 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                q.bind(i)
            } else if let Some(f) = n.as_f64() {
                q.bind(f)
            } else {
                q.bind(n.to_string())
            }
        }
        Value::String(s) => q.bind(s.clone()),
        // 对象/数组：JSON.stringify（对齐 db.js）
        other => q.bind(serde_json::to_string(other).unwrap_or_else(|_| "null".into())),
    }
}

/// 动态部分更新：UPDATE <table> SET k=?,... WHERE id=?（对齐旧 db.update，跳过 id 键）。
/// allowed 为 None 表示不过滤列名（旧实现不过滤，按调用方传入字段为准）。
async fn dynamic_update(
    pool: &sqlx::SqlitePool,
    table: &str,
    id: &str,
    data: &Map<String, Value>,
) -> Result<u64, sqlx::Error> {
    let keys: Vec<&String> = data.keys().filter(|k| k.as_str() != "id").collect();
    if keys.is_empty() {
        return Ok(0);
    }
    let set_clause = keys
        .iter()
        .map(|k| format!("{k} = ?"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("UPDATE {table} SET {set_clause} WHERE id = ?");
    let mut q = sqlx::query(&sql);
    for k in &keys {
        q = bind_value(q, &data[*k]);
    }
    q = q.bind(id);
    let res = q.execute(pool).await?;
    Ok(res.rows_affected())
}

// ============================================================
// 活动（Activities）
// ============================================================

#[derive(sqlx::FromRow)]
struct ActivityRow {
    id: i64,
    title: Option<String>,
    intro: Option<String>,
    detail: Option<String>,
    image: Option<String>,
    #[sqlx(rename = "totalPoints")]
    total_points: Option<i64>,
    #[sqlx(rename = "actionName")]
    action_name: Option<String>,
    #[sqlx(rename = "pointsPerAction")]
    points_per_action: Option<i64>,
    status: Option<String>,
    #[sqlx(rename = "type")]
    type_: Option<String>,
    #[sqlx(rename = "displayOrder")]
    display_order: Option<i64>,
}

fn activity_to_json(a: &ActivityRow) -> Value {
    json!({
        "id": a.id,
        "title": a.title,
        "intro": a.intro,
        "detail": a.detail,
        "image": a.image,
        "totalPoints": a.total_points,
        "actionName": a.action_name,
        "pointsPerAction": a.points_per_action,
        "status": a.status,
        "type": a.type_,
        "displayOrder": a.display_order,
    })
}

// GET /activities（公开）
async fn list_activities(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<ActivityRow> =
        sqlx::query_as("SELECT * FROM activities ORDER BY displayOrder ASC, id ASC")
            .fetch_all(&state.pools.news)
            .await?;
    let data: Vec<Value> = rows.iter().map(activity_to_json).collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// POST /admin/activities（Manage）
async fn create_activity(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;

    // 图片转存
    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let saved = save_image(&state, img).await;
            body["image"] = Value::String(saved);
        }
    }

    let g = |k: &str| body.get(k).cloned().unwrap_or(Value::Null);
    let mut q = sqlx::query_scalar::<_, i64>(
        "INSERT INTO activities (title, intro, detail, image, totalPoints, actionName, pointsPerAction, status, type) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    );
    for k in [
        "title",
        "intro",
        "detail",
        "image",
        "totalPoints",
        "actionName",
        "pointsPerAction",
        "status",
        "type",
    ] {
        q = bind_scalar(q, &g(k));
    }
    let new_id: i64 = q.fetch_one(&state.pools.news).await?;

    // 同步 displayOrder = id（默认新项排末尾）
    sqlx::query("UPDATE activities SET displayOrder = id WHERE id = ?")
        .bind(new_id)
        .execute(&state.pools.news)
        .await?;

    // 响应：回显请求体 + id + displayOrder（对齐旧 newItem）
    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".into(), json!(new_id));
        obj.insert("displayOrder".into(), json!(new_id));
    }
    Ok(Json(json!({ "message": "success", "data": body })))
}

// PUT /admin/activities/:id（Manage）
async fn update_activity(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;

    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let saved = save_image(&state, img).await;
            body["image"] = Value::String(saved);
        }
    }
    if let Some(obj) = body.as_object_mut() {
        obj.remove("id"); // 不更新 ID
    }

    let obj = body.as_object().cloned().unwrap_or_default();
    dynamic_update(&state.pools.news, "activities", &id, &obj).await?;

    // 响应：{ id, ...newItem }
    let mut data = body.clone();
    if let Some(o) = data.as_object_mut() {
        o.insert("id".into(), Value::String(id));
    }
    Ok(Json(json!({ "message": "success", "data": data })))
}

// DELETE /admin/activities/:id（Manage）
async fn delete_activity(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.news)
        .await?;
    Ok(Json(json!({ "message": "success" })))
}

// POST /admin/activities/reorder（Manage）
async fn reorder_activities(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;
    reorder_generic(&state, "activities", &body).await
}

// ============================================================
// 奖品（Prizes）
// ============================================================

#[derive(sqlx::FromRow)]
struct PrizeRow {
    id: i64,
    name: Option<String>,
    description: Option<String>,
    points: Option<i64>,
    stock: Option<i64>,
    category: Option<String>,
    rarity: Option<String>,
    color: Option<String>,
    size: Option<String>,
    image: Option<String>,
    #[sqlx(rename = "displayOrder")]
    display_order: Option<i64>,
}

fn prize_to_json(p: &PrizeRow) -> Value {
    json!({
        "id": p.id,
        "name": p.name,
        "description": p.description,
        "points": p.points,
        "stock": p.stock,
        "category": p.category,
        "rarity": p.rarity,
        "color": p.color,
        "size": p.size,
        "image": p.image,
        "displayOrder": p.display_order,
    })
}

// GET /prizes（公开）
async fn list_prizes(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<PrizeRow> =
        sqlx::query_as("SELECT * FROM prizes ORDER BY displayOrder ASC, id ASC")
            .fetch_all(&state.pools.news)
            .await?;
    let data: Vec<Value> = rows.iter().map(prize_to_json).collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// POST /admin/prizes（Manage）
async fn create_prize(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;

    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let saved = save_image(&state, img).await;
            body["image"] = Value::String(saved);
        }
    }

    let g = |k: &str| body.get(k).cloned().unwrap_or(Value::Null);
    let mut q = sqlx::query_scalar::<_, i64>(
        "INSERT INTO prizes (name, description, points, stock, category, rarity, color, size, image) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    );
    for k in [
        "name",
        "description",
        "points",
        "stock",
        "category",
        "rarity",
        "color",
        "size",
        "image",
    ] {
        q = bind_scalar(q, &g(k));
    }
    let new_id: i64 = q.fetch_one(&state.pools.news).await?;

    sqlx::query("UPDATE prizes SET displayOrder = id WHERE id = ?")
        .bind(new_id)
        .execute(&state.pools.news)
        .await?;

    if let Some(obj) = body.as_object_mut() {
        obj.insert("id".into(), json!(new_id));
        obj.insert("displayOrder".into(), json!(new_id));
    }
    Ok(Json(json!({ "message": "success", "data": body })))
}

// PUT /admin/prizes/:id（Manage）
async fn update_prize(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;

    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let saved = save_image(&state, img).await;
            body["image"] = Value::String(saved);
        }
    }
    if let Some(obj) = body.as_object_mut() {
        obj.remove("id");
    }

    let obj = body.as_object().cloned().unwrap_or_default();
    dynamic_update(&state.pools.news, "prizes", &id, &obj).await?;

    let mut data = body.clone();
    if let Some(o) = data.as_object_mut() {
        o.insert("id".into(), Value::String(id));
    }
    Ok(Json(json!({ "message": "success", "data": data })))
}

// DELETE /admin/prizes/:id（Manage）
async fn delete_prize(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;
    sqlx::query("DELETE FROM prizes WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.news)
        .await?;
    Ok(Json(json!({ "message": "success" })))
}

// POST /admin/prizes/reorder（Manage）
async fn reorder_prizes(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;
    reorder_generic(&state, "prizes", &body).await
}

/// 通用 reorder（activities/prizes 同逻辑，对齐旧实现）：
/// 取出这些 id 当前的 displayOrder，升序排序后按 ids 顺序回填。
async fn reorder_generic(state: &AppState, table: &str, body: &Value) -> AppResult<Response> {
    let ids = body.get("ids").and_then(|v| v.as_array());
    let ids = match ids {
        Some(a) => a,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid data" })),
            )
                .into_response())
        }
    };
    // ids 元素转 i64（旧实现是 JS Number，这里统一为整数）
    let id_list: Vec<i64> = ids.iter().filter_map(num_i64).collect();

    let mut tx = state.pools.news.begin().await?;

    // 1. 取当前 displayOrder
    let placeholders = std::iter::repeat("?")
        .take(id_list.len())
        .collect::<Vec<_>>()
        .join(",");
    let mut order_vals: Vec<i64> = if id_list.is_empty() {
        Vec::new()
    } else {
        let sql = format!("SELECT id, displayOrder FROM {table} WHERE id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, (i64, Option<i64>)>(&sql);
        for id in &id_list {
            q = q.bind(id);
        }
        let rows: Vec<(i64, Option<i64>)> = q.fetch_all(&mut *tx).await?;
        rows.into_iter().map(|(_, ord)| ord.unwrap_or(0)).collect()
    };
    // 2. 升序
    order_vals.sort();

    // 3. 回填到对应 id
    for (i, id) in id_list.iter().enumerate() {
        if i < order_vals.len() {
            sqlx::query(&format!("UPDATE {table} SET displayOrder = ? WHERE id = ?"))
                .bind(order_vals[i])
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(Json(json!({ "message": "success" })).into_response())
}

// ============================================================
// 文章（Articles）
// ============================================================

// 文章行：列多且含 JSON 字段，用 FromRow 结构体。
#[derive(sqlx::FromRow)]
struct ArticleRow {
    id: i64,
    title: Option<String>,
    subtitle: Option<String>,
    date: Option<String>,
    #[sqlx(rename = "type")]
    type_: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    image: Option<String>,
    #[sqlx(rename = "originalImage")]
    original_image: Option<String>,
    #[sqlx(rename = "coverFocalX")]
    cover_focal_x: Option<f64>,
    #[sqlx(rename = "coverFocalY")]
    cover_focal_y: Option<f64>,
    content: Option<String>,
    #[sqlx(rename = "isPinned")]
    is_pinned: Option<i64>,
    #[sqlx(rename = "pinOrder")]
    pin_order: Option<i64>,
    participants: Option<String>,
    status: Option<String>,
    created_at: Option<String>,
    summary: Option<String>,
}

/// parseArticleRows：解析 tags/participants/content JSON，isPinned → bool，含 content（完整对象）。
fn parse_article_full(r: &ArticleRow) -> Value {
    json!({
        "id": r.id,
        "title": r.title,
        "subtitle": r.subtitle,
        "date": r.date,
        "type": r.type_,
        "author": r.author,
        "tags": parse_json_arr(r.tags.as_deref()),
        "image": r.image,
        "originalImage": r.original_image,
        "coverFocalX": r.cover_focal_x,
        "coverFocalY": r.cover_focal_y,
        "content": parse_json_arr(r.content.as_deref()),
        "isPinned": r.is_pinned.unwrap_or(0) != 0,
        "pinOrder": r.pin_order,
        "participants": parse_json_arr(r.participants.as_deref()),
        "status": r.status,
        "created_at": r.created_at,
        "summary": r.summary,
    })
}

/// 列表项预览：对齐 buildArticleListItem（不含 content，含 preview，participants 空数组 → null）。
fn build_article_list_item(r: &ArticleRow) -> Value {
    let tags = parse_json_arr(r.tags.as_deref());
    let participants_val = parse_json_arr(r.participants.as_deref());
    let content_val = parse_json_arr(r.content.as_deref());

    // preview：优先 summary，否则拼接 paragraph 文本到 >240 字符
    let summary_trimmed = r.summary.as_deref().map(|s| s.trim()).unwrap_or("");
    let preview = if !summary_trimmed.is_empty() {
        summary_trimmed.to_string()
    } else if let Some(blocks) = content_val.as_array() {
        let mut texts: Vec<String> = Vec::new();
        for block in blocks {
            let is_para = block.get("type").and_then(|v| v.as_str()) == Some("paragraph");
            let text = block.get("text").and_then(|v| v.as_str());
            if is_para {
                if let Some(t) = text {
                    if !t.is_empty() {
                        texts.push(t.to_string());
                        if texts.join("\n").chars().count() > 240 {
                            break;
                        }
                    }
                }
            }
        }
        texts.join("\n")
    } else {
        String::new()
    };

    // participants 长度为 0 → null
    let participants = match participants_val.as_array() {
        Some(a) if a.is_empty() => Value::Null,
        _ => participants_val,
    };

    json!({
        "id": r.id,
        "title": r.title,
        "subtitle": r.subtitle,
        "date": r.date,
        "type": r.type_,
        "author": r.author,
        "tags": tags,
        "image": r.image,
        "originalImage": r.original_image,
        "coverFocalX": r.cover_focal_x,
        "coverFocalY": r.cover_focal_y,
        "isPinned": r.is_pinned.unwrap_or(0) != 0,
        "pinOrder": r.pin_order,
        "participants": participants,
        "status": r.status,
        "preview": preview,
        "summary": r.summary,
    })
}

// 注：created_at 不在旧 list SQL 中，但 ArticleRow(FromRow) 需要该列；这里补选，
// 不影响 build_article_list_item 输出（其不返回 created_at）。
const ARTICLE_LIST_COLS: &str = "SELECT id, title, subtitle, date, type, author, tags, image, \
    originalImage, coverFocalX, coverFocalY, content, isPinned, pinOrder, participants, status, \
    created_at, summary \
    FROM articles WHERE status IS NULL OR status = 'published'";

// GET /articles（公开，已发布列表）
async fn list_articles(State(state): State<AppState>) -> AppResult<Json<Value>> {
    // 注意：列表 SELECT 仍取 content（buildArticleListItem 需要它生成 preview），
    // 但响应里不返回 content（对齐旧 buildArticleListItem 输出）。旧 SQL 未取 content
    // 而依赖 summary/preview 退化；这里取 content 以忠实复现 preview 生成逻辑。
    let mut rows: Vec<ArticleRow> = sqlx::query_as(ARTICLE_LIST_COLS)
        .fetch_all(&state.pools.news)
        .await?;

    // 排序：置顶优先（pinOrder 升序），其余按 id 降序
    rows.sort_by(|a, b| {
        let ap = a.is_pinned.unwrap_or(0) != 0;
        let bp = b.is_pinned.unwrap_or(0) != 0;
        match (ap, bp) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => a
                .pin_order
                .unwrap_or(0)
                .cmp(&b.pin_order.unwrap_or(0)),
            (false, false) => b.id.cmp(&a.id),
        }
    });

    let data: Vec<Value> = rows.iter().map(build_article_list_item).collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// GET /articles/:id（公开；非 published 需后台权限）
async fn get_article(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: Option<AuthUser>,
) -> AppResult<Response> {
    let row: Option<ArticleRow> = sqlx::query_as("SELECT * FROM articles WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pools.news)
        .await?;
    let row = match row {
        Some(r) => r,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Article not found" })),
            )
                .into_response())
        }
    };

    // status 存在且 != 'published' → 需后台读权限（旧实现比对 ADMIN_TOKEN，这里改 RBAC）
    let status = row.status.as_deref().unwrap_or("");
    if !status.is_empty() && status != "published" {
        let allowed = match &user {
            Some(u) => authorize(&state.pools.core, u, "news", Action::Read)
                .await
                .is_ok(),
            None => false,
        };
        if !allowed {
            return Ok((
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Unauthorized" })),
            )
                .into_response());
        }
    }

    let mut article = parse_article_full(&row);
    if let Some(o) = article.as_object_mut() {
        o.insert("isContentPreview".into(), json!(false));
    }
    Ok(Json(json!({ "message": "success", "data": article })).into_response())
}

// GET /admin/articles（Read）
async fn admin_list_articles(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Read).await?;
    let rows: Vec<ArticleRow> = sqlx::query_as("SELECT * FROM articles ORDER BY id DESC")
        .fetch_all(&state.pools.news)
        .await?;
    let data: Vec<Value> = rows.iter().map(parse_article_full).collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// POST /articles（公开投稿 → pending；后台 → published。对齐旧 token 分支，改 RBAC：登录且有 Write 权限则 published）
async fn create_article(
    State(state): State<AppState>,
    user: Option<AuthUser>,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    // 是否后台：登录用户且具备 news Write 权限 → published；否则游客投稿 → pending
    let is_admin = match &user {
        Some(u) => authorize(&state.pools.core, u, "news", Action::Write)
            .await
            .is_ok(),
        None => false,
    };
    let status = if is_admin { "published" } else { "pending" };
    let created_at = chrono::Utc::now().to_rfc3339();

    // isPinned → 0/1；pinOrder → Number || 0
    let is_pinned = body
        .get("isPinned")
        .map(|v| match v {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            _ => false,
        })
        .unwrap_or(false);
    let pin_order = body.get("pinOrder").and_then(num_i64).unwrap_or(0);

    // 图片转存：image / originalImage / content 内嵌图
    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let s = save_image(&state, img).await;
            body["image"] = Value::String(s);
        }
    }
    if let Some(img) = body.get("originalImage").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let s = save_image(&state, img).await;
            body["originalImage"] = Value::String(s);
        }
    }
    if let Some(content) = body.get("content").cloned() {
        if content.is_array() {
            let processed = process_content_images(&state, &content).await;
            body["content"] = processed;
        }
    }

    let g = |k: &str| body.get(k).cloned().unwrap_or(Value::Null);
    // tags/content/participants：对象/数组入库为 JSON 字符串（bind_scalar 处理）
    let mut q = sqlx::query_scalar::<_, i64>(
        "INSERT INTO articles \
         (title, subtitle, date, type, author, tags, image, originalImage, coverFocalX, coverFocalY, \
          content, isPinned, pinOrder, participants, status, created_at, summary) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    );
    q = bind_scalar(q, &g("title"));
    q = bind_scalar(q, &g("subtitle"));
    q = bind_scalar(q, &g("date"));
    q = bind_scalar(q, &g("type"));
    q = bind_scalar(q, &g("author"));
    q = bind_scalar(q, &g("tags"));
    q = bind_scalar(q, &g("image"));
    q = bind_scalar(q, &g("originalImage"));
    q = bind_scalar(q, &g("coverFocalX"));
    q = bind_scalar(q, &g("coverFocalY"));
    q = bind_scalar(q, &g("content"));
    q = q.bind(if is_pinned { 1_i64 } else { 0_i64 });
    q = q.bind(pin_order);
    q = bind_scalar(q, &g("participants"));
    q = q.bind(status);
    q = q.bind(&created_at);
    q = bind_scalar(q, &g("summary"));
    let new_id: i64 = q.fetch_one(&state.pools.news).await?;

    // 响应：回显 newArticle（含已处理 image/content、规范化 isPinned/pinOrder/status/created_at）+ id
    if let Some(obj) = body.as_object_mut() {
        obj.insert("status".into(), json!(status));
        obj.insert("created_at".into(), json!(created_at));
        obj.insert("isPinned".into(), json!(if is_pinned { 1 } else { 0 }));
        obj.insert("pinOrder".into(), json!(pin_order));
        obj.insert("id".into(), json!(new_id));
    }
    Ok(Json(json!({ "message": "success", "data": body, "status": status })))
}

// PUT /articles/:id（Write）
async fn update_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Write).await?;

    if let Some(img) = body.get("image").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let s = save_image(&state, img).await;
            body["image"] = Value::String(s);
        }
    }
    if let Some(img) = body.get("originalImage").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            let s = save_image(&state, img).await;
            body["originalImage"] = Value::String(s);
        }
    }
    if let Some(content) = body.get("content").cloned() {
        if content.is_array() {
            let processed = process_content_images(&state, &content).await;
            body["content"] = processed;
        }
    }
    // isPinned 存在 → 0/1
    if let Some(v) = body.get("isPinned").cloned() {
        let b = match v {
            Value::Bool(b) => b,
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            _ => false,
        };
        body["isPinned"] = json!(if b { 1 } else { 0 });
    }

    if let Some(obj) = body.as_object_mut() {
        obj.remove("id");
        obj.remove("created_at");
    }

    let obj = body.as_object().cloned().unwrap_or_default();
    dynamic_update(&state.pools.news, "articles", &id, &obj).await?;

    let mut data = body.clone();
    if let Some(o) = data.as_object_mut() {
        o.insert("id".into(), Value::String(id));
    }
    Ok(Json(json!({ "message": "success", "data": data })))
}

// DELETE /articles/:id（Manage）
async fn delete_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;
    sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.news)
        .await?;
    Ok(Json(json!({ "message": "success" })))
}

// ============================================================
// 积分（Points）
// ============================================================

// GET /admin/points/users（Read）
async fn points_users(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "news", Action::Read).await?;
    let rows: Vec<(String, Option<i64>)> =
        sqlx::query_as("SELECT id, total FROM users ORDER BY total DESC")
            .fetch_all(&state.pools.news)
            .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(id, total)| json!({ "id": id, "total": total }))
        .collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// GET /points/search（公开）
async fn points_search(
    State(state): State<AppState>,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let query = q.get("q").map(|s| s.as_str()).unwrap_or("");
    if query.trim().is_empty() {
        return Ok(Json(json!({ "message": "success", "data": [] })));
    }
    let like = format!("%{query}%");
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM users WHERE id LIKE ? LIMIT 10")
        .bind(&like)
        .fetch_all(&state.pools.news)
        .await?;
    let data: Vec<Value> = rows.into_iter().map(|(id,)| json!(id)).collect();
    Ok(Json(json!({ "message": "success", "data": data })))
}

// GET /points/:id（公开）
async fn points_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let user_row: Option<(String, Option<i64>)> =
        sqlx::query_as("SELECT id, total FROM users WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.pools.news)
            .await?;
    let history = fetch_points_history(&state, &id).await?;

    let (uid, total) = match user_row {
        Some((uid, total)) => (uid, total.unwrap_or(0)),
        None => (id.clone(), 0),
    };
    Ok(Json(json!({
        "message": "success",
        "data": { "id": uid, "total": total, "history": history }
    })))
}

// POST /points/update（Manage）
async fn points_update(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "news", Action::Manage).await?;

    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
    if id.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing User ID" })),
        )
            .into_response());
    }
    // parseInt(change, 10)：取整数；非法 → 400
    let change_num = match body.get("change") {
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Some(Value::String(s)) => parse_int_radix10(s),
        _ => None,
    };
    let change_num = match change_num {
        Some(n) => n,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid change amount" })),
            )
                .into_response())
        }
    };
    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("系统调整")
        .to_string();

    // 取或建用户
    let existing: Option<(String, Option<i64>)> =
        sqlx::query_as("SELECT id, total FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pools.news)
            .await?;
    let old_total = match existing {
        Some((_, t)) => t.unwrap_or(0),
        None => {
            sqlx::query("INSERT INTO users (id, total) VALUES (?, ?)")
                .bind(id)
                .bind(0_i64)
                .execute(&state.pools.news)
                .await?;
            0
        }
    };

    let new_total = old_total + change_num;
    sqlx::query("UPDATE users SET total = ? WHERE id = ?")
        .bind(new_total)
        .bind(id)
        .execute(&state.pools.news)
        .await?;

    // 历史记录
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let change_str = if change_num > 0 {
        format!("+{change_num}")
    } else {
        format!("{change_num}")
    };
    let timestamp = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO points_history (user_id, date, change, reason, timestamp) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&date)
    .bind(&change_str)
    .bind(&reason)
    .bind(timestamp)
    .execute(&state.pools.news)
    .await?;

    let history = fetch_points_history(&state, id).await?;
    Ok(Json(json!({
        "message": "success",
        "data": { "id": id, "total": new_total, "history": history }
    }))
    .into_response())
}

/// 取用户积分历史（最近 50 条，timestamp 降序）。
async fn fetch_points_history(state: &AppState, user_id: &str) -> AppResult<Vec<Value>> {
    let rows: Vec<(Option<String>, Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT date, change, reason, timestamp FROM points_history \
         WHERE user_id = ? ORDER BY timestamp DESC LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&state.pools.news)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(date, change, reason, timestamp)| {
            json!({ "date": date, "change": change, "reason": reason, "timestamp": timestamp })
        })
        .collect())
}

// ============================================================
// 杂项辅助
// ============================================================

/// 把 JSON 值绑定到 query_scalar（INSERT ... RETURNING id）。
/// 对象/数组 → JSON 字符串（对齐旧 db.js insert），bool → 0/1，数字保持类型。
fn bind_scalar<'q>(
    q: sqlx::query::QueryScalar<'q, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'q>>,
    v: &Value,
) -> sqlx::query::QueryScalar<'q, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        Value::Null => q.bind(None::<String>),
        Value::Bool(b) => q.bind(if *b { 1_i64 } else { 0_i64 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                q.bind(i)
            } else if let Some(f) = n.as_f64() {
                q.bind(f)
            } else {
                q.bind(n.to_string())
            }
        }
        Value::String(s) => q.bind(s.clone()),
        other => q.bind(serde_json::to_string(other).unwrap_or_else(|_| "null".into())),
    }
}

/// 模拟 JS parseInt(s, 10)：跳过前导空白，可选符号，取尽可能长的十进制前缀。
fn parse_int_radix10(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut i = 0;
    let mut sign = 1_i64;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None; // 无数字 → NaN
    }
    t[start..i].parse::<i64>().ok().map(|n| sign * n)
}
