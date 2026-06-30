//! art 模块：绘画部画廊（作品上传/审核、评论、点赞、积分、创作者管理）。
//! 忠实移植旧 haruhi-art-club 后端（server/index.js + db.js + ai.js），统一挂载于 /api/art。
//!
//! 与旧实现的差异：
//! - 旧路径在根 /api/* 下，这里统一前缀 /api/art/*（router nest 在 /art）。
//! - 后台管理由旧的 x-admin-password 头改为 JWT + RBAC（authorize/AuthUser）。
//! - 匿名身份签名 Cookie（haruhi_anon）行为忠实保留：HMAC-SHA256 签名校验/下发。

use axum::body::Bytes;
use axum::extract::multipart::Field;
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
use std::path::{Path as FsPath, PathBuf};
use tokio::io::AsyncWriteExt;

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
        .route("/visitors", post(record_visitor))
        .route("/announcements", get(list_announcements))
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
        .route(
            "/admin/announcements",
            get(admin_list_announcements).post(admin_create_announcement),
        )
        .route(
            "/admin/announcements/{id}/update",
            post(admin_update_announcement),
        )
        .route(
            "/admin/announcements/{id}",
            axum::routing::delete(admin_delete_announcement),
        )
        .route("/admin/points-ledger", get(admin_points_ledger))
        .route("/admin/points/grant", post(admin_points_grant))
        .merge(super::art_guild::router())
}

// ============================================================
// 匿名身份 Cookie（haruhi_anon）：HMAC-SHA256 签名
// ============================================================

const COOKIE_NAME: &str = "haruhi_anon";
const COOKIE_SIG: &str = "haruhi_anon_sig";
const VISITOR_SESSION_WINDOW_MINUTES: i64 = 10;

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
pub(crate) async fn member_display_names(
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

/// 批量映射作品行，并按 uploader_uid 注入实时账号昵称 uploader_display_name（昵称优先）。
/// 历史匿名作品（uploader_uid 非 u{id} 或查不到）不注入，前端回退显示 uploader_name 快照。
async fn map_artworks_with_names(state: &AppState, rows: &[ArtRow]) -> Vec<Value> {
    let uids: Vec<String> = rows.iter().filter_map(|r| r.uploader_uid.clone()).collect();
    let names = member_display_names(&state.pools.core, &uids).await;
    rows.iter()
        .map(|r| {
            let mut v = map_artwork_row(r);
            if let (Some(uid), Some(obj)) = (r.uploader_uid.as_deref(), v.as_object_mut()) {
                if let Some(n) = names.get(uid) {
                    obj.insert("uploader_display_name".to_string(), json!(n));
                }
            }
            v
        })
        .collect()
}

async fn count_random_art_segment(
    pool: &sqlx::SqlitePool,
    where_sql: &str,
    params: &[String],
    after_pivot: bool,
    pivot: i64,
) -> AppResult<i64> {
    let cmp = if after_pivot { ">=" } else { "<" };
    let sql = format!("SELECT COUNT(1) FROM artworks a {where_sql} AND a.random_key {cmp} ?");
    let mut q = sqlx::query_scalar::<_, i64>(&sql);
    for p in params {
        q = q.bind(p);
    }
    Ok(q.bind(pivot).fetch_one(pool).await?)
}

async fn fetch_random_art_segment(
    pool: &sqlx::SqlitePool,
    where_sql: &str,
    params: &[String],
    after_pivot: bool,
    pivot: i64,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<ArtRow>> {
    if limit <= 0 {
        return Ok(vec![]);
    }
    let cmp = if after_pivot { ">=" } else { "<" };
    let sql = format!(
        "{SELECT_ART} {where_sql} AND a.random_key {cmp} ? \
         ORDER BY a.random_key ASC, a.id ASC LIMIT ? OFFSET ?"
    );
    let mut q = sqlx::query_as::<_, ArtRow>(&sql);
    for p in params {
        q = q.bind(p);
    }
    Ok(q.bind(pivot)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

async fn fetch_random_artworks(
    pool: &sqlx::SqlitePool,
    where_sql: &str,
    params: &[String],
    seed: i64,
    page_size: i64,
    offset: i64,
) -> AppResult<Vec<ArtRow>> {
    let pivot = seed.rem_euclid(2_147_483_647).max(1);
    let after_count = count_random_art_segment(pool, where_sql, params, true, pivot).await?;
    let mut rows = if offset < after_count {
        fetch_random_art_segment(pool, where_sql, params, true, pivot, page_size, offset).await?
    } else {
        fetch_random_art_segment(
            pool,
            where_sql,
            params,
            false,
            pivot,
            page_size,
            offset - after_count,
        )
        .await?
    };

    let remaining = page_size - rows.len() as i64;
    if remaining > 0 && offset < after_count {
        let mut wrapped =
            fetch_random_art_segment(pool, where_sql, params, false, pivot, remaining, 0).await?;
        rows.append(&mut wrapped);
    }
    Ok(rows)
}

#[derive(Clone)]
struct SavedArtUpload {
    ext: String,
    rel: String,
    path: PathBuf,
}

async fn save_art_upload_field(
    mut field: Field<'_>,
    month_dir: &FsPath,
    folder: &str,
    now_millis: i64,
) -> AppResult<SavedArtUpload> {
    let ext = haruhi_media::ext_of(field.file_name().unwrap_or("image.bin"), "");
    if !haruhi_media::is_image_ext(&ext) {
        return Err(AppError::bad_request(format!("不支持的文件类型: .{ext}")));
    }

    tokio::fs::create_dir_all(month_dir)
        .await
        .map_err(|e| AppError::internal(format!("创建上传目录失败: {e}")))?;
    let stored_file = format!("{now_millis}-{:x}.{ext}", rand_hex());
    let final_path = month_dir.join(&stored_file);
    let tmp_path = month_dir.join(format!(".{stored_file}.{:x}.tmp", rand_hex()));
    let mut out = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| AppError::internal(format!("创建上传临时文件失败: {e}")))?;
    let mut written: usize = 0;

    while let Some(chunk) = match field.chunk().await {
        Ok(chunk) => chunk,
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AppError::bad_request(format!("读取文件失败: {e}")));
        }
    } {
        written = match written.checked_add(chunk.len()) {
            Some(v) => v,
            None => {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(AppError::bad_request("文件过大"));
            }
        };
        if written > haruhi_media::MAX_IMAGE_BYTES {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AppError::bad_request(
                haruhi_media::UploadReject::TooLarge(written, haruhi_media::MAX_IMAGE_BYTES)
                    .to_string(),
            ));
        }
        if let Err(e) = out.write_all(&chunk).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AppError::internal(format!("写入上传文件失败: {e}")));
        }
    }
    out.flush()
        .await
        .map_err(|e| AppError::internal(format!("刷新上传文件失败: {e}")))?;
    drop(out);

    haruhi_media::check_image(&ext, written).map_err(|r| AppError::bad_request(r.to_string()))?;
    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|e| AppError::internal(format!("保存上传文件失败: {e}")))?;

    Ok(SavedArtUpload {
        ext,
        rel: format!("art/{folder}/{stored_file}"),
        path: final_path,
    })
}

async fn cleanup_saved_art_uploads(
    display_files: &[SavedArtUpload],
    original_files: &[SavedArtUpload],
) {
    for file in display_files.iter().chain(original_files.iter()) {
        let _ = tokio::fs::remove_file(&file.path).await;
    }
}

// ============================================================
// 公开接口
// ============================================================

// 0. 访客计数：复用 art 匿名 Cookie；同一身份超过 10 分钟未访问，再计作一次独立访问。
async fn record_visitor(State(state): State<AppState>, headers: HeaderMap) -> AppResult<Response> {
    let (anon_id, set) = resolve_anon(&state.cfg.art_cookie_secret, &headers);
    let now = now_iso();
    let mut tx = state.pools.art.begin().await?;

    // art_visitors 表已随迁移 0003 建好，这里不再运行期建表。
    let insert = sqlx::query(
        "INSERT OR IGNORE INTO art_visitors(anon_id, first_seen_at, last_seen_at, visit_count)
         VALUES(?,?,?,1)",
    )
    .bind(&anon_id)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    let is_new_visitor = insert.rows_affected() > 0;
    let mut counted_visit = is_new_visitor;

    if !is_new_visitor {
        let counted = sqlx::query(
            "UPDATE art_visitors
             SET last_seen_at=?, visit_count=visit_count+1
             WHERE anon_id=?
               AND datetime(last_seen_at, '+' || ? || ' minutes') < datetime(?)",
        )
        .bind(&now)
        .bind(&anon_id)
        .bind(VISITOR_SESSION_WINDOW_MINUTES)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        counted_visit = counted.rows_affected() > 0;

        if !counted_visit {
            sqlx::query("UPDATE art_visitors SET last_seen_at=? WHERE anon_id=?")
                .bind(&now)
                .bind(&anon_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    sqlx::query(
        "INSERT OR IGNORE INTO art_visitor_stats(id, total_visits, unique_visitors, updated_at)
         VALUES(1,0,0,?)",
    )
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    if counted_visit {
        let unique_delta = if is_new_visitor { 1_i64 } else { 0_i64 };
        sqlx::query(
            "UPDATE art_visitor_stats
             SET total_visits=total_visits+1,
                 unique_visitors=unique_visitors+?,
                 updated_at=?
             WHERE id=1",
        )
        .bind(unique_delta)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    let (total, unique_visitors): (i64, i64) =
        sqlx::query_as("SELECT total_visits, unique_visitors FROM art_visitor_stats WHERE id=1")
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;

    Ok(json_with_cookie(
        json!({
            "ok": true,
            "total": total,
            "isNew": counted_visit,
            "isNewVisitor": is_new_visitor,
            "uniqueVisitors": unique_visitors
        }),
        set,
    ))
}

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
    }

    let rows: Vec<ArtRow> = if let Some(seed) = seed_used {
        fetch_random_artworks(
            &state.pools.art,
            &where_sql,
            &params,
            seed,
            page_size,
            offset,
        )
        .await?
    } else {
        let list_sql = format!("{SELECT_ART} {where_sql} {order_by} LIMIT ? OFFSET ?");
        let mut list_q = sqlx::query_as::<_, ArtRow>(&list_sql);
        for p in &params {
            list_q = list_q.bind(p);
        }
        list_q = list_q.bind(page_size).bind(offset);
        list_q.fetch_all(&state.pools.art).await?
    };
    // 批量取作者公会徽章，避免逐行 N+1（一次 IN 查询、不建档）。
    let uids: Vec<String> = rows.iter().filter_map(|r| r.uploader_uid.clone()).collect();
    let guild_map = super::art_guild::guild_summaries_for_uids(&state, &uids).await;
    let names = member_display_names(&state.pools.core, &uids).await;
    let data: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut value = map_artwork_row(row);
            let uid = row.uploader_uid.as_deref().unwrap_or("").trim();
            if !uid.is_empty() {
                if let Some(obj) = value.as_object_mut() {
                    if let Some(summary) = guild_map.get(uid) {
                        obj.insert("guild".into(), summary.clone());
                    }
                    if let Some(n) = names.get(uid) {
                        obj.insert("uploader_display_name".into(), json!(n));
                    }
                }
            }
            value
        })
        .collect();

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
    user: Option<AuthUser>,
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

    super::art_guild::record_user_event(&state, user, "browse_artwork", Some(id)).await;

    let mut value = map_artwork_row(&row);
    let author_uid = row.uploader_uid.as_deref().unwrap_or("").trim();
    if !author_uid.is_empty() {
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "guild".into(),
                super::art_guild::guild_summary_for_uid(&state, author_uid).await,
            );
        }
    }
    Ok(json_with_cookie(json!({ "ok": true, "data": value }), set))
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
    // 落盘：uploads/art/<YYYY-MM>/<ts>-<rand>.<ext>，库存 art/<YYYY-MM>/<file>。
    // 图片字段边读边写临时文件，避免多图上传时把展示图和原图同时压进内存。
    let now = chrono::Utc::now();
    let now_millis = now.timestamp_millis();
    let folder = now.format("%Y-%m").to_string();
    let art_root = state.cfg.uploads_subdir("art");
    let month_dir = art_root.join(&folder);

    // 收集字段
    let mut display_files: Vec<SavedArtUpload> = Vec::new();
    let mut original_files: Vec<SavedArtUpload> = Vec::new();
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "images" => match save_art_upload_field(field, &month_dir, &folder, now_millis).await {
                Ok(file) => display_files.push(file),
                Err(e) => {
                    cleanup_saved_art_uploads(&display_files, &original_files).await;
                    return Err(e);
                }
            },
            "originals" => {
                match save_art_upload_field(field, &month_dir, &folder, now_millis).await {
                    Ok(file) => original_files.push(file),
                    Err(e) => {
                        cleanup_saved_art_uploads(&display_files, &original_files).await;
                        return Err(e);
                    }
                }
            }
            _ => {
                let txt = field.text().await.unwrap_or_default();
                fields.insert(name, txt);
            }
        }
    }

    if display_files.is_empty() {
        cleanup_saved_art_uploads(&display_files, &original_files).await;
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "缺少图片文件" })),
        )
            .into_response());
    }

    let mut images_list: Vec<Value> = Vec::new();
    let mut cover_disp_rel = String::new();
    let mut cover_orig_rel = String::new();

    for (i, disp) in display_files.iter().enumerate() {
        let rel_disp = disp.rel.clone();
        // 后台预热画廊缩略图：新作品过审后首个访客无需等待现场生成。
        // 失败无害（/thumb 端点按需兜底），gif/svg 本就不转码故跳过。
        if disp.ext != "gif" && disp.ext != "svg" {
            // 文件已落盘，按磁盘路径让 vips 生成（不再持有整段字节）
            let src = disp.path.clone();
            let cache = thumb_cache_path(&state, &rel_disp, 640);
            tokio::spawn(async move {
                let _ = build_thumb(&src, 640, &cache).await;
            });
        }
        let rel_orig = original_files
            .get(i)
            .map(|orig| orig.rel.clone())
            .unwrap_or_else(|| rel_disp.clone());

        if i == 0 {
            cover_disp_rel = rel_disp.clone();
            cover_orig_rel = rel_orig.clone();
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
        cleanup_saved_art_uploads(&display_files, &original_files).await;
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "作品名称为必填" })),
        )
            .into_response());
    }

    // AI 审核改为后台执行：上传请求只负责持久化文件和记录，避免外部 AI 抖动拖住响应。
    let ai = haruhi_ai::AiClient::from_config(&state.cfg);
    let ai_online = ai.is_online();

    let tags_arr = normalize_tags_to_array(get("tags"));
    let tags_json = serde_json::to_string(&tags_arr).unwrap_or_else(|_| "[]".into());
    let tags_norm = make_tags_norm(&tags_arr);
    let licenses_arr = parse_licenses(get("licenses"));
    let licenses_json = serde_json::to_string(&licenses_arr).unwrap_or_else(|_| "[]".into());

    let created_at = now_iso();
    let final_status = "pending".to_string();
    let review_note = if ai_online {
        "AI审核中".to_string()
    } else {
        "AI文本服务异常; AI视觉服务异常".to_string()
    };
    let reviewed_at: Option<String> = None;

    let _ = (cover_disp_rel.is_empty(), cover_orig_rel.is_empty()); // 保留语义：封面=images[0]
    let cover_path = images_list[0]["path"].as_str().unwrap_or("").to_string();
    let cover_original = images_list[0]["original"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let last_id: i64 = sqlx::query_scalar(
        "INSERT INTO artworks \
         (title, description, uploader_name, uploader_uid, author_user_id, source_type, content_type, tags_json, tags_norm, origin_url, \
          file_path, file_path_original, status, review_note, reviewed_at, created_at, licenses_json, ai_reason, images_json, random_key) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
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
    .bind(&cover_path)
    .bind(&cover_original)
    .bind(&final_status)
    .bind(&review_note)
    .bind(&reviewed_at)
    .bind(&created_at)
    .bind(&licenses_json)
    .bind(&review_note)
    .bind(&images_json)
    .bind(rand_int(1, 2_147_483_647))
    .fetch_one(&state.pools.art)
    .await?;

    let points_added = false;
    if ai_online {
        spawn_artwork_ai_review(
            state.clone(),
            ArtworkAiReviewJob {
                artwork_id: last_id,
                title: title.clone(),
                description: description.clone(),
                cover_path: cover_path.clone(),
            },
        );
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
        "aiReviewPending": ai_online,
        "message": message,
    }))
    .into_response())
}

struct ArtworkAiReviewJob {
    artwork_id: i64,
    title: String,
    description: String,
    cover_path: String,
}

fn spawn_artwork_ai_review(state: AppState, job: ArtworkAiReviewJob) {
    tokio::spawn(async move {
        let artwork_id = job.artwork_id;
        if let Err(e) = run_artwork_ai_review(state, job).await {
            tracing::error!(artwork_id, error = %e, "画廊作品后台 AI 审核失败");
        }
    });
}

async fn run_artwork_ai_review(state: AppState, job: ArtworkAiReviewJob) -> AppResult<()> {
    let artwork_id = job.artwork_id;
    let ai = haruhi_ai::AiClient::from_config(&state.cfg);
    let text_ai = ai.clone();
    let text = format!("{}\n{}", job.title, job.description);
    let text_check = async move {
        text_ai
            .check_text(haruhi_ai::ART_SYSTEM_PROMPT, &text)
            .await
    };
    let image_check = async {
        match tokio::fs::read(state.cfg.uploads_dir.join(&job.cover_path)).await {
            Ok(bytes) => {
                let mime = image_mime_for_path(&job.cover_path);
                ai.check_image(haruhi_ai::ART_SYSTEM_PROMPT, &bytes, mime)
                    .await
            }
            Err(e) => {
                tracing::warn!(artwork_id, path = %job.cover_path, error = %e, "读取封面图用于 AI 审核失败");
                haruhi_ai::Verdict {
                    ok: true,
                    reason: "AI_API_ERROR".into(),
                }
            }
        }
    };
    let (text_verdict, image_verdict) = tokio::join!(text_check, image_check);
    let (final_status, review_note) = artwork_ai_status(text_verdict, image_verdict);
    let reviewed_at: Option<String> = if final_status == "approved" {
        Some(now_iso())
    } else {
        None
    };

    let affected = sqlx::query(
        "UPDATE artworks
         SET status=?, review_note=?, ai_reason=?, reviewed_at=?
         WHERE id=? AND status='pending'",
    )
    .bind(&final_status)
    .bind(&review_note)
    .bind(&review_note)
    .bind(&reviewed_at)
    .bind(artwork_id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();

    if affected == 0 {
        tracing::info!(artwork_id, "作品已不处于 pending，跳过后台 AI 审核结果");
        return Ok(());
    }

    if final_status == "flagged" {
        crate::notify::ai_flagged(
            &state,
            "art",
            "作品",
            &job.title,
            &artwork_id.to_string(),
            &review_note,
        );
    } else if final_status == "approved" {
        // 作品公开：对齐积分（首发即按应得发放）并首次发放声望、推进委托。
        super::art_guild::on_artwork_published(&state, artwork_id, "").await?;
    }

    Ok(())
}

fn image_mime_for_path(path: &str) -> &'static str {
    match haruhi_media::ext_of(path, "").as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "heic" | "heif" => "image/heic",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

fn artwork_ai_status(
    text_check: haruhi_ai::Verdict,
    image_verdict: haruhi_ai::Verdict,
) -> (String, String) {
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

    (final_status, ai_reason.join("; "))
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
        super::art_guild::record_user_event(
            &state,
            Some(user),
            "comment_artwork",
            Some(artwork_id),
        )
        .await;
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
    if status.is_success() {
        super::art_guild::record_user_event(&state, Some(user), "like_artwork", Some(id)).await;
    }
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
    // 作者撤稿：作品不再公开，扣回此前已发放的投稿积分。
    super::art_guild::on_artwork_withdrawn(&state, id).await?;
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
    let data = map_artworks_with_names(&state, &rows).await;
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
    let data = map_artworks_with_names(&state, &rows).await;
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

    let affected = sqlx::query(
        "UPDATE artworks SET status='approved', review_note=?, reviewed_at=? WHERE id=?",
    )
    .bind(&note)
    .bind(&now)
    .bind(id)
    .execute(&state.pools.art)
    .await?
    .rows_affected();
    if affected == 0 {
        return Ok((StatusCode::NOT_FOUND, Json(json!({ "ok": false }))).into_response());
    }

    // 作品公开：对齐积分至应得值（隐藏后复审会重新补发），并在首次公开时发放声望、推进委托。
    super::art_guild::on_artwork_published(&state, id, " (人工复核)").await?;
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
    // 驳回即不公开：扣回此前可能已发放的投稿积分（如曾通过后又被驳回）。
    super::art_guild::on_artwork_withdrawn(&state, id).await?;
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
        // 状态联动积分：转公开则对齐应得（首次公开补声望/委托），转隐藏/标记则扣回。
        if status == "approved" {
            super::art_guild::on_artwork_published(&state, id, "").await?;
        } else {
            super::art_guild::on_artwork_withdrawn(&state, id).await?;
        }
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

    // 统一身份后：上传者名/UID 归用户所有，管理员不再可改，故不写 uploader_name/uploader_uid 两列
    sqlx::query(
        "UPDATE artworks SET title=?, description=?, tags_json=?, tags_norm=?, \
         source_type=?, content_type=?, origin_url=?, licenses_json=? WHERE id=?",
    )
    .bind(s("title"))
    .bind(s("description"))
    .bind(&tags_json)
    .bind(&tags_norm)
    .bind(s("source_type"))
    .bind(s("content_type"))
    .bind(s("origin_url"))
    .bind(&licenses_json)
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    // 来源/内容分类被改可能改变应得积分：按作品当前是否公开重新对齐（公开作品才计分）。
    let is_public: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM artworks WHERE id=? AND status='approved'")
            .bind(id)
            .fetch_optional(&state.pools.art)
            .await?;
    super::art_guild::reconcile_artwork_points(&state, id, is_public.is_some()).await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_delete_artwork(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;

    // 硬删除前先结算积分：作品行删除后无法再溯源，须在此把已发放积分扣回（净额归零）。
    super::art_guild::on_artwork_withdrawn(&state, id).await?;

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

#[derive(sqlx::FromRow)]
struct AdminCreatorRow {
    uid: String,
    avatar_url: Option<String>,
    created_at: Option<String>,
    qq: Option<String>,
    user_id: Option<i64>,
    reputation: i64,
    rating: String,
    access_tier: String,
    coins: i64,
    total_artworks: i64,
    approved_artworks: i64,
    pending_artworks: i64,
    restricted_artworks: i64,
    personal_artworks: i64,
    network_artworks: i64,
    haruhi_artworks: i64,
    other_artworks: i64,
    haruhi_personal_artworks: i64,
    first_upload_at: Option<String>,
    latest_upload_at: Option<String>,
}

async fn admin_list_creators(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<AdminCreatorRow> = sqlx::query_as(
        "SELECT
            c.uid,
            c.avatar_url,
            c.created_at,
            c.qq,
            gp.user_id,
            COALESCE(gp.reputation, 0) AS reputation,
            COALESCE(gp.rating, 'F') AS rating,
            COALESCE(gp.access_tier, 'observer_clearance') AS access_tier,
            COALESCE((
                SELECT CAST(SUM(CAST(NULLIF(TRIM(points), '') AS INTEGER)) AS INTEGER)
                FROM points_ledger pl WHERE pl.uid=c.uid
            ), 0) AS coins,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid) AS total_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.status='approved') AS approved_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.status='pending') AS pending_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.status IN ('rejected','hidden','flagged')) AS restricted_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.source_type='personal') AS personal_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.source_type='network') AS network_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.content_type='haruhi') AS haruhi_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.content_type='other') AS other_artworks,
            (SELECT COUNT(1) FROM artworks a WHERE a.uploader_uid=c.uid AND a.status='approved' AND a.source_type='personal' AND a.content_type='haruhi') AS haruhi_personal_artworks,
            (SELECT MIN(created_at) FROM artworks a WHERE a.uploader_uid=c.uid) AS first_upload_at,
            (SELECT MAX(created_at) FROM artworks a WHERE a.uploader_uid=c.uid) AS latest_upload_at
         FROM creators c
         LEFT JOIN guild_profiles gp ON gp.uid=c.uid
         ORDER BY datetime(c.created_at) DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let uids: Vec<String> = rows.iter().map(|row| row.uid.clone()).collect();
    let names = member_display_names(&state.pools.core, &uids).await;
    let mut user_ids: Vec<i64> = rows
        .iter()
        .filter_map(|row| row.user_id.or_else(|| user_id_from_creator_uid(&row.uid)))
        .collect();
    user_ids.sort_unstable();
    user_ids.dedup();
    let user_contacts = user_display_contacts(&state.pools.core, &user_ids).await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let inferred_user_id = row.user_id.or_else(|| user_id_from_creator_uid(&row.uid));
            let user_contact = inferred_user_id.and_then(|id| user_contacts.get(&id));
            let name = names
                .get(&row.uid)
                .cloned()
                .or_else(|| user_contact.map(|info| info.0.clone()));
            let email = clean_creator_contact(user_contact.and_then(|info| info.1.clone()));
            let qq = clean_creator_contact(row.qq);
            let (contact_type, contact_value) =
                preferred_creator_contact(qq.as_deref(), email.as_deref());
            let contact_label = contact_type.map(creator_contact_type_label);
            let rating_label = super::art_guild::rating_label(&row.rating);
            let access_label = super::art_guild::access_label(&row.access_tier);
            let access_short_label = super::art_guild::access_short_label(&row.access_tier);
            json!({
                "uid": row.uid,
                "name": name,
                "avatar_url": row.avatar_url,
                "created_at": row.created_at,
                "qq": qq,
                "email": email,
                "contactType": contact_type,
                "contactLabel": contact_label,
                "contactValue": contact_value,
                "userId": inferred_user_id,
                "reputation": row.reputation,
                "level": super::art_guild::level_from_reputation(row.reputation),
                "rating": row.rating,
                "ratingLabel": rating_label,
                "accessTier": row.access_tier,
                "accessLabel": access_label,
                "accessShortLabel": access_short_label,
                "coins": row.coins,
                "totalArtworks": row.total_artworks,
                "approvedArtworks": row.approved_artworks,
                "pendingArtworks": row.pending_artworks,
                "restrictedArtworks": row.restricted_artworks,
                "personalArtworks": row.personal_artworks,
                "networkArtworks": row.network_artworks,
                "haruhiArtworks": row.haruhi_artworks,
                "otherArtworks": row.other_artworks,
                "haruhiPersonalArtworks": row.haruhi_personal_artworks,
                "firstUploadAt": row.first_upload_at,
                "latestUploadAt": row.latest_upload_at
            })
        })
        .collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

async fn user_display_contacts(
    core: &sqlx::SqlitePool,
    ids: &[i64],
) -> AppResult<std::collections::HashMap<i64, (String, Option<String>)>> {
    let mut map = std::collections::HashMap::new();
    if ids.is_empty() {
        return Ok(map);
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, nickname, username, email FROM users WHERE id IN ({placeholders}) AND deleted_at IS NULL"
    );
    let mut q = sqlx::query_as::<_, (i64, Option<String>, String, Option<String>)>(&sql);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(core).await?;
    for (id, nickname, username, email) in rows {
        let display_name = nickname
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(username);
        map.insert(id, (display_name, email));
    }
    Ok(map)
}

fn user_id_from_creator_uid(uid: &str) -> Option<i64> {
    uid.strip_prefix('u')?.parse::<i64>().ok()
}

fn clean_creator_contact(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn preferred_creator_contact(
    qq: Option<&str>,
    email: Option<&str>,
) -> (Option<&'static str>, Option<String>) {
    if let Some(qq) = qq.filter(|s| !s.trim().is_empty()) {
        return (Some("qq"), Some(qq.trim().to_string()));
    }
    if let Some(email) = email.filter(|s| !s.trim().is_empty()) {
        return (Some("email"), Some(email.trim().to_string()));
    }
    (None, None)
}

fn creator_contact_type_label(contact_type: &str) -> &'static str {
    match contact_type {
        "qq" => "QQ",
        "email" => "邮箱",
        _ => "联系方式",
    }
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
        super::art_guild::ensure_profile_for_uid(&state, &uid).await?;
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
    super::art_guild::ensure_profile_for_uid(&state, &final_uid).await?;
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

// ============================================================
// 社团公告：公开只读 + 后台 CRUD（替代前端硬编码 mock）
// ============================================================

type AnnouncementRow = (
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    i64,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn announcement_value(r: AnnouncementRow) -> Value {
    let (
        id,
        category,
        title,
        summary,
        body,
        tags_json,
        pinned,
        status,
        published_at,
        created_at,
        updated_at,
    ) = r;
    json!({
        "id": id,
        "category": category,
        "title": title,
        "summary": summary,
        "body": body,
        "tags": safe_json_arr(tags_json.as_deref()),
        "pinned": pinned != 0,
        "status": status,
        "publishedAt": published_at,
        "createdAt": created_at,
        "updatedAt": updated_at,
    })
}

/// 解析公告写入字段（创建/更新共用）：校验标题非空，category/status 收敛到允许值，
/// tags 收 JSON 数组、published_at 缺省取当前时间。
fn parse_announcement_input(
    body: &Value,
) -> AppResult<(String, String, String, String, String, i64, String, String)> {
    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if title.is_empty() {
        return Err(AppError::bad_request("公告标题不能为空"));
    }
    let category = match body
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("activity")
        .trim()
    {
        "maintenance" => "maintenance",
        _ => "activity",
    }
    .to_string();
    let summary = body
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let content = body
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let tags: Vec<String> = body
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into());
    let pinned = i64::from(
        body.get("pinned")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    );
    let status = match body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("published")
    {
        "draft" => "draft",
        _ => "published",
    }
    .to_string();
    let published_at = body
        .get("publishedAt")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(now_iso);
    Ok((
        category,
        title,
        summary,
        content,
        tags_json,
        pinned,
        status,
        published_at,
    ))
}

// 公开：已发布公告列表（置顶优先、发布时间倒序）
async fn list_announcements(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<AnnouncementRow> = sqlx::query_as(
        "SELECT id, category, title, summary, body, tags_json, pinned, status, published_at, created_at, updated_at \
         FROM announcements WHERE status='published' \
         ORDER BY pinned DESC, datetime(published_at) DESC, id DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows.into_iter().map(announcement_value).collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

// 后台：全部公告（含草稿）
async fn admin_list_announcements(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let rows: Vec<AnnouncementRow> = sqlx::query_as(
        "SELECT id, category, title, summary, body, tags_json, pinned, status, published_at, created_at, updated_at \
         FROM announcements \
         ORDER BY pinned DESC, datetime(published_at) DESC, id DESC",
    )
    .fetch_all(&state.pools.art)
    .await?;
    let data: Vec<Value> = rows.into_iter().map(announcement_value).collect();
    Ok(Json(json!({ "ok": true, "data": data })))
}

// 后台：创建公告
async fn admin_create_announcement(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let (category, title, summary, content, tags_json, pinned, status, published_at) =
        parse_announcement_input(&body)?;
    let now = now_iso();
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO announcements(category, title, summary, body, tags_json, pinned, status, published_at, created_at, updated_at) \
         VALUES(?,?,?,?,?,?,?,?,?,?) RETURNING id",
    )
    .bind(&category)
    .bind(&title)
    .bind(&summary)
    .bind(&content)
    .bind(&tags_json)
    .bind(pinned)
    .bind(&status)
    .bind(&published_at)
    .bind(&now)
    .bind(&now)
    .fetch_one(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true, "id": id })))
}

// 后台：更新公告
async fn admin_update_announcement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM announcements WHERE id=?")
        .bind(id)
        .fetch_optional(&state.pools.art)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("公告不存在"));
    }
    let (category, title, summary, content, tags_json, pinned, status, published_at) =
        parse_announcement_input(&body)?;
    sqlx::query(
        "UPDATE announcements SET category=?, title=?, summary=?, body=?, tags_json=?, pinned=?, status=?, published_at=?, updated_at=? \
         WHERE id=?",
    )
    .bind(&category)
    .bind(&title)
    .bind(&summary)
    .bind(&content)
    .bind(&tags_json)
    .bind(pinned)
    .bind(&status)
    .bind(&published_at)
    .bind(now_iso())
    .bind(id)
    .execute(&state.pools.art)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

// 后台：删除公告
async fn admin_delete_announcement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    sqlx::query("DELETE FROM announcements WHERE id=?")
        .bind(id)
        .execute(&state.pools.art)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn admin_points_ledger(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Read).await?;
    let uid = q.get("uid").map(|s| s.trim()).unwrap_or("");
    let rows: Vec<(
        Option<String>,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = if uid.is_empty() {
        sqlx::query_as(
            "SELECT pl.uid, CAST(NULLIF(TRIM(pl.points), '') AS INTEGER) AS points, pl.granted_at, pl.note, a.title AS artwork_title \
                 FROM points_ledger pl LEFT JOIN artworks a ON a.id=pl.artwork_id \
                 ORDER BY datetime(pl.granted_at) DESC",
        )
        .fetch_all(&state.pools.art)
        .await?
    } else {
        sqlx::query_as(
            "SELECT pl.uid, CAST(NULLIF(TRIM(pl.points), '') AS INTEGER) AS points, pl.granted_at, pl.note, a.title AS artwork_title \
                 FROM points_ledger pl LEFT JOIN artworks a ON a.id=pl.artwork_id \
                 WHERE pl.uid=? ORDER BY datetime(pl.granted_at) DESC",
        )
        .bind(uid)
        .fetch_all(&state.pools.art)
        .await?
    };
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
    // 管理员手工增减：标注 manual，计入历史获得积分（仅 redemption 兑换消耗被排除）。
    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) VALUES(?,?,?,?,?,?,?)",
    )
    .bind(uid)
    .bind(artwork_id)
    .bind(points)
    .bind(note)
    .bind("manual")
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
