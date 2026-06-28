//! art 模块：绘画部画廊（作品上传/审核、评论、点赞、积分、创作者管理）。
//! 忠实移植旧 haruhi-art-club 后端（server/index.js + db.js + ai.js），统一挂载于 /api/art。
//!
//! 与旧实现的差异：
//! - 旧路径在根 /api/* 下，这里统一前缀 /api/art/*（router nest 在 /art）。
//! - 后台管理由旧的 x-admin-password 头改为 JWT + RBAC（authorize/AuthUser）。
//! - 匿名身份签名 Cookie（haruhi_anon）行为忠实保留：HMAC-SHA256 签名校验/下发。

use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;

use crate::state::AppState;

// ============================================================
// 路由装配
// ============================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // ---- 公开接口 ----
        .route("/points/leaderboard", get(points_leaderboard))
        .route("/points/history", get(points_history))
        .route("/creators/search", get(creators_search))
        .route("/creators/verify", get(creators_verify))
        .route("/artworks", get(list_artworks).post(create_artwork))
        .route("/artworks/{id}", get(get_artwork))
        .route("/thumb", get(get_thumb))
        .route("/comments", get(list_comments).post(create_comment))
        .route("/likes/artwork/{id}", post(like_artwork))
        .route("/likes/comment/{id}", post(like_comment))
        .route("/claim", post(claim_anon))
        // ---- 个人中心（需登录，按 author_user_id 归属本人）----
        .route("/me/artworks", get(my_artworks))
        .route("/me/comments", get(my_comments))
        .route("/me/points", get(my_points))
        .route(
            "/me/artworks/{id}",
            axum::routing::patch(update_my_artwork).delete(delete_my_artwork),
        )
        .route(
            "/me/comments/{id}",
            axum::routing::delete(delete_my_comment),
        )
        // ---- 后台接口（RBAC）----
        .route("/admin/pending-artworks", get(admin_pending_artworks))
        .route("/admin/audit-history", get(admin_audit_history))
        .route("/admin/artworks/{id}/approve", post(admin_approve_artwork))
        .route("/admin/artworks/{id}/reject", post(admin_reject_artwork))
        .route("/admin/artworks/{id}/status", post(admin_artwork_status))
        .route("/admin/artworks/{id}/update", post(admin_artwork_update))
        .route(
            "/admin/artworks/{id}",
            axum::routing::delete(admin_delete_artwork),
        )
        .route("/admin/comments", get(admin_list_comments))
        .route("/admin/comments/{id}/status", post(admin_comment_status))
        .route(
            "/admin/comments/{id}",
            axum::routing::delete(admin_delete_comment),
        )
        .route(
            "/admin/creators",
            get(admin_list_creators).post(admin_create_creator),
        )
        .route("/admin/creators/{uid}/update", post(admin_update_creator))
        .route(
            "/admin/creators/{uid}",
            axum::routing::delete(admin_delete_creator),
        )
        .route("/admin/points-ledger", get(admin_points_ledger))
        .route("/admin/points/grant", post(admin_points_grant))
}

// ============================================================
// 匿名身份 Cookie（haruhi_anon）：HMAC-SHA256 签名
// ============================================================

const COOKIE_NAME: &str = "haruhi_anon";
const COOKIE_SIG: &str = "haruhi_anon_sig";

type HmacSha256 = Hmac<Sha256>;

/// base64url（无填充，- / _），对齐旧 b64url。
fn b64url(buf: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

fn sign(secret: &str, val: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC 接受任意长度密钥");
    mac.update(val.as_bytes());
    b64url(&mac.finalize().into_bytes())
}

/// 极简 Cookie 头解析（对齐旧 parseCookies，URL 解码 value）。
fn parse_cookies(header: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let s = header.unwrap_or("");
    if s.is_empty() {
        return out;
    }
    for p in s.split(';') {
        if let Some(i) = p.find('=') {
            let k = p[..i].trim();
            let v = p[i + 1..].trim();
            if k.is_empty() {
                continue;
            }
            out.insert(k.to_string(), url_decode(v));
        }
    }
    out
}

/// 最小 percent-decode（覆盖 uuid 不会被编码，但与旧 decodeURIComponent 行为兼容）。
fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]));
            if let (Some(a), Some(b)) = h {
                out.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// 解析请求 Cookie，校验签名；无/非法则生成新 anon_id。
/// 返回 (anon_id, set_cookie_headers)：当需要下发新 Cookie 时返回两条 Set-Cookie。
fn resolve_anon(secret: &str, headers: &HeaderMap) -> (String, Option<[String; 2]>) {
    let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok());
    let cookies = parse_cookies(cookie_header);
    let id = cookies.get(COOKIE_NAME);
    let sig = cookies.get(COOKIE_SIG);
    let ok = match (id, sig) {
        (Some(id), Some(sig)) => !id.is_empty() && sign(secret, id) == *sig,
        _ => false,
    };
    if ok {
        (id.unwrap().clone(), None)
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        let sig = sign(secret, &new_id);
        let common = "Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000";
        let set = [
            format!("{COOKIE_NAME}={new_id}; {common}"),
            format!("{COOKIE_SIG}={sig}; {common}"),
        ];
        (new_id, Some(set))
    }
}

/// 只解析、校验旧匿名 Cookie 的 anon_id（合法才返回 Some，绝不新发）。供登录用户「认领」历史匿名内容。
fn parse_anon_cookie(secret: &str, headers: &HeaderMap) -> Option<String> {
    let cookies = parse_cookies(headers.get("cookie").and_then(|v| v.to_str().ok()));
    let id = cookies.get(COOKIE_NAME)?;
    let sig = cookies.get(COOKIE_SIG)?;
    if !id.is_empty() && sign(secret, id) == *sig {
        Some(id.clone())
    } else {
        None
    }
}

/// 把 Set-Cookie 头附加到一个 JSON 响应上（用于公开接口需要下发匿名 Cookie 的场景）。
fn json_with_cookie(value: Value, set: Option<[String; 2]>) -> Response {
    let mut resp = Json(value).into_response();
    if let Some(cookies) = set {
        for c in cookies {
            if let Ok(hv) = axum::http::HeaderValue::from_str(&c) {
                resp.headers_mut()
                    .append(axum::http::header::SET_COOKIE, hv);
            }
        }
    }
    resp
}

// ============================================================
// 小工具（对齐旧 index.js 的各 helper）
// ============================================================

use haruhi_core::parse::{clamp_int, clamp_len, safe_text};

/// 解析存库 JSON 数组为 Value 数组，失败返回空数组（对齐 safeJsonArr）。
fn safe_json_arr(s: Option<&str>) -> Vec<Value> {
    match serde_json::from_str::<Value>(s.unwrap_or("[]")) {
        Ok(Value::Array(a)) => a,
        _ => vec![],
    }
}

fn make_tags_norm(tags: &[String]) -> String {
    let norm = tags
        .iter()
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    if norm.is_empty() {
        String::new()
    } else {
        format!(" {norm} ")
    }
}

/// 把逗号/空格/中文逗号分隔的标签串归一为去重数组（对齐 normalizeTagsToArray）。
fn normalize_tags_to_array(s: Option<&str>) -> Vec<String> {
    let raw = s.unwrap_or("").trim();
    if raw.is_empty() {
        return vec![];
    }
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for t0 in raw.split(|c: char| c.is_whitespace() || c == ',' || c == '，') {
        let t0 = t0.trim();
        if t0.is_empty() {
            continue;
        }
        let t = t0.strip_prefix('#').map(|x| x.trim()).unwrap_or(t0);
        if t.is_empty() {
            continue;
        }
        let k = t.to_lowercase();
        if seen.contains(&k) {
            continue;
        }
        seen.insert(k);
        out.push(t.to_string());
    }
    out
}

/// 解析 licenses（JSON 字符串数组），对齐 parseLicenses。
fn parse_licenses(raw: Option<&str>) -> Vec<String> {
    let s = raw.unwrap_or("").trim();
    if s.is_empty() {
        return vec![];
    }
    match serde_json::from_str::<Value>(s) {
        Ok(Value::Array(a)) => a
            .into_iter()
            .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => vec![],
    }
}

// ---- artworks 行结构（SELECT a.*, c.avatar_url AS uploader_avatar）----
// 字段名须与 SELECT_ART 列别名一致（21 列超出 sqlx 元组 16 上限，故用 FromRow 结构体）。
#[derive(sqlx::FromRow)]
struct ArtRow {
    id: i64,
    title: Option<String>,
    description: Option<String>,
    uploader_name: Option<String>,
    uploader_uid: Option<String>,
    source_type: Option<String>,
    content_type: Option<String>,
    tags_json: Option<String>,
    #[allow(dead_code)]
    tags_norm: Option<String>,
    origin_url: Option<String>,
    file_path: Option<String>,
    file_path_original: Option<String>,
    status: Option<String>,
    review_note: Option<String>,
    reviewed_at: Option<String>,
    created_at: Option<String>,
    licenses_json: Option<String>,
    like_total: i64,
    images_json: Option<String>,
    ai_reason: Option<String>,
    uploader_avatar: Option<String>,
}

const SELECT_ART: &str = "SELECT a.id, a.title, a.description, a.uploader_name, a.uploader_uid, \
    a.source_type, a.content_type, a.tags_json, a.tags_norm, a.origin_url, a.file_path, \
    a.file_path_original, a.status, a.review_note, a.reviewed_at, a.created_at, a.licenses_json, \
    COALESCE(CAST(NULLIF(TRIM(a.like_total), '') AS INTEGER), 0) AS like_total, a.images_json, a.ai_reason, c.avatar_url AS uploader_avatar \
    FROM artworks a LEFT JOIN creators c ON a.uploader_uid = c.uid";

/// 映射作品行为响应 JSON（忠实对齐 mapArtworkRow，含单图/多图逻辑）。
fn map_artwork_row(r: &ArtRow) -> Value {
    let raw_images = safe_json_arr(r.images_json.as_deref());
    let images: Vec<Value> = if raw_images.is_empty() && r.file_path.is_some() {
        let fp = r.file_path.as_deref().unwrap();
        let orig = r
            .file_path_original
            .as_deref()
            .map(|o| format!("uploads/{o}"))
            .unwrap_or_else(|| format!("uploads/{fp}"));
        vec![json!({
            "image_url": format!("uploads/{fp}"),
            "original_url": orig,
        })]
    } else {
        raw_images
            .iter()
            .map(|img| {
                let path = img.get("path").and_then(|v| v.as_str());
                let original = img.get("original").and_then(|v| v.as_str());
                let image_url = path.map(|p| format!("uploads/{p}")).unwrap_or_default();
                let original_url = match original {
                    Some(o) => format!("uploads/{o}"),
                    None => path.map(|p| format!("uploads/{p}")).unwrap_or_default(),
                };
                json!({ "image_url": image_url, "original_url": original_url })
            })
            .collect()
    };

    let first_image_url = images
        .first()
        .and_then(|v| v.get("image_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let first_original_url = images
        .first()
        .and_then(|v| v.get("original_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let image_url = match r.file_path.as_deref() {
        Some(fp) => format!("uploads/{fp}"),
        None => first_image_url,
    };
    let original_url = match r.file_path_original.as_deref() {
        Some(fp) => format!("uploads/{fp}"),
        None => first_original_url,
    };

    json!({
        "id": r.id,
        "title": r.title,
        "description": r.description,
        "uploader_name": r.uploader_name,
        "uploader_uid": r.uploader_uid,
        "uploader_avatar": r.uploader_avatar.clone().unwrap_or_default(),
        "source_type": r.source_type,
        "content_type": r.content_type,
        "tags": safe_json_arr(r.tags_json.as_deref()),
        "licenses": safe_json_arr(r.licenses_json.as_deref()),
        "origin_url": r.origin_url,
        "created_at": r.created_at,
        "reviewed_at": r.reviewed_at,
        "review_note": r.review_note.clone().unwrap_or_default(),
        "status": r.status,
        "like_total": r.like_total,
        "image_url": image_url,
        "original_url": original_url,
        "images": images,
        "ai_reason": r.ai_reason.clone().unwrap_or_default(),
    })
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// 把形如 "u{id}" 的 uid 批量解析为账号显示名（昵称优先、为空回退 username）。
/// 非 "u{id}" 或查不到的 uid 不进结果（调用方回退显示原 uid，即历史匿名创作者）。
async fn member_display_names(
    core: &sqlx::SqlitePool,
    uids: &[String],
) -> std::collections::HashMap<String, String> {
    let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let ids: Vec<i64> = uids
        .iter()
        .filter_map(|u| u.strip_prefix('u').and_then(|s| s.parse::<i64>().ok()))
        .collect();
    if ids.is_empty() {
        return map;
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, nickname, username FROM users WHERE id IN ({placeholders}) AND deleted_at IS NULL"
    );
    let mut q = sqlx::query_as::<_, (i64, Option<String>, String)>(&sql);
    for id in &ids {
        q = q.bind(id);
    }
    if let Ok(rows) = q.fetch_all(core).await {
        for (id, nickname, username) in rows {
            let name = nickname
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(username);
            map.insert(format!("u{id}"), name);
        }
    }
    map
}

// ============================================================
// 公开接口
// ============================================================

// 1. 积分排行榜
async fn points_leaderboard(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let page = clamp_int(q.get("page").map(|s| s.as_str()), 1, 100, 1);
    let page_size: i64 = 10;
    let offset = (page - 1) * page_size;

    let rows: Vec<(Option<String>, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT pl.uid, CAST(SUM(CAST(NULLIF(TRIM(pl.points), '') AS INTEGER)) AS INTEGER) as total, c.avatar_url \
         FROM points_ledger pl LEFT JOIN creators c ON c.uid = pl.uid \
         GROUP BY pl.uid ORDER BY total DESC LIMIT ? OFFSET ?",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pools.art)
    .await?;

    let uids: Vec<String> = rows.iter().filter_map(|(u, _, _)| u.clone()).collect();
    let names = member_display_names(&state.pools.core, &uids).await;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(uid, total, avatar_url)| {
            let name = uid.as_ref().and_then(|u| names.get(u).cloned());
            json!({ "uid": uid, "name": name, "total": total, "avatar_url": avatar_url })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

// 2. 积分查询
async fn points_history(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let uid = q.get("uid").map(|s| s.trim()).unwrap_or("").to_string();
    if uid.is_empty() {
        return Ok(Json(json!({ "ok": false, "message": "Missing uid" })));
    }

    let total: Option<i64> = sqlx::query_scalar(
        "SELECT CAST(SUM(CAST(NULLIF(TRIM(points), '') AS INTEGER)) AS INTEGER) as total FROM points_ledger WHERE uid=?",
    )
    .bind(&uid)
    .fetch_one(&state.pools.art)
    .await?;
    let total = total.unwrap_or(0);

    let history_rows: Vec<(Option<i64>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT CAST(NULLIF(TRIM(pl.points), '') AS INTEGER) AS points, pl.note, pl.created_at, a.title as artwork_title \
             FROM points_ledger pl LEFT JOIN artworks a ON a.id = pl.artwork_id \
             WHERE pl.uid=? ORDER BY datetime(pl.created_at) DESC LIMIT 50",
        )
        .bind(&uid)
        .fetch_all(&state.pools.art)
        .await?;
    let history: Vec<Value> = history_rows
        .into_iter()
        .map(|(points, note, created_at, artwork_title)| {
            json!({ "points": points, "note": note, "created_at": created_at, "artwork_title": artwork_title })
        })
        .collect();

    let creator_row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT uid, avatar_url FROM creators WHERE uid=?")
            .bind(&uid)
            .fetch_optional(&state.pools.art)
            .await?;
    // 即使 creators 表无此行，若 uid 形如 u{id} 也解析账号昵称展示
    let names = member_display_names(&state.pools.core, &[uid.clone()]).await;
    let name = names.get(&uid).cloned();
    let avatar_url = creator_row.and_then(|(_, a)| a);
    let creator = if name.is_some() || avatar_url.is_some() {
        Some(json!({ "uid": uid.clone(), "name": name, "avatar_url": avatar_url }))
    } else {
        None
    };

    Ok(Json(json!({
        "ok": true, "total": total, "history": history, "creator": creator
    })))
}

// 3. 创作者模糊搜索
async fn creators_search(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let term = q.get("q").map(|s| s.trim()).unwrap_or("").to_string();
    if term.is_empty() {
        return Ok(Json(json!({ "ok": true, "data": [] })));
    }
    let like = format!("%{term}%");
    // 账号昵称/用户名匹配 → u{id}（让用户能按昵称搜，而非只按 uid）
    let account_rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM users WHERE (nickname LIKE ? OR username LIKE ?) AND deleted_at IS NULL LIMIT 8",
    )
    .bind(&like)
    .bind(&like)
    .fetch_all(&state.pools.core)
    .await
    .unwrap_or_default();
    // creators 表 uid 匹配（含历史匿名 uid）
    let creator_rows: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT uid, avatar_url FROM creators WHERE uid LIKE ? LIMIT 8")
            .bind(&like)
            .fetch_all(&state.pools.art)
            .await?;
    let avatar_map: std::collections::HashMap<String, Option<String>> =
        creator_rows.iter().cloned().collect();
    // 合并去重：账号命中的 u{id} 在前，creators 命中的 uid 在后
    let mut uids: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (id,) in &account_rows {
        let u = format!("u{id}");
        if seen.insert(u.clone()) {
            uids.push(u);
        }
    }
    for (uid, _) in &creator_rows {
        if seen.insert(uid.clone()) {
            uids.push(uid.clone());
        }
    }
    let names = member_display_names(&state.pools.core, &uids).await;
    let data: Vec<Value> = uids
        .into_iter()
        .take(8)
        .map(|uid| {
            json!({
                "uid": uid.clone(),
                "name": names.get(&uid).cloned(),
                "avatar_url": avatar_map.get(&uid).cloned().flatten()
            })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

// 4. 创作者校验
async fn creators_verify(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let uid = q.get("uid").map(|s| s.trim()).unwrap_or("").to_string();
    if uid.is_empty() {
        return Ok(Json(json!({ "ok": true, "exists": false })));
    }
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT uid, avatar_url FROM creators WHERE uid=?")
            .bind(&uid)
            .fetch_optional(&state.pools.art)
            .await?;
    let names = member_display_names(&state.pools.core, &[uid.clone()]).await;
    let name = names.get(&uid).cloned();
    let creator = row.as_ref().map(
        |(uid, avatar_url)| json!({ "uid": uid, "name": name.clone(), "avatar_url": avatar_url }),
    );
    Ok(Json(
        json!({ "ok": true, "exists": row.is_some(), "creator": creator }),
    ))
}

// 5. 作品列表（过滤/搜索/排序/分页）
async fn list_artworks(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let getq = |k: &str| q.get(k).map(|s| s.as_str());

    let status = getq("status").unwrap_or("approved").to_string();
    let content_type = getq("content_type").unwrap_or("all").to_string();
    let source_type = getq("source_type").unwrap_or("all").to_string();
    let uploader_uid = getq("uploader_uid").unwrap_or("").trim().to_string();
    let sort = getq("sort").unwrap_or("time").to_string();
    let q_raw = getq("q").unwrap_or("").trim().to_string();
    let search_field = getq("searchField").unwrap_or("all").to_string();
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 6, 60, 24);

    // 构造 WHERE（用字符串参数，sqlx 运行时绑定）
    let mut where_sql = String::from("WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if status != "all" {
        where_sql.push_str(" AND a.status=?");
        params.push(status.clone());
    }
    if content_type != "all" {
        where_sql.push_str(" AND a.content_type=?");
        params.push(content_type.clone());
    }
    if source_type != "all" && source_type != "balanced" {
        where_sql.push_str(" AND a.source_type=?");
        params.push(source_type.clone());
    }
    if !uploader_uid.is_empty() {
        where_sql.push_str(" AND a.uploader_uid=?");
        params.push(uploader_uid.clone());
    }
    if !q_raw.is_empty() {
        let q_lower = q_raw.to_lowercase();
        let like = format!("%{}%", q_lower.replace(['%', '_'], ""));
        match search_field.as_str() {
            "title" => {
                where_sql.push_str(" AND a.title LIKE ?");
                params.push(like);
            }
            "uid" => {
                where_sql.push_str(" AND (a.uploader_uid LIKE ? OR a.uploader_name LIKE ?)");
                params.push(like.clone());
                params.push(like);
            }
            "tag" => {
                where_sql.push_str(" AND a.tags_norm LIKE ?");
                params.push(like);
            }
            _ => {
                where_sql.push_str(
                    " AND (a.title LIKE ? OR a.description LIKE ? OR a.tags_norm LIKE ? \
                     OR a.uploader_uid LIKE ? OR a.uploader_name LIKE ?)",
                );
                for _ in 0..5 {
                    params.push(like.clone());
                }
            }
        }
    }

    // 总数
    let count_sql = format!("SELECT COUNT(1) AS c FROM artworks a {where_sql}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        count_q = count_q.bind(p);
    }
    let total: i64 = count_q.fetch_one(&state.pools.art).await?;

    let offset = (page - 1) * page_size;

    // 排序
    let mut order_by = String::from("ORDER BY datetime(a.created_at) DESC, a.id DESC");
    let mut order_params: Vec<i64> = Vec::new();
    let mut seed_used: Option<i64> = None;

    if sort == "likes" {
        order_by =
            "ORDER BY COALESCE(CAST(NULLIF(TRIM(a.like_total), '') AS INTEGER), 0) DESC, datetime(a.created_at) DESC, a.id DESC".into();
    } else if sort == "time" {
        order_by = "ORDER BY datetime(a.created_at) DESC, a.id DESC".into();
    } else if sort == "random" {
        let seed = match getq("seed") {
            Some(s) if !s.trim().is_empty() => clamp_int(Some(s), 0, 2147483647, 0),
            _ => rand_int(0, 2147483647),
        };
        seed_used = Some(seed);
        // 与旧实现位运算 XOR 混合一致
        order_by = "ORDER BY (((a.id + ? - 2 * (a.id & ?)) * 2654435761) % 2147483647 + 1) * 1103515245 % 2147483647 ASC".into();
        order_params.push(seed);
        order_params.push(seed);
    }

    let list_sql = format!("{SELECT_ART} {where_sql} {order_by} LIMIT ? OFFSET ?");
    let mut list_q = sqlx::query_as::<_, ArtRow>(&list_sql);
    for p in &params {
        list_q = list_q.bind(p);
    }
    for p in &order_params {
        list_q = list_q.bind(p);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows: Vec<ArtRow> = list_q.fetch_all(&state.pools.art).await?;
    let data: Vec<Value> = rows.iter().map(map_artwork_row).collect();

    Ok(Json(json!({
        "ok": true,
        "data": data,
        "total": total,
        "sortUsed": sort,
        "seedUsed": seed_used,
        "debugId": chrono::Utc::now().timestamp_millis(),
    })))
}

// 6. 作品详情
async fn get_artwork(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let (anon_id, set) = resolve_anon(&state.cfg.art_cookie_secret, &headers);

    let row: Option<ArtRow> = sqlx::query_as(&format!("{SELECT_ART} WHERE a.id = ?"))
        .bind(id)
        .fetch_optional(&state.pools.art)
        .await?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                json_with_cookie(json!({ "ok": false, "message": "Artwork not found" }), set),
            )
                .into_response())
        }
    };

    // status 非 approved 且非本人（uploader_uid == anon_id）→ 404
    let status = row.status.as_deref().unwrap_or("");
    let uploader_uid = row.uploader_uid.as_deref().unwrap_or("");
    if status != "approved" && uploader_uid != anon_id {
        return Ok((
            StatusCode::NOT_FOUND,
            json_with_cookie(
                json!({ "ok": false, "message": "Artwork not found or restricted" }),
                set,
            ),
        )
            .into_response());
    }

    Ok(json_with_cookie(
        json!({ "ok": true, "data": map_artwork_row(&row) }),
        set,
    ))
}

// 6.5 缩略图：GET /thumb?path=art/2026-02/x.webp&w=640
// 生成 + 磁盘缓存（uploads/art/.thumbs/<w>/<path>.<ext>.webp）。生产由 nginx 对
// .thumbs/ 静态直出、未命中才回源本端点；缓存全量预热（deploy/backfill-thumbs.sh）
// 后，本端点几乎只在新图首访时被命中。生成走 libvips 子进程（内存有界）。

/// 允许的缩略宽度白名单（防御任意 w 撑爆缓存目录）。
const THUMB_WIDTHS: &[u32] = &[320, 640, 960];
/// 缩略图 WebP 质量（vips webpsave 的 Q 参数，0-100）。
const THUMB_QUALITY: u8 = 82;
/// 缩略图生成并发闸：libvips 子进程虽内存有界，但冷缓存下一页网格会同时回源
/// 多张；限并发=2（对齐生产 2 核），杜绝突发把小内存机器叠爆。
static THUMB_GATE: std::sync::LazyLock<tokio::sync::Semaphore> =
    std::sync::LazyLock::new(|| tokio::sync::Semaphore::new(2));

#[derive(serde::Deserialize)]
struct ThumbQuery {
    path: String,
    w: Option<u32>,
}

/// 校验并规范化 path：必须是 uploads 根下 art/ 内的相对路径，拒绝任何穿越成分。
fn sanitize_thumb_path(raw: &str) -> Option<String> {
    let p = raw.trim().trim_start_matches('/');
    if !p.starts_with("art/") || p.contains('\\') || p.contains('\0') {
        return None;
    }
    // 逐段校验：禁止 ".."、"."、空段（"a//b"）
    if p.split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return None;
    }
    Some(p.to_string())
}

/// 源文件（art/ 相对路径）在宽度 w 下的缩略缓存磁盘路径。
fn thumb_cache_path(state: &AppState, rel: &str, w: u32) -> std::path::PathBuf {
    let ext = haruhi_media::ext_of(rel, "");
    state
        .cfg
        .uploads_subdir("art")
        .join(".thumbs")
        .join(w.to_string())
        .join(rel.trim_start_matches("art/"))
        .with_extension(format!("{ext}.webp"))
}

/// 用 libvips 生成缩略图并原子落盘到缓存（tmp + rename，防并发读到半成品）。
/// 受 THUMB_GATE 限并发；vips 内存有界，从根上避免进程内全解码的 RSS 膨胀。
async fn build_thumb(src: &std::path::Path, w: u32, cache: &std::path::Path) -> AppResult<()> {
    let _permit = THUMB_GATE
        .acquire()
        .await
        .map_err(|e| AppError::internal(format!("缩略图限流器异常: {e}")))?;
    if let Some(dir) = cache.parent() {
        // 不吞错：权限/磁盘故障应明确暴露，而非伪装成"生成失败"反复回源
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| AppError::internal(format!("缩略图缓存目录创建失败: {e}")))?;
    }
    let tmp = cache.with_extension(format!("tmp{:x}.webp", rand_hex()));
    match haruhi_media::thumbnail_webp_vips(src, &tmp, w, THUMB_QUALITY).await {
        Ok(()) => {
            if tokio::fs::rename(&tmp, cache).await.is_err() {
                let _ = tokio::fs::remove_file(&tmp).await;
                return Err(AppError::internal("缩略图落盘失败"));
            }
            Ok(())
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp).await;
            Err(AppError::internal(format!("缩略图生成失败: {e}")))
        }
    }
}

async fn get_thumb(
    State(state): State<AppState>,
    Query(q): Query<ThumbQuery>,
) -> AppResult<Response> {
    let w = q.w.unwrap_or(640);
    if !THUMB_WIDTHS.contains(&w) {
        return Err(AppError::bad_request("不支持的缩略宽度"));
    }
    let rel = sanitize_thumb_path(&q.path).ok_or_else(|| AppError::bad_request("非法路径"))?;

    // gif（动图）/ svg（矢量）不转码，直接交回静态服务，保留原始观感
    let ext = haruhi_media::ext_of(&rel, "");
    if ext == "gif" || ext == "svg" {
        return Ok(axum::response::Redirect::permanent(&format!("/uploads/{rel}")).into_response());
    }

    let src = state.cfg.uploads_dir.join(&rel);
    let cache = thumb_cache_path(&state, &rel, w);

    // 缓存未命中才生成。源不存在→404；生成失败（vips 缺失/坏图）→回退原图（302），
    // 保证网格不裂，且 vips 修复后下次访问会重新生成。
    if tokio::fs::metadata(&cache).await.is_err() {
        if tokio::fs::metadata(&src).await.is_err() {
            return Err(AppError::not_found("图片不存在"));
        }
        if let Err(e) = build_thumb(&src, w, &cache).await {
            tracing::warn!(rel = %rel, error = %e, "缩略图生成失败，回退原图");
            return Ok(
                axum::response::Redirect::temporary(&format!("/uploads/{rel}")).into_response(),
            );
        }
    }

    let bytes = tokio::fs::read(&cache)
        .await
        .map_err(|_| AppError::internal("缩略图读取失败"))?;

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "image/webp"),
            // 缩略图内容随源文件名唯一（文件名带时间戳+随机串），可放心 immutable
            ("Cache-Control", "public, max-age=31536000, immutable"),
        ],
        bytes,
    )
        .into_response())
}

// 7. 创建作品（multipart：images[] + originals[] + 文本字段；公开匿名，按 IP 限流）
async fn create_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    headers: axum::http::HeaderMap,
    mut mp: Multipart,
) -> AppResult<Response> {
    // 必须登录 + 邮箱已验证：取代过去客户端自报 uploader_uid/name 的半匿名上传
    let member_name = crate::auth_routes::require_verified_member(&state.pools.core, &user).await?;
    let ip = crate::ratelimit::client_ip(&headers);
    if let Err(secs) = state.upload_limiter.check_and_record(&ip) {
        return Err(AppError::TooManyRequests(format!(
            "上传过于频繁，请 {secs} 秒后再试"
        )));
    }
    // 收集字段
    let mut display_files: Vec<(String, Bytes)> = Vec::new(); // (orig_name, bytes)
    let mut original_files: Vec<(String, Bytes)> = Vec::new();
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "images" => {
                let fname = field.file_name().unwrap_or("image.bin").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("读取文件失败: {e}")))?;
                // 类型/大小白名单：画廊为公开匿名上传口，仅接受图片，防任意文件滥用。
                let ext = haruhi_media::ext_of(&fname, "");
                haruhi_media::check_image(&ext, bytes.len())
                    .map_err(|r| AppError::bad_request(r.to_string()))?;
                display_files.push((fname, bytes));
            }
            "originals" => {
                let fname = field.file_name().unwrap_or("image.bin").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("读取文件失败: {e}")))?;
                let ext = haruhi_media::ext_of(&fname, "");
                haruhi_media::check_image(&ext, bytes.len())
                    .map_err(|r| AppError::bad_request(r.to_string()))?;
                original_files.push((fname, bytes));
            }
            _ => {
                let txt = field.text().await.unwrap_or_default();
                fields.insert(name, txt);
            }
        }
    }

    if display_files.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "缺少图片文件" })),
        )
            .into_response());
    }

    // 落盘：uploads/art/<YYYY-MM>/<ts>-<rand>.<ext>，库存 art/<YYYY-MM>/<file>
    let now = chrono::Utc::now();
    let folder = now.format("%Y-%m").to_string();
    let art_root = state.cfg.uploads_subdir("art");
    let month_dir = art_root.join(&folder);

    let mut images_list: Vec<Value> = Vec::new();
    let mut cover_disp_rel = String::new();
    let mut cover_orig_rel = String::new();
    let mut cover_bytes: Option<Bytes> = None;

    for (i, (disp_name, disp_bytes)) in display_files.iter().enumerate() {
        let (orig_name, orig_bytes) = original_files
            .get(i)
            .map(|(n, b)| (n.as_str(), b))
            .unwrap_or((disp_name.as_str(), disp_bytes));

        let disp_ext = haruhi_media::ext_of(disp_name, "bin");
        let orig_ext = haruhi_media::ext_of(orig_name, "bin");
        let disp_file = format!("{}-{:x}.{}", now.timestamp_millis(), rand_hex(), disp_ext);
        let orig_file = format!("{}-{:x}.{}", now.timestamp_millis(), rand_hex(), orig_ext);

        haruhi_media::save_file(&month_dir, &disp_file, disp_bytes).await?;
        let rel_disp = format!("art/{folder}/{disp_file}");

        // 后台预热画廊缩略图：新作品过审后首个访客无需等待现场生成。
        // 失败无害（/thumb 端点按需兜底），gif/svg 本就不转码故跳过。
        if disp_ext != "gif" && disp_ext != "svg" {
            // 文件已落盘，按磁盘路径让 vips 生成（不再持有整段字节）
            let src = state.cfg.uploads_dir.join(&rel_disp);
            let cache = thumb_cache_path(&state, &rel_disp, 640);
            tokio::spawn(async move {
                let _ = build_thumb(&src, 640, &cache).await;
            });
        }
        let rel_orig = if std::ptr::eq(orig_bytes, disp_bytes) {
            // originals 缺位时与 display 同文件（对齐旧 orig=disp）
            rel_disp.clone()
        } else {
            haruhi_media::save_file(&month_dir, &orig_file, orig_bytes).await?;
            format!("art/{folder}/{orig_file}")
        };

        if i == 0 {
            cover_disp_rel = rel_disp.clone();
            cover_orig_rel = rel_orig.clone();
            cover_bytes = Some(disp_bytes.clone());
        }
        images_list.push(json!({ "path": rel_disp, "original": rel_orig }));
    }

    let images_json = serde_json::to_string(&images_list).unwrap_or_else(|_| "[]".into());

    let get = |k: &str| fields.get(k).map(|s| s.as_str());
    let title = safe_text(get("title"));
    let description = safe_text(get("description"));
    // 署名与身份键一律取自登录账号，忽略客户端自报（uploader_uid = "u{id}" 沿用积分/创作者体系）
    let uploader_name = member_name;
    let uploader_uid = crate::auth_routes::member_uid(user.id);
    let source_type = {
        let s = safe_text(get("source_type"));
        if s.is_empty() {
            "network".to_string()
        } else {
            s
        }
    };
    let content_type = {
        let s = safe_text(get("content_type"));
        if s.is_empty() {
            "haruhi".to_string()
        } else {
            s
        }
    };
    let origin_url = safe_text(get("origin_url"));

    if title.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "作品名称为必填" })),
        )
            .into_response());
    }

    // AI 审核
    let ai = haruhi_ai::AiClient::from_config(&state.cfg);
    let text_check = ai
        .check_text(
            haruhi_ai::ART_SYSTEM_PROMPT,
            &format!("{title}\n{description}"),
        )
        .await;
    let image_verdict = match &cover_bytes {
        Some(b) => {
            ai.check_image(haruhi_ai::ART_SYSTEM_PROMPT, b, "image/webp")
                .await
        }
        None => haruhi_ai::Verdict {
            ok: true,
            reason: "EMPTY".into(),
        },
    };

    // 状态机（对齐旧 finalStatus 逻辑）
    let mut final_status = "approved".to_string();
    let mut ai_reason: Vec<String> = Vec::new();

    if !text_check.ok {
        final_status = "flagged".into();
        ai_reason.push(format!("文本: {}", text_check.reason));
    } else if text_check.reason == "AI_API_ERROR" || text_check.reason == "AI_OFFLINE" {
        final_status = "pending".into();
        ai_reason.push("AI文本服务异常".into());
    }

    if final_status != "flagged" {
        if !image_verdict.ok {
            final_status = "flagged".into();
            ai_reason.push(format!("封面图: {}", image_verdict.reason));
        } else if image_verdict.reason == "AI_API_ERROR" || image_verdict.reason == "AI_OFFLINE" {
            if final_status == "approved" {
                final_status = "pending".into();
            }
            ai_reason.push("AI视觉服务异常".into());
        }
    }

    let tags_arr = normalize_tags_to_array(get("tags"));
    let tags_json = serde_json::to_string(&tags_arr).unwrap_or_else(|_| "[]".into());
    let tags_norm = make_tags_norm(&tags_arr);
    let licenses_arr = parse_licenses(get("licenses"));
    let licenses_json = serde_json::to_string(&licenses_arr).unwrap_or_else(|_| "[]".into());

    let created_at = now_iso();
    let review_note = ai_reason.join("; ");
    let reviewed_at: Option<String> = if final_status == "approved" {
        Some(created_at.clone())
    } else {
        None
    };

    let _ = (cover_disp_rel.is_empty(), cover_orig_rel.is_empty()); // 保留语义：封面=images[0]
    let cover_path = &images_list[0]["path"].as_str().unwrap_or("").to_string();
    let cover_original = &images_list[0]["original"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let last_id: i64 = sqlx::query_scalar(
        "INSERT INTO artworks \
         (title, description, uploader_name, uploader_uid, author_user_id, source_type, content_type, tags_json, tags_norm, origin_url, \
          file_path, file_path_original, status, review_note, reviewed_at, created_at, licenses_json, ai_reason, images_json) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&title)
    .bind(&description)
    .bind(opt(&uploader_name))
    .bind(opt(&uploader_uid))
    .bind(user.id)
    .bind(&source_type)
    .bind(&content_type)
    .bind(&tags_json)
    .bind(&tags_norm)
    .bind(opt(&origin_url))
    .bind(cover_path)
    .bind(cover_original)
    .bind(&final_status)
    .bind(&review_note)
    .bind(&reviewed_at)
    .bind(&created_at)
    .bind(&licenses_json)
    .bind(&review_note)
    .bind(&images_json)
    .fetch_one(&state.pools.art)
    .await?;

    // AI 拦截（flagged）→ 通知管理员（异步、不阻塞、失败仅记日志）
    if final_status == "flagged" {
        crate::notify::ai_flagged(
            &state,
            "art",
            "作品",
            &title,
            &last_id.to_string(),
            &review_note,
        );
    }

    // 自动发积分（approved + personal + 有 uid）
    let mut points_added = false;
    if final_status == "approved" && source_type == "personal" && !uploader_uid.is_empty() {
        let (points, note) = if content_type == "haruhi" {
            (120, "投稿凉宫个人作品奖励")
        } else {
            (30, "投稿其他个人作品奖励")
        };
        if points > 0 {
            let res = sqlx::query(
                "INSERT INTO points_ledger(uid, artwork_id, points, note, created_at, granted_at) VALUES(?,?,?,?,?,?)",
            )
            .bind(&uploader_uid)
            .bind(last_id)
            .bind(points)
            .bind(note)
            .bind(&created_at)
            .bind(&created_at)
            .execute(&state.pools.art)
            .await;
            match res {
                Ok(_) => points_added = true,
                Err(e) => tracing::error!("Auto grant points failed: {e}"),
            }
        }
    }

    let message = if final_status == "approved" {
        "发布成功"
    } else {
        "提交成功，待审核"
    };
    Ok(Json(json!({
        "ok": true,
        "status": final_status,
        "pointsAdded": points_added,
        "message": message,
    }))
    .into_response())
}

// 8. 评论列表
async fn list_comments(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Response> {
    let artwork_id = q.get("artwork_id").and_then(|s| s.parse::<i64>().ok());
    let artwork_id = match artwork_id {
        Some(id) => id,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "message": "artwork_id 无效" })),
            )
                .into_response())
        }
    };
    let rows: Vec<(i64, i64, Option<String>, Option<i64>, Option<String>, i64, Option<String>)> =
        sqlx::query_as(
            "SELECT id, artwork_id, user_name, \
             CAST(NULLIF(TRIM(avatar_key), '') AS INTEGER) AS avatar_key, body, \
             COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0) AS like_total, created_at \
             FROM comments WHERE artwork_id=? AND status='public' ORDER BY datetime(created_at) ASC",
        )
        .bind(artwork_id)
        .fetch_all(&state.pools.art)
        .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, artwork_id, user_name, avatar_key, body, like_total, created_at)| {
                json!({
                    "id": id, "artwork_id": artwork_id, "user_name": user_name,
                    "avatar_key": avatar_key, "body": body, "like_total": like_total,
                    "created_at": created_at,
                })
            },
        )
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })).into_response())
}

// 9. 创建评论
async fn create_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    // 必须登录 + 邮箱已验证；署名取账号昵称，身份键取 "u{id}"（不再读匿名 cookie）
    let user_name = crate::auth_routes::require_verified_member(&state.pools.core, &user).await?;
    let anon_id = crate::auth_routes::member_uid(user.id);
    let set: Option<[String; 2]> = None;

    let artwork_id = body.get("artwork_id").and_then(json_num_i64);
    let comment_body = clamp_len(body.get("body").and_then(|v| v.as_str()), 800);

    let artwork_id = match artwork_id {
        Some(id) => id,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                json_with_cookie(json!({ "ok": false, "message": "artwork_id 无效" }), set),
            )
                .into_response())
        }
    };
    if user_name.is_empty() || comment_body.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            json_with_cookie(json!({ "ok": false, "message": "必填项缺失" }), set),
        )
            .into_response());
    }

    let ai = haruhi_ai::AiClient::from_config(&state.cfg);
    let check = ai
        .check_text(
            haruhi_ai::ART_SYSTEM_PROMPT,
            &format!("{user_name}: {comment_body}"),
        )
        .await;

    let mut status = "public".to_string();
    let mut ai_reason = String::new();
    if !check.ok {
        status = "flagged".into();
        ai_reason = check.reason.clone();
    } else if check.reason == "AI_API_ERROR" || check.reason == "AI_OFFLINE" {
        status = "flagged".into();
        ai_reason = "AI服务不可用，转人工".into();
    }

    let avatar_key = rand_int(1, 13); // [1,13) → 1..=12，对齐 crypto.randomInt(1,13)
    let created_at = now_iso();

    let last_id: i64 = sqlx::query_scalar(
        "INSERT INTO comments(artwork_id, anon_id, user_name, avatar_key, body, like_total, created_at, status, ai_reason, author_user_id) \
         VALUES(?,?,?,?,?,0,?,?,?,?) RETURNING id",
    )
    .bind(artwork_id)
    .bind(&anon_id)
    .bind(&user_name)
    .bind(avatar_key)
    .bind(&comment_body)
    .bind(&created_at)
    .bind(&status)
    .bind(&ai_reason)
    .bind(user.id)
    .fetch_one(&state.pools.art)
    .await?;

    // AI 拦截（flagged）→ 通知管理员（异步、不阻塞、失败仅记日志）
    if status == "flagged" {
        let snippet: String = comment_body.chars().take(40).collect();
        crate::notify::ai_flagged(
            &state,
            "art",
            "评论",
            &format!("{user_name}：{snippet}"),
            &last_id.to_string(),
            &ai_reason,
        );
    }

    if status != "public" {
        Ok(json_with_cookie(
            json!({ "ok": true, "message": "评论包含敏感内容或需复核，已转入人工审核", "flagged": true }),
            set,
        ))
    } else {
        Ok(json_with_cookie(
            json!({ "ok": true, "data": {
                "id": last_id, "artwork_id": artwork_id, "user_name": user_name,
                "avatar_key": avatar_key, "body": comment_body, "like_total": 0, "created_at": created_at,
            }}),
            set,
        ))
    }
}

// 10/11. 点赞（artwork / comment）：每日每目标 ≤10 次
async fn like_target(
    state: &AppState,
    anon_id: &str,
    target_type: &str,
    target_id: i64,
) -> (StatusCode, Value) {
    let day = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let now = now_iso();
    let table = if target_type == "artwork" {
        "artworks"
    } else {
        "comments"
    };

    let mut tx = match state.pools.art.begin().await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, json!({ "ok": false })),
    };

    // 目标是否存在
    let exists: Result<Option<i64>, _> =
        sqlx::query_scalar(&format!("SELECT id FROM {table} WHERE id=?"))
            .bind(target_id)
            .fetch_optional(&mut *tx)
            .await;
    match exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            let _ = tx.rollback().await;
            return (StatusCode::NOT_FOUND, json!({ "ok": false }));
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({ "ok": false }));
        }
    }

    // 当日已用次数
    let row: Result<Option<(i64, i64)>, _> = sqlx::query_as(
        "SELECT id, count FROM likes_daily WHERE anon_id=? AND target_type=? AND target_id=? AND day=?",
    )
    .bind(anon_id)
    .bind(target_type)
    .bind(target_id)
    .bind(&day)
    .fetch_optional(&mut *tx)
    .await;
    let row = match row {
        Ok(r) => r,
        Err(_) => {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({ "ok": false }));
        }
    };
    let used = row.map(|(_, c)| c).unwrap_or(0);
    if used >= 10 {
        let _ = tx.rollback().await;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            json!({ "ok": false, "message": "上限" }),
        );
    }

    let step = async {
        if let Some((id, _)) = row {
            sqlx::query("UPDATE likes_daily SET count=count+1, updated_at=? WHERE id=?")
                .bind(&now)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query(
                "INSERT INTO likes_daily(anon_id, target_type, target_id, day, count, created_at, updated_at) \
                 VALUES(?,?,?,?,1,?,?)",
            )
            .bind(anon_id)
            .bind(target_type)
            .bind(target_id)
            .bind(&day)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(&format!(
            "UPDATE {table} SET like_total=COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0)+1 WHERE id=?"
        ))
        .bind(target_id)
        .execute(&mut *tx)
        .await?;
        let updated: i64 =
            sqlx::query_scalar(&format!(
                "SELECT COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0) FROM {table} WHERE id=?"
            ))
                .bind(target_id)
                .fetch_one(&mut *tx)
                .await?;
        Ok::<i64, sqlx::Error>(updated)
    }
    .await;

    match step {
        Ok(updated) => {
            if tx.commit().await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, json!({ "ok": false }));
            }
            (StatusCode::OK, json!({ "ok": true, "totalLikes": updated }))
        }
        Err(_) => {
            let _ = tx.rollback().await;
            (StatusCode::INTERNAL_SERVER_ERROR, json!({ "ok": false }))
        }
    }
}

async fn like_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    // 点赞须登录：每日每目标限额按账号计（anon_id = "u{id}"）
    let anon_id = crate::auth_routes::member_uid(user.id);
    let (status, body) = like_target(&state, &anon_id, "artwork", id).await;
    Ok((status, Json(body)).into_response())
}

async fn like_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let anon_id = crate::auth_routes::member_uid(user.id);
    let (status, body) = like_target(&state, &anon_id, "comment", id).await;
    Ok((status, Json(body)).into_response())
}

/// 认领历史匿名内容：用旧的已签名 `haruhi_anon` cookie，把该匿名身份名下尚未归属的
/// 作品/评论绑定到当前登录账号（取代半匿名后的平滑迁移路径）。
async fn claim_anon(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> AppResult<Json<Value>> {
    let anon_id = match parse_anon_cookie(&state.cfg.art_cookie_secret, &headers) {
        Some(id) => id,
        None => {
            return Ok(Json(
                json!({ "ok": false, "claimed": 0, "message": "未发现可认领的匿名身份" }),
            ))
        }
    };
    let aw = sqlx::query(
        "UPDATE artworks SET author_user_id = ? WHERE uploader_uid = ? AND author_user_id IS NULL",
    )
    .bind(user.id)
    .bind(&anon_id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();
    let cm = sqlx::query(
        "UPDATE comments SET author_user_id = ? WHERE anon_id = ? AND author_user_id IS NULL",
    )
    .bind(user.id)
    .bind(&anon_id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();
    Ok(Json(
        json!({ "ok": true, "claimed": aw + cm, "artworks": aw, "comments": cm }),
    ))
}

// ============================================================
// 个人中心（需登录；一律按 author_user_id 限定为本人内容）
// ============================================================

/// GET /api/art/me/artworks —— 我的作品（按 author_user_id 归属本人，分页 + 可选状态筛选）。
/// 与公开 list_artworks 不同：不限 approved，作者能看到自己 pending/flagged/hidden 的全部作品。
async fn my_artworks(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let getq = |k: &str| q.get(k).map(|s| s.as_str());
    let status = getq("status").unwrap_or("all").to_string();
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 6, 60, 24);
    let offset = (page - 1) * page_size;

    let mut where_sql = String::from("WHERE a.author_user_id = ?");
    if status != "all" {
        where_sql.push_str(" AND a.status = ?");
    }

    let count_sql = format!("SELECT COUNT(1) FROM artworks a {where_sql}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql).bind(user.id);
    if status != "all" {
        count_q = count_q.bind(&status);
    }
    let total: i64 = count_q.fetch_one(&state.pools.art).await?;

    let list_sql = format!(
        "{SELECT_ART} {where_sql} ORDER BY datetime(a.created_at) DESC, a.id DESC LIMIT ? OFFSET ?"
    );
    let mut list_q = sqlx::query_as::<_, ArtRow>(&list_sql).bind(user.id);
    if status != "all" {
        list_q = list_q.bind(&status);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows: Vec<ArtRow> = list_q.fetch_all(&state.pools.art).await?;
    let data: Vec<Value> = rows.iter().map(map_artwork_row).collect();

    Ok(Json(json!({ "ok": true, "data": data, "total": total })))
}

/// GET /api/art/me/comments —— 我的评论（按 author_user_id），附所属作品标题/状态以便跳转。
async fn my_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let page = clamp_int(q.get("page").map(|s| s.as_str()), 1, 9999, 1);
    let page_size = clamp_int(q.get("pageSize").map(|s| s.as_str()), 6, 60, 24);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM comments WHERE author_user_id = ?")
        .bind(user.id)
        .fetch_one(&state.pools.art)
        .await?;

    let rows: Vec<(
        i64,
        i64,
        Option<String>,
        Option<String>,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT cm.id, cm.artwork_id, cm.body, cm.created_at, \
         COALESCE(CAST(NULLIF(TRIM(cm.like_total), '') AS INTEGER), 0) AS like_total, \
         cm.status, a.title AS artwork_title, a.status AS artwork_status \
         FROM comments cm LEFT JOIN artworks a ON a.id = cm.artwork_id \
         WHERE cm.author_user_id = ? ORDER BY datetime(cm.created_at) DESC LIMIT ? OFFSET ?",
    )
    .bind(user.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                artwork_id,
                body,
                created_at,
                like_total,
                status,
                artwork_title,
                artwork_status,
            )| {
                json!({
                    "id": id, "artwork_id": artwork_id, "body": body, "created_at": created_at,
                    "like_total": like_total, "status": status,
                    "artwork_title": artwork_title, "artwork_status": artwork_status,
                })
            },
        )
        .collect();
    Ok(Json(json!({ "ok": true, "data": data, "total": total })))
}

/// GET /api/art/me/points —— 我的画廊积分（创作激励分：余额 + 最近流水）。
/// uid 取 "u{user_id}"，沿用既有 points_ledger 体系。
async fn my_points(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let uid = crate::auth_routes::member_uid(user.id);
    let total: Option<i64> = sqlx::query_scalar(
        "SELECT CAST(SUM(CAST(NULLIF(TRIM(points), '') AS INTEGER)) AS INTEGER) FROM points_ledger WHERE uid=?",
    )
    .bind(&uid)
    .fetch_one(&state.pools.art)
    .await?;
    let total = total.unwrap_or(0);

    let history_rows: Vec<(Option<i64>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT CAST(NULLIF(TRIM(pl.points), '') AS INTEGER) AS points, pl.note, pl.created_at, a.title AS artwork_title \
             FROM points_ledger pl LEFT JOIN artworks a ON a.id = pl.artwork_id \
             WHERE pl.uid=? ORDER BY datetime(pl.created_at) DESC LIMIT 100",
        )
        .bind(&uid)
        .fetch_all(&state.pools.art)
        .await?;
    let history: Vec<Value> = history_rows
        .into_iter()
        .map(|(points, note, created_at, artwork_title)| {
            json!({ "points": points, "note": note, "created_at": created_at, "artwork_title": artwork_title })
        })
        .collect();

    Ok(Json(
        json!({ "ok": true, "uid": uid, "total": total, "history": history }),
    ))
}

/// PATCH /api/art/me/artworks/{id} —— 作者本人编辑作品文本字段（标题/简介/标签/来源链接）。
/// 仅限 author_user_id == 当前用户；状态与审核归属由管理员把关，此处不改 status。
async fn update_my_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let owner: Option<Option<i64>> =
        sqlx::query_scalar("SELECT author_user_id FROM artworks WHERE id=?")
            .bind(id)
            .fetch_optional(&state.pools.art)
            .await?;
    match owner {
        Some(Some(uid)) if uid == user.id => {}
        Some(_) => {
            return Ok((
                StatusCode::FORBIDDEN,
                Json(json!({ "ok": false, "message": "无权编辑该作品" })),
            )
                .into_response())
        }
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(json!({ "ok": false, "message": "作品不存在" })),
            )
                .into_response())
        }
    }

    let s = |k: &str| body.get(k).and_then(|v| v.as_str());
    let title = clamp_len(s("title"), 200);
    if title.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "标题不能为空" })),
        )
            .into_response());
    }
    let description = clamp_len(s("description"), 4000);
    let tags_arr = normalize_tags_to_array(s("tags"));
    let tags_json = serde_json::to_string(&tags_arr).unwrap_or_else(|_| "[]".into());
    let tags_norm = make_tags_norm(&tags_arr);
    let origin_url = clamp_len(s("origin_url"), 500);

    let updated = sqlx::query(
        "UPDATE artworks SET title=?, description=?, tags_json=?, tags_norm=?, origin_url=? \
         WHERE id=? AND author_user_id=?",
    )
    .bind(&title)
    .bind(&description)
    .bind(&tags_json)
    .bind(&tags_norm)
    .bind(&origin_url)
    .bind(id)
    .bind(user.id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();
    // 两次 SQL 之间记录可能被删除或改绑：0 行更新即视为失败，不能谎报成功
    if updated == 0 {
        return Ok((
            StatusCode::CONFLICT,
            Json(json!({ "ok": false, "message": "作品不存在或无权修改" })),
        )
            .into_response());
    }
    Ok(Json(json!({ "ok": true })).into_response())
}

/// DELETE /api/art/me/artworks/{id} —— 作者本人下架作品（软删为 hidden）。
/// 保留行与文件，符合「旧数据保留」原则；彻底删除仍仅限管理员。
async fn delete_my_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let affected =
        sqlx::query("UPDATE artworks SET status='hidden' WHERE id=? AND author_user_id=?")
            .bind(id)
            .bind(user.id)
            .execute(&state.pools.art)
            .await?
            .rows_affected();
    if affected == 0 {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({ "ok": false, "message": "无权操作或作品不存在" })),
        )
            .into_response());
    }
    Ok(Json(json!({ "ok": true, "status": "hidden" })).into_response())
}

/// DELETE /api/art/me/comments/{id} —— 作者本人删除评论（软删为 hidden）。
async fn delete_my_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let affected =
        sqlx::query("UPDATE comments SET status='hidden' WHERE id=? AND author_user_id=?")
            .bind(id)
            .bind(user.id)
            .execute(&state.pools.art)
            .await?
            .rows_affected();
    if affected == 0 {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({ "ok": false, "message": "无权操作或评论不存在" })),
        )
            .into_response());
    }
    Ok(Json(json!({ "ok": true, "status": "hidden" })).into_response())
}

// ============================================================
// 后台接口（RBAC：authorize on app="art"）
// ============================================================

async fn admin_pending_artworks(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status IN ('pending', 'flagged') ORDER BY datetime(a.created_at) DESC"
    ))
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows.iter().map(map_artwork_row).collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_audit_history(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status IN ('approved', 'rejected', 'hidden') \
         ORDER BY datetime(a.reviewed_at) DESC, datetime(a.created_at) DESC LIMIT 500"
    ))
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows.iter().map(map_artwork_row).collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_approve_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "art", Action::Moderate).await?;
    let note = body
        .get("note")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let now = now_iso();

    // 取作品，决定是否补发积分
    let art: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT source_type, uploader_uid, content_type, created_at FROM artworks WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let art = match art {
        Some(a) => a,
        None => return Ok((StatusCode::NOT_FOUND, Json(json!({ "ok": false }))).into_response()),
    };

    sqlx::query("UPDATE artworks SET status='approved', review_note=?, reviewed_at=? WHERE id=?")
        .bind(&note)
        .bind(&now)
        .bind(id)
        .execute(&state.pools.art)
        .await?;

    let (source_type, uploader_uid, content_type, created_at) = art;
    if source_type.as_deref() == Some("personal") {
        if let Some(uid) = uploader_uid.filter(|u| !u.is_empty()) {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM points_ledger WHERE artwork_id=?")
                    .bind(id)
                    .fetch_optional(&state.pools.art)
                    .await?;
            if exists.is_none() {
                let (points, p_note) = if content_type.as_deref() == Some("haruhi") {
                    (120, "投稿凉宫个人作品奖励 (人工复核)")
                } else {
                    (30, "投稿其他个人作品奖励 (人工复核)")
                };
                if points > 0 {
                    let created = created_at.unwrap_or_else(now_iso);
                    if let Err(e) = sqlx::query(
                        "INSERT INTO points_ledger(uid, artwork_id, points, note, created_at, granted_at) VALUES(?,?,?,?,?,?)",
                    )
                    .bind(&uid)
                    .bind(id)
                    .bind(points)
                    .bind(p_note)
                    .bind(&created)
                    .bind(&now)
                    .execute(&state.pools.art)
                    .await
                    {
                        tracing::error!("Manual approval grant points failed: {e}");
                    }
                }
            }
        }
    }
    Ok(Json(json!({ "ok": true })).into_response())
}

async fn admin_reject_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Moderate).await?;
    let note = body
        .get("note")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    sqlx::query("UPDATE artworks SET status='rejected', review_note=?, reviewed_at=? WHERE id=?")
        .bind(&note)
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_artwork_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "art", Action::Moderate).await?;
    let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if ["hidden", "approved", "flagged"].contains(&status) {
        sqlx::query("UPDATE artworks SET status=? WHERE id=?")
            .bind(status)
            .bind(id)
            .execute(&state.pools.art)
            .await?;
        Ok(Json(json!({ "ok": true })).into_response())
    } else {
        Ok((StatusCode::BAD_REQUEST, Json(json!({ "ok": false }))).into_response())
    }
}

async fn admin_artwork_update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let s = |k: &str| body.get(k).and_then(|v| v.as_str());

    let tags_arr = normalize_tags_to_array(s("tags"));
    let tags_json = serde_json::to_string(&tags_arr).unwrap_or_else(|_| "[]".into());
    let tags_norm = make_tags_norm(&tags_arr);
    let licenses_arr = parse_licenses(s("licenses"));
    let licenses_json = serde_json::to_string(&licenses_arr).unwrap_or_else(|_| "[]".into());

    sqlx::query(
        "UPDATE artworks SET title=?, description=?, tags_json=?, tags_norm=?, \
         uploader_name=?, uploader_uid=?, source_type=?, content_type=?, origin_url=?, licenses_json=? WHERE id=?",
    )
    .bind(s("title"))
    .bind(s("description"))
    .bind(&tags_json)
    .bind(&tags_norm)
    .bind(s("uploader_name"))
    .bind(s("uploader_uid"))
    .bind(s("source_type"))
    .bind(s("content_type"))
    .bind(s("origin_url"))
    .bind(&licenses_json)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_delete_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;

    let row: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT file_path, file_path_original, images_json FROM artworks WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    if let Some((file_path, file_path_original, images_json)) = row {
        let mut files: Vec<String> = Vec::new();
        if let Some(f) = file_path.filter(|s| !s.is_empty()) {
            files.push(f);
        }
        if let Some(f) = file_path_original.filter(|s| !s.is_empty()) {
            files.push(f);
        }
        for img in safe_json_arr(images_json.as_deref()) {
            if let Some(p) = img.get("path").and_then(|v| v.as_str()) {
                files.push(p.to_string());
            }
            if let Some(o) = img.get("original").and_then(|v| v.as_str()) {
                files.push(o.to_string());
            }
        }
        let uploads_root = &state.cfg.uploads_dir;
        for f in files {
            let p = uploads_root.join(&f);
            let _ = tokio::fs::remove_file(p).await;
            // 同步清理缩略缓存，避免已删除内容仍可通过缓存访问
            for &w in THUMB_WIDTHS {
                let _ = tokio::fs::remove_file(thumb_cache_path(&state, &f, w)).await;
            }
        }
    }

    sqlx::query("DELETE FROM artworks WHERE id=?")
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    sqlx::query("DELETE FROM comments WHERE artwork_id=?")
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    sqlx::query("DELETE FROM likes_daily WHERE target_type='artwork' AND target_id=?")
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_list_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let status = q
        .get("status")
        .map(|s| s.as_str())
        .unwrap_or("flagged")
        .to_string();

    let rows: Vec<(
        i64,
        i64,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = if status != "all" {
        sqlx::query_as(
                "SELECT id, artwork_id, anon_id, user_name, \
                 CAST(NULLIF(TRIM(avatar_key), '') AS INTEGER) AS avatar_key, body, \
                 COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0) AS like_total, created_at, status, ai_reason \
                 FROM comments WHERE status=? ORDER BY datetime(created_at) DESC LIMIT 200",
            )
            .bind(&status)
            .fetch_all(&state.pools.art)
            .await?
    } else {
        sqlx::query_as(
                "SELECT id, artwork_id, anon_id, user_name, \
                 CAST(NULLIF(TRIM(avatar_key), '') AS INTEGER) AS avatar_key, body, \
                 COALESCE(CAST(NULLIF(TRIM(like_total), '') AS INTEGER), 0) AS like_total, created_at, status, ai_reason \
                 FROM comments ORDER BY datetime(created_at) DESC LIMIT 200",
            )
            .fetch_all(&state.pools.art)
            .await?
    };

    let data: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                artwork_id,
                anon_id,
                user_name,
                avatar_key,
                body,
                like_total,
                created_at,
                status,
                ai_reason,
            )| {
                json!({
                    "id": id, "artwork_id": artwork_id, "anon_id": anon_id, "user_name": user_name,
                    "avatar_key": avatar_key, "body": body, "like_total": like_total,
                    "created_at": created_at, "status": status, "ai_reason": ai_reason,
                })
            },
        )
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_comment_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Moderate).await?;
    let status = body.get("status").and_then(|v| v.as_str());
    sqlx::query("UPDATE comments SET status=? WHERE id=?")
        .bind(status)
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_delete_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("DELETE FROM comments WHERE id=?")
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_list_creators(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT uid, avatar_url, created_at, qq FROM creators ORDER BY datetime(created_at) DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(uid, avatar_url, created_at, qq)| {
            json!({ "uid": uid, "avatar_url": avatar_url, "created_at": created_at, "qq": qq })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_create_creator(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let uid = body
        .get("uid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !uid.is_empty() {
        let exists: Option<String> = sqlx::query_scalar("SELECT uid FROM creators WHERE uid=?")
            .bind(&uid)
            .fetch_optional(&state.pools.art)
            .await?;
        if exists.is_none() {
            sqlx::query("INSERT INTO creators(uid, avatar_url, created_at) VALUES(?,'',?)")
                .bind(&uid)
                .bind(now_iso())
                .execute(&state.pools.art)
                .await?;
        }
    }
    Ok(Json(json!({ "ok": true })))
}

// 创作者更新：multipart（avatar 文件可选 + new_uid + qq）
async fn admin_update_creator(
    State(state): State<AppState>,
    user: AuthUser,
    Path(old_uid): Path<String>,
    mut mp: Multipart,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;

    let mut new_uid = String::new();
    let mut qq = String::new();
    let mut avatar: Option<(String, Bytes)> = None;
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        match field.name().unwrap_or("") {
            "new_uid" => new_uid = field.text().await.unwrap_or_default().trim().to_string(),
            "qq" => qq = field.text().await.unwrap_or_default().trim().to_string(),
            "avatar" => {
                let fname = field.file_name().unwrap_or("avatar.bin").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("读取头像失败: {e}")))?;
                avatar = Some((fname, bytes));
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    if old_uid.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "Missing param" })),
        )
            .into_response());
    }

    let creator: Option<(String,)> = sqlx::query_as("SELECT uid FROM creators WHERE uid=?")
        .bind(&old_uid)
        .fetch_optional(&state.pools.art)
        .await?;
    if creator.is_none() {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "message": "Creator not found" })),
        )
            .into_response());
    }

    let mut tx = state.pools.art.begin().await?;
    let txn = async {
        let mut final_uid = old_uid.clone();
        // UID 变更：检查冲突并级联更新
        if !new_uid.is_empty() && new_uid != old_uid {
            let conflict: Option<i64> = sqlx::query_scalar("SELECT 1 FROM creators WHERE uid=?")
                .bind(&new_uid)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            if conflict.is_some() {
                return Err(format!("UID \"{new_uid}\" already exists"));
            }
            sqlx::query("UPDATE creators SET uid=? WHERE uid=?")
                .bind(&new_uid)
                .bind(&old_uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query("UPDATE artworks SET uploader_uid=? WHERE uploader_uid=?")
                .bind(&new_uid)
                .bind(&old_uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query("UPDATE points_ledger SET uid=? WHERE uid=?")
                .bind(&new_uid)
                .bind(&old_uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            final_uid = new_uid.clone();
        }
        // 更新 QQ
        sqlx::query("UPDATE creators SET qq=? WHERE uid=?")
            .bind(&qq)
            .bind(&final_uid)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        Ok::<String, String>(final_uid)
    }
    .await;

    let final_uid = match txn {
        Ok(uid) => uid,
        Err(msg) => {
            let _ = tx.rollback().await;
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "message": msg })),
            )
                .into_response());
        }
    };

    // 头像落盘（事务外，对齐旧逻辑在事务内但落盘失败语义差异极小；这里在事务内更新 DB）
    if let Some((fname, bytes)) = avatar {
        let now = chrono::Utc::now();
        let folder = now.format("%Y-%m").to_string();
        let ext = haruhi_media::ext_of(&fname, "bin");
        let file = format!("{}-{:x}.{}", now.timestamp_millis(), rand_hex(), ext);
        let dir = state.cfg.uploads_subdir("art").join(&folder);
        if let Err(e) = haruhi_media::save_file(&dir, &file, &bytes).await {
            let _ = tx.rollback().await;
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "message": e.to_string() })),
            )
                .into_response());
        }
        let avatar_url = format!("uploads/art/{folder}/{file}");
        if let Err(e) = sqlx::query("UPDATE creators SET avatar_url=? WHERE uid=?")
            .bind(&avatar_url)
            .bind(&final_uid)
            .execute(&mut *tx)
            .await
        {
            let _ = tx.rollback().await;
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "message": e.to_string() })),
            )
                .into_response());
        }
    }

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

async fn admin_delete_creator(
    State(state): State<AppState>,
    user: AuthUser,
    Path(uid): Path<String>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("DELETE FROM creators WHERE uid=?")
        .bind(&uid)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_points_ledger(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<(
        Option<String>,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT pl.uid, CAST(NULLIF(TRIM(pl.points), '') AS INTEGER) AS points, pl.granted_at, pl.note, a.title AS artwork_title \
             FROM points_ledger pl LEFT JOIN artworks a ON a.id=pl.artwork_id \
             ORDER BY datetime(pl.granted_at) DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|(uid, points, granted_at, note, artwork_title)| {
            json!({ "uid": uid, "points": points, "granted_at": granted_at, "note": note, "artwork_title": artwork_title })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn admin_points_grant(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let uid = body.get("uid").and_then(|v| v.as_str());
    let artwork_id = body.get("artwork_id").and_then(json_num_i64);
    let points = body.get("points").and_then(json_num_i64);
    let note = body.get("note").and_then(|v| v.as_str());
    let now = now_iso();
    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, created_at, granted_at) VALUES(?,?,?,?,?,?)",
    )
    .bind(uid)
    .bind(artwork_id)
    .bind(points)
    .bind(note)
    .bind(&now)
    .bind(&now)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

// ============================================================
// 杂项辅助
// ============================================================

/// 空串映射为 NULL（对齐旧 `x || null`）。
fn opt(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// 从 JSON 取整数（兼容数字或数字字符串，对齐旧 Number()）。
fn json_num_i64(v: &Value) -> Option<i64> {
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

/// [min, max) 随机整数（对齐 crypto.randomInt(min, max)）。
fn rand_int(min: i64, max: i64) -> i64 {
    use rand::Rng;
    if max <= min {
        return min;
    }
    rand::thread_rng().gen_range(min..max)
}

/// 16 位十六进制随机片段（对齐 Math.random().toString(16).slice(2)）。
fn rand_hex() -> u64 {
    use rand::Rng;
    rand::thread_rng().gen::<u64>()
}
