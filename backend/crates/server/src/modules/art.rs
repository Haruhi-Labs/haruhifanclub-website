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
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use sha2::Sha256;
use std::path::{Component, Path as FsPath, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};
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
        .route("/creators/feed", get(creator_feed))
        .route("/visitors", post(record_visitor))
        .route("/announcements", get(list_announcements))
        .route("/artworks", get(list_artworks).post(create_artwork))
        .route("/artworks/{id}", get(get_artwork))
        .route("/artworks/{id}/related", get(related_artworks))
        .route(
            "/artworks/{id}/creator-neighbors",
            get(creator_neighbor_artworks),
        )
        .route(
            "/artworks/{id}/creator-timeline",
            get(creator_artwork_timeline),
        )
        .route("/artworks/{id}/favorite", post(toggle_artwork_favorite))
        .route("/creator-exhibits", get(creator_exhibits))
        .route("/recommendations", get(recommend_artworks))
        .route("/recommendation-events", post(record_recommendation_events))
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
        .route("/admin/points/penalize", post(admin_points_penalize))
        .merge(super::art_guild::router())
}

// ============================================================
// 匿名身份 Cookie（haruhi_anon）：HMAC-SHA256 签名
// ============================================================

const COOKIE_NAME: &str = "haruhi_anon";
const COOKIE_SIG: &str = "haruhi_anon_sig";
const VISITOR_SESSION_WINDOW_MINUTES: i64 = 10;
const CREATOR_FEED_CACHE_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const CREATOR_FEED_CACHE_LIMIT: usize = 256;
const RECOMMENDATION_FEED_CACHE_TTL: Duration = Duration::from_secs(4 * 60 * 60);
const RECOMMENDATION_FEED_CACHE_LIMIT: usize = 512;
const ARTWORK_DIMENSION_CACHE_LIMIT: usize = 8_192;
static ARTWORK_DIMENSION_CACHE: LazyLock<Mutex<std::collections::HashMap<PathBuf, (u32, u32)>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Default)]
pub struct CreatorFeedCache(Arc<Mutex<CreatorFeedCacheState>>);

#[derive(Default)]
struct CreatorFeedCacheState {
    feeds: std::collections::HashMap<String, CachedCreatorFeed>,
}

struct CachedCreatorFeed {
    created_at: Instant,
    uids: Vec<String>,
}

impl CreatorFeedCache {
    fn cached_random_order(
        &self,
        requested_feed_id: Option<&str>,
        available_uids: &[String],
    ) -> (String, Vec<String>) {
        let now = Instant::now();
        let mut cache = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .feeds
            .retain(|_, feed| now.duration_since(feed.created_at) < CREATOR_FEED_CACHE_TTL);

        if let Some(feed_id) = requested_feed_id.filter(|value| !value.trim().is_empty()) {
            if let Some(feed) = cache.feeds.get(feed_id) {
                return (feed_id.to_string(), feed.uids.clone());
            }
        }

        if cache.feeds.len() >= CREATOR_FEED_CACHE_LIMIT {
            if let Some(oldest) = cache
                .feeds
                .iter()
                .min_by_key(|(_, feed)| feed.created_at)
                .map(|(feed_id, _)| feed_id.clone())
            {
                cache.feeds.remove(&oldest);
            }
        }

        let feed_id = uuid::Uuid::new_v4().to_string();
        let mut uids = available_uids.to_vec();
        uids.shuffle(&mut rand::thread_rng());
        cache.feeds.insert(
            feed_id.clone(),
            CachedCreatorFeed {
                created_at: now,
                uids: uids.clone(),
            },
        );
        (feed_id, uids)
    }
}

#[derive(Clone, Default)]
pub struct RecommendationFeedCache(Arc<Mutex<RecommendationFeedCacheState>>);

#[derive(Default)]
struct RecommendationFeedCacheState {
    feeds: std::collections::HashMap<String, CachedRecommendationFeed>,
}

struct CachedRecommendationFeed {
    touched_at: Instant,
    actor_key: String,
    filter_key: String,
    seen_ids: std::collections::HashSet<i64>,
}

impl RecommendationFeedCache {
    fn begin_batch(
        &self,
        requested_feed_id: Option<&str>,
        actor_key: &str,
        filter_key: &str,
    ) -> (String, std::collections::HashSet<i64>, bool) {
        let now = Instant::now();
        let mut cache = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .feeds
            .retain(|_, feed| now.duration_since(feed.touched_at) < RECOMMENDATION_FEED_CACHE_TTL);

        let requested_feed_id = requested_feed_id.filter(|value| !value.trim().is_empty());
        if let Some(feed_id) = requested_feed_id {
            if let Some(feed) = cache.feeds.get_mut(feed_id) {
                if feed.actor_key == actor_key && feed.filter_key == filter_key {
                    feed.touched_at = now;
                    return (feed_id.to_string(), feed.seen_ids.clone(), false);
                }
            }
        }

        if cache.feeds.len() >= RECOMMENDATION_FEED_CACHE_LIMIT {
            if let Some(oldest) = cache
                .feeds
                .iter()
                .min_by_key(|(_, feed)| feed.touched_at)
                .map(|(feed_id, _)| feed_id.clone())
            {
                cache.feeds.remove(&oldest);
            }
        }

        let feed_id = uuid::Uuid::new_v4().to_string();
        cache.feeds.insert(
            feed_id.clone(),
            CachedRecommendationFeed {
                touched_at: now,
                actor_key: actor_key.to_string(),
                filter_key: filter_key.to_string(),
                seen_ids: std::collections::HashSet::new(),
            },
        );
        (
            feed_id,
            std::collections::HashSet::new(),
            requested_feed_id.is_some(),
        )
    }

    fn record_batch(&self, feed_id: &str, artwork_ids: impl IntoIterator<Item = i64>) {
        let mut cache = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(feed) = cache.feeds.get_mut(feed_id) {
            feed.touched_at = Instant::now();
            feed.seen_ids.extend(artwork_ids);
        }
    }
}

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
#[derive(Clone, Default, sqlx::FromRow)]
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
    exhibit_enabled: Option<i64>,
}

#[derive(Clone, Copy, Default)]
struct PopularityStats {
    views: i64,
    likes: i64,
    comments: i64,
    score: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PopularityRange {
    Week,
    Year,
    History,
}

impl PopularityRange {
    fn parse(raw: Option<&str>) -> Self {
        match raw.unwrap_or("history") {
            "week" => Self::Week,
            "year" => Self::Year,
            _ => Self::History,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Year => "year",
            Self::History => "history",
        }
    }
}

/// 有效浏览提供覆盖规模，点赞/评论按转化率放大覆盖规模。
/// Wilson 下界抑制小样本偶然高转化；保守曝光基数兼容启用浏览统计前的历史互动。
fn popularity_score(views: i64, likes: i64, comments: i64) -> f64 {
    let views = views.max(0) as f64;
    let likes = likes.max(0) as f64;
    let comments = comments.max(0) as f64;
    if views + likes + comments == 0.0 {
        return 0.0;
    }

    let audience = views.max(likes * 5.0).max(comments * 12.0).max(1.0);
    let reach = (1.0 + audience).ln();
    let like_confidence = wilson_lower_bound(likes.min(audience), audience);
    let comment_confidence = wilson_lower_bound(comments.min(audience), audience);
    100.0 * reach * (1.0 + 3.0 * like_confidence + 6.0 * comment_confidence)
}

fn wilson_lower_bound(successes: f64, trials: f64) -> f64 {
    if trials <= 0.0 {
        return 0.0;
    }
    let z = 1.96_f64;
    let z2 = z * z;
    let p = (successes / trials).clamp(0.0, 1.0);
    let denominator = 1.0 + z2 / trials;
    let centre = p + z2 / (2.0 * trials);
    let margin = z * ((p * (1.0 - p) + z2 / (4.0 * trials)) / trials).sqrt();
    ((centre - margin) / denominator).max(0.0)
}

#[cfg(test)]
mod popularity_score_tests {
    use super::{popularity_score, PopularityRange};

    #[test]
    fn empty_activity_has_no_popularity() {
        assert_eq!(popularity_score(0, 0, 0), 0.0);
    }

    #[test]
    fn stronger_intent_increases_score_at_equal_reach() {
        let viewed = popularity_score(200, 0, 0);
        let liked = popularity_score(200, 20, 0);
        let discussed = popularity_score(200, 20, 8);
        assert!(viewed < liked);
        assert!(liked < discussed);
    }

    #[test]
    fn confidence_and_reach_beat_a_tiny_perfect_sample() {
        let tiny = popularity_score(2, 2, 1);
        let established = popularity_score(200, 40, 10);
        assert!(tiny < established);
    }

    #[test]
    fn legacy_interactions_still_rank_before_view_collection_begins() {
        assert!(popularity_score(0, 12, 3) > 0.0);
    }

    #[test]
    fn unsupported_range_falls_back_to_history() {
        assert!(matches!(
            PopularityRange::parse(Some("unknown")),
            PopularityRange::History
        ));
    }
}

async fn popularity_stats(
    state: &AppState,
    range: PopularityRange,
) -> AppResult<std::collections::HashMap<i64, PopularityStats>> {
    let view_sql = match range {
        PopularityRange::Week => "SELECT artwork_id, COUNT(1) FROM artwork_views WHERE datetime(viewed_at) >= datetime('now', '-7 days') GROUP BY artwork_id",
        PopularityRange::Year => "SELECT artwork_id, COUNT(1) FROM artwork_views WHERE datetime(viewed_at) >= datetime('now', '-365 days') GROUP BY artwork_id",
        PopularityRange::History => "SELECT artwork_id, COUNT(1) FROM artwork_views GROUP BY artwork_id",
    };
    let like_sql = match range {
        PopularityRange::Week => "SELECT target_id, COUNT(DISTINCT anon_id) FROM likes_daily WHERE target_type='artwork' AND date(day) >= date('now', '-7 days') GROUP BY target_id",
        PopularityRange::Year => "SELECT target_id, COUNT(DISTINCT anon_id) FROM likes_daily WHERE target_type='artwork' AND date(day) >= date('now', '-365 days') GROUP BY target_id",
        PopularityRange::History => "SELECT target_id, COUNT(DISTINCT anon_id) FROM likes_daily WHERE target_type='artwork' GROUP BY target_id",
    };
    let comment_sql = match range {
        PopularityRange::Week => "SELECT artwork_id, COUNT(DISTINCT CASE WHEN TRIM(COALESCE(anon_id, ''))='' THEN 'comment:' || id ELSE anon_id END) FROM comments WHERE status='public' AND datetime(created_at) >= datetime('now', '-7 days') GROUP BY artwork_id",
        PopularityRange::Year => "SELECT artwork_id, COUNT(DISTINCT CASE WHEN TRIM(COALESCE(anon_id, ''))='' THEN 'comment:' || id ELSE anon_id END) FROM comments WHERE status='public' AND datetime(created_at) >= datetime('now', '-365 days') GROUP BY artwork_id",
        PopularityRange::History => "SELECT artwork_id, COUNT(DISTINCT CASE WHEN TRIM(COALESCE(anon_id, ''))='' THEN 'comment:' || id ELSE anon_id END) FROM comments WHERE status='public' GROUP BY artwork_id",
    };

    let (view_rows, like_rows, comment_rows) = tokio::try_join!(
        sqlx::query_as::<_, (i64, i64)>(view_sql).fetch_all(&state.pools.art),
        sqlx::query_as::<_, (i64, i64)>(like_sql).fetch_all(&state.pools.art),
        sqlx::query_as::<_, (i64, i64)>(comment_sql).fetch_all(&state.pools.art),
    )?;
    let mut out = std::collections::HashMap::<i64, PopularityStats>::new();
    for (id, views) in view_rows {
        out.entry(id).or_default().views = views;
    }
    for (id, likes) in like_rows {
        out.entry(id).or_default().likes = likes;
    }
    for (id, comments) in comment_rows {
        out.entry(id).or_default().comments = comments;
    }
    for stats in out.values_mut() {
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }
    Ok(out)
}

async fn record_artwork_view(
    state: &AppState,
    user_id: Option<i64>,
    anon_id: &str,
    artwork_id: i64,
) {
    let actor_key = user_id
        .map(|id| format!("user:{id}"))
        .unwrap_or_else(|| format!("anon:{anon_id}"));
    let now = chrono::Utc::now();
    let bucket = now.timestamp() / (30 * 60);
    if let Err(error) = sqlx::query(
        "INSERT OR IGNORE INTO artwork_views(artwork_id, actor_key, view_bucket, viewed_at) VALUES(?,?,?,?)",
    )
    .bind(artwork_id)
    .bind(actor_key)
    .bind(bucket)
    .bind(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
    .execute(&state.pools.art)
    .await
    {
        tracing::warn!(?error, artwork_id, "记录画廊有效浏览失败");
    }
}

const SELECT_ART: &str = "SELECT a.id, a.title, a.description, a.uploader_name, a.uploader_uid, \
    a.source_type, a.content_type, a.tags_json, a.tags_norm, a.origin_url, a.file_path, \
    a.file_path_original, a.status, a.review_note, a.reviewed_at, a.created_at, a.licenses_json, \
    COALESCE(CAST(NULLIF(TRIM(a.like_total), '') AS INTEGER), 0) AS like_total, a.images_json, a.ai_reason, c.avatar_url AS uploader_avatar, a.exhibit_enabled \
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
                let mut image = json!({ "image_url": image_url, "original_url": original_url });
                if let Some(obj) = image.as_object_mut() {
                    if let Some(width) = img.get("width").and_then(Value::as_u64) {
                        obj.insert("width".into(), json!(width));
                    }
                    if let Some(height) = img.get("height").and_then(Value::as_u64) {
                        obj.insert("height".into(), json!(height));
                    }
                }
                image
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
    let first_image_width = images
        .first()
        .and_then(|value| value.get("width"))
        .and_then(Value::as_u64);
    let first_image_height = images
        .first()
        .and_then(|value| value.get("height"))
        .and_then(Value::as_u64);

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
        "image_width": first_image_width,
        "image_height": first_image_height,
        "images": images,
        "ai_reason": r.ai_reason.clone().unwrap_or_default(),
        "exhibit_enabled": r.exhibit_enabled.map(|value| value != 0),
    })
}

fn artwork_metadata_dimensions(row: &ArtRow) -> Option<(u32, u32)> {
    let first = safe_json_arr(row.images_json.as_deref())
        .into_iter()
        .next()?;
    let width = u32::try_from(first.get("width")?.as_u64()?).ok()?;
    let height = u32::try_from(first.get("height")?.as_u64()?).ok()?;
    (width > 0 && height > 0).then_some((width, height))
}

fn artwork_display_path(row: &ArtRow) -> Option<String> {
    let image_path = safe_json_arr(row.images_json.as_deref())
        .into_iter()
        .next()
        .and_then(|image| image.get("path").and_then(Value::as_str).map(str::to_owned));
    let relative = image_path.or_else(|| row.file_path.clone())?;
    let relative = relative.trim();
    if relative.is_empty()
        || !FsPath::new(relative)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return None;
    }
    Some(relative.to_string())
}

async fn artwork_dimensions(
    state: &AppState,
    rows: &[ArtRow],
) -> std::collections::HashMap<i64, (u32, u32)> {
    let mut dimensions = std::collections::HashMap::new();
    let mut pending = Vec::new();
    for row in rows {
        if let Some(size) = artwork_metadata_dimensions(row) {
            dimensions.insert(row.id, size);
        } else if let Some(relative) = artwork_display_path(row) {
            pending.push((row.id, relative));
        }
    }
    if pending.is_empty() {
        return dimensions;
    }

    let uploads_dir = state.cfg.uploads_dir.clone();
    let measured = tokio::task::spawn_blocking(move || {
        let mut found = std::collections::HashMap::new();
        for (id, relative) in pending {
            let path = uploads_dir.join(relative);
            let cached = ARTWORK_DIMENSION_CACHE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(&path)
                .copied();
            let size = cached.or_else(|| {
                let measured = haruhi_media::image_dimensions(&path).ok()?;
                let mut cache = ARTWORK_DIMENSION_CACHE
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if cache.len() >= ARTWORK_DIMENSION_CACHE_LIMIT {
                    cache.clear();
                }
                cache.insert(path, measured);
                Some(measured)
            });
            if let Some(size) = size {
                found.insert(id, size);
            }
        }
        found
    })
    .await;
    match measured {
        Ok(measured) => dimensions.extend(measured),
        Err(error) => tracing::warn!(?error, "读取作品图片尺寸任务失败"),
    }
    dimensions
}

fn insert_artwork_dimensions(value: &mut Value, dimensions: Option<(u32, u32)>) {
    let (Some((width, height)), Some(obj)) = (dimensions, value.as_object_mut()) else {
        return;
    };
    obj.insert("image_width".into(), json!(width));
    obj.insert("image_height".into(), json!(height));
    if let Some(first) = obj
        .get_mut("images")
        .and_then(Value::as_array_mut)
        .and_then(|images| images.first_mut())
        .and_then(Value::as_object_mut)
    {
        first.insert("width".into(), json!(width));
        first.insert("height".into(), json!(height));
    }
}

#[cfg(test)]
mod artwork_dimension_tests {
    use super::{
        artwork_display_path, artwork_metadata_dimensions, insert_artwork_dimensions, ArtRow,
    };
    use serde_json::json;

    #[test]
    fn reads_dimensions_from_image_metadata() {
        let row = ArtRow {
            images_json: Some(
                r#"[{"path":"art/2026-07/work.webp","width":1200,"height":1800}]"#.into(),
            ),
            ..ArtRow::default()
        };
        assert_eq!(artwork_metadata_dimensions(&row), Some((1200, 1800)));
        assert_eq!(
            artwork_display_path(&row).as_deref(),
            Some("art/2026-07/work.webp")
        );
    }

    #[test]
    fn rejects_paths_outside_uploads_root() {
        let row = ArtRow {
            file_path: Some("art/../../secret.webp".into()),
            ..ArtRow::default()
        };
        assert_eq!(artwork_display_path(&row), None);
    }

    #[test]
    fn inserts_dimensions_into_cover_and_first_image() {
        let mut value = json!({ "images": [{ "image_url": "uploads/art/work.webp" }] });
        insert_artwork_dimensions(&mut value, Some((960, 1440)));
        assert_eq!(value["image_width"], 960);
        assert_eq!(value["image_height"], 1440);
        assert_eq!(value["images"][0]["width"], 960);
        assert_eq!(value["images"][0]["height"], 1440);
    }
}

fn insert_popularity(value: &mut Value, stats: PopularityStats, range: PopularityRange) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    let score = stats.score.round() as i64;
    obj.insert("popularity_score".into(), json!(score));
    obj.insert(
        "popularity".into(),
        json!({
            "score": score,
            "views": stats.views,
            "likes": stats.likes,
            "comments": stats.comments,
            "range": range.as_str(),
        }),
    );
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
    let (names, dimensions) = tokio::join!(
        member_display_names(&state.pools.core, &uids),
        artwork_dimensions(state, rows),
    );
    rows.iter()
        .map(|r| {
            let mut v = map_artwork_row(r);
            insert_artwork_dimensions(&mut v, dimensions.get(&r.id).copied());
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
    let names = member_display_names(&state.pools.core, std::slice::from_ref(&uid)).await;
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
    let names = member_display_names(&state.pools.core, std::slice::from_ref(&uid)).await;
    let name = names.get(&uid).cloned();
    let creator = row.as_ref().map(
        |(uid, avatar_url)| json!({ "uid": uid, "name": name.clone(), "avatar_url": avatar_url }),
    );
    Ok(Json(
        json!({ "ok": true, "exists": row.is_some(), "creator": creator }),
    ))
}

#[derive(serde::Deserialize)]
struct CreatorFeedQuery {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    #[serde(rename = "feedId", alias = "feed_id")]
    feed_id: Option<String>,
}

fn creator_feed_recommendation_score(
    row: &ArtRow,
    stats: PopularityStats,
    max_quality_raw: f64,
    entropy: &str,
) -> f64 {
    let quality = ((stats.score + 1.0).ln() / max_quality_raw).clamp(0.0, 1.0);
    let age_days =
        recommendation_age_days(row.reviewed_at.as_deref().or(row.created_at.as_deref()));
    let freshness = (-age_days / 120.0).exp().clamp(0.0, 1.0);
    let exploration = recommendation_noise(entropy, row.id);
    quality * 0.40 + freshness * 0.30 + exploration * 0.30
}

// 4.4 创作者信息流：服务端生成并缓存随机作者顺序，客户端用 feedId 稳定分页；
// 每位作者的作品使用 hybrid-v1 冷启动推荐权重（质量/新鲜度/探索）选取至多四张。
const CREATOR_FEED_WORK_LIMIT: usize = 4;

async fn creator_feed(
    State(state): State<AppState>,
    Query(q): Query<CreatorFeedQuery>,
) -> AppResult<Json<Value>> {
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.source_type='personal' \
         AND TRIM(COALESCE(a.uploader_uid, '')) <> '' ORDER BY a.id DESC"
    ))
    .fetch_all(&state.pools.art)
    .await?;

    let mut grouped = std::collections::BTreeMap::<String, Vec<&ArtRow>>::new();
    for row in &rows {
        if let Some(uid) = row
            .uploader_uid
            .as_deref()
            .map(str::trim)
            .filter(|uid| !uid.is_empty())
        {
            grouped.entry(uid.to_string()).or_default().push(row);
        }
    }

    let available_uids: Vec<String> = grouped.keys().cloned().collect();
    let requested_feed_id = q
        .feed_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty());
    let (feed_id, cached_order) = state
        .creator_feed
        .cached_random_order(requested_feed_id, &available_uids);
    let available: std::collections::HashSet<&str> =
        available_uids.iter().map(String::as_str).collect();
    let ordered_uids: Vec<String> = cached_order
        .into_iter()
        .filter(|uid| available.contains(uid.as_str()))
        .collect();

    let page_size = q.page_size.unwrap_or(6).clamp(1, 24);
    let cache_reset = requested_feed_id
        .map(|requested| requested != feed_id)
        .unwrap_or(false);
    let page = if cache_reset {
        1
    } else {
        q.page.unwrap_or(1).max(1)
    };
    let offset = ((page - 1) * page_size) as usize;
    let page_uids: Vec<String> = ordered_uids
        .iter()
        .skip(offset)
        .take(page_size as usize)
        .cloned()
        .collect();

    let mut stats_by_id = popularity_stats(&state, PopularityRange::History).await?;
    for row in &rows {
        let stats = stats_by_id.entry(row.id).or_default();
        stats.likes = stats.likes.max(row.like_total.max(0));
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }
    let max_quality_raw = rows
        .iter()
        .map(|row| {
            let score = stats_by_id
                .get(&row.id)
                .map(|stats| stats.score)
                .unwrap_or(0.0);
            (score + 1.0).ln()
        })
        .fold(1.0_f64, f64::max);
    let page_uid_set: std::collections::HashSet<&str> =
        page_uids.iter().map(String::as_str).collect();
    let page_rows: Vec<ArtRow> = rows
        .iter()
        .filter(|row| {
            row.uploader_uid
                .as_deref()
                .is_some_and(|uid| page_uid_set.contains(uid))
        })
        .cloned()
        .collect();
    let (names, dimensions) = tokio::join!(
        member_display_names(&state.pools.core, &page_uids),
        artwork_dimensions(&state, &page_rows),
    );

    let mut data = Vec::with_capacity(page_uids.len());
    for (creator_position, uid) in page_uids.iter().enumerate() {
        let Some(creator_rows) = grouped.get(uid) else {
            continue;
        };
        let entropy = format!("creator-feed:{feed_id}:{uid}");
        let mut recommended = creator_rows.clone();
        recommended.sort_by(|a, b| {
            let a_stats = stats_by_id.get(&a.id).copied().unwrap_or_default();
            let b_stats = stats_by_id.get(&b.id).copied().unwrap_or_default();
            let a_score = creator_feed_recommendation_score(a, a_stats, max_quality_raw, &entropy);
            let b_score = creator_feed_recommendation_score(b, b_stats, max_quality_raw, &entropy);
            b_score.total_cmp(&a_score).then_with(|| b.id.cmp(&a.id))
        });
        recommended.truncate(CREATOR_FEED_WORK_LIMIT);

        let creator_name = names
            .get(uid)
            .cloned()
            .or_else(|| {
                creator_rows
                    .iter()
                    .find_map(|row| row.uploader_name.clone())
            })
            .unwrap_or_else(|| uid.clone());
        let avatar = creator_rows
            .iter()
            .find_map(|row| row.uploader_avatar.clone())
            .unwrap_or_default();
        let items: Vec<Value> = recommended
            .iter()
            .enumerate()
            .map(|(artwork_position, row)| {
                let mut value = map_artwork_row(row);
                insert_artwork_dimensions(&mut value, dimensions.get(&row.id).copied());
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("uploader_display_name".into(), json!(creator_name.clone()));
                    obj.insert(
                        "recommendation".into(),
                        json!({
                            "batch_id": feed_id,
                            "position": offset * CREATOR_FEED_WORK_LIMIT
                                + creator_position * CREATOR_FEED_WORK_LIMIT
                                + artwork_position,
                        }),
                    );
                }
                insert_popularity(
                    &mut value,
                    stats_by_id.get(&row.id).copied().unwrap_or_default(),
                    PopularityRange::History,
                );
                value
            })
            .collect();
        data.push(json!({
            "uid": uid,
            "name": creator_name,
            "avatar": avatar,
            "totalWorks": creator_rows.len(),
            "items": items,
        }));
    }

    let total = ordered_uids.len();
    Ok(Json(json!({
        "ok": true,
        "data": data,
        "total": total,
        "page": page,
        "pageSize": page_size,
        "hasMore": offset.saturating_add(data.len()) < total,
        "feedId": feed_id,
        "cacheReset": cache_reset,
        "algorithmVersion": RECOMMENDATION_VERSION,
    })))
}

// 4.5 创作者展位：未配置过的作者默认展示历史人气最高的三件已通过个人作品；
// 一旦作者在个人中心调整过，则严格使用其显式选择。
async fn creator_exhibits(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.source_type='personal' \
         AND TRIM(COALESCE(a.uploader_uid, '')) <> '' ORDER BY a.id DESC"
    ))
    .fetch_all(&state.pools.art)
    .await?;
    let mut stats_by_id = popularity_stats(&state, PopularityRange::History).await?;
    for row in &rows {
        let stats = stats_by_id.entry(row.id).or_default();
        stats.likes = stats.likes.max(row.like_total.max(0));
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }

    let uids: Vec<String> = rows
        .iter()
        .filter_map(|row| row.uploader_uid.clone())
        .collect();
    let names = member_display_names(&state.pools.core, &uids).await;
    let mut grouped = std::collections::BTreeMap::<String, Vec<&ArtRow>>::new();
    for row in &rows {
        if let Some(uid) = row
            .uploader_uid
            .as_deref()
            .filter(|uid| !uid.trim().is_empty())
        {
            grouped.entry(uid.to_string()).or_default().push(row);
        }
    }

    let mut data = Vec::new();
    for (uid, creator_rows) in grouped {
        let configured = creator_rows.iter().any(|row| row.exhibit_enabled.is_some());
        let mut selected: Vec<&ArtRow> = if configured {
            creator_rows
                .iter()
                .copied()
                .filter(|row| row.exhibit_enabled == Some(1))
                .collect()
        } else {
            let mut ranked = creator_rows.clone();
            ranked.sort_by(|a, b| {
                let a_stats = stats_by_id.get(&a.id).copied().unwrap_or_default();
                let b_stats = stats_by_id.get(&b.id).copied().unwrap_or_default();
                b_stats
                    .score
                    .total_cmp(&a_stats.score)
                    .then_with(|| b.like_total.cmp(&a.like_total))
                    .then_with(|| b.id.cmp(&a.id))
            });
            ranked.into_iter().take(3).collect()
        };
        if selected.is_empty() {
            continue;
        }
        selected.sort_by(|a, b| {
            let a_stats = stats_by_id.get(&a.id).copied().unwrap_or_default();
            let b_stats = stats_by_id.get(&b.id).copied().unwrap_or_default();
            b_stats
                .score
                .total_cmp(&a_stats.score)
                .then_with(|| b.id.cmp(&a.id))
        });
        let creator_name = names
            .get(&uid)
            .cloned()
            .or_else(|| selected.iter().find_map(|row| row.uploader_name.clone()))
            .unwrap_or_else(|| uid.clone());
        let avatar = selected
            .iter()
            .find_map(|row| row.uploader_avatar.clone())
            .unwrap_or_default();
        let items: Vec<Value> = selected
            .iter()
            .map(|row| {
                let mut value = map_artwork_row(row);
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("uploader_display_name".into(), json!(creator_name.clone()));
                    obj.insert("exhibit_enabled".into(), json!(true));
                }
                insert_popularity(
                    &mut value,
                    stats_by_id.get(&row.id).copied().unwrap_or_default(),
                    PopularityRange::History,
                );
                value
            })
            .collect();
        data.push(json!({
            "uid": uid,
            "name": creator_name,
            "avatar": avatar,
            "items": items,
        }));
    }

    Ok(Json(
        json!({ "ok": true, "total": data.len(), "data": data }),
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
    let popularity_range = PopularityRange::parse(getq("range"));
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

    if sort == "popular" || sort == "likes" {
        order_by = "ORDER BY (\
            COALESCE(CAST(NULLIF(TRIM(a.like_total), '') AS INTEGER), 0) + \
            8 * (SELECT COUNT(1) FROM comments cm \
                 WHERE cm.artwork_id = a.id AND cm.status = 'public')\
        ) DESC, datetime(COALESCE(a.reviewed_at, a.created_at)) DESC, a.id DESC"
            .into();
    } else if sort == "time" {
        order_by =
            "ORDER BY datetime(COALESCE(a.reviewed_at, a.created_at)) DESC, a.id DESC".into();
    } else if sort == "recommended" || sort == "random" {
        let seed = match getq("seed") {
            Some(s) if !s.trim().is_empty() => clamp_int(Some(s), 0, 2147483647, 0),
            _ => rand_int(0, 2147483647),
        };
        seed_used = Some(seed);
    }

    let mut popularity_by_id = std::collections::HashMap::<i64, PopularityStats>::new();
    let rows: Vec<ArtRow> = if sort == "popular" || sort == "likes" {
        let list_sql = format!("{SELECT_ART} {where_sql}");
        let mut list_q = sqlx::query_as::<_, ArtRow>(&list_sql);
        for p in &params {
            list_q = list_q.bind(p);
        }
        let mut ranked_rows = list_q.fetch_all(&state.pools.art).await?;
        popularity_by_id = popularity_stats(&state, popularity_range).await?;
        for row in &ranked_rows {
            let stats = popularity_by_id.entry(row.id).or_default();
            if popularity_range == PopularityRange::History {
                stats.likes = stats.likes.max(row.like_total.max(0));
            }
            stats.score = popularity_score(stats.views, stats.likes, stats.comments);
        }
        ranked_rows.sort_by(|a, b| {
            let a_stats = popularity_by_id.get(&a.id).copied().unwrap_or_default();
            let b_stats = popularity_by_id.get(&b.id).copied().unwrap_or_default();
            b_stats
                .score
                .total_cmp(&a_stats.score)
                .then_with(|| b_stats.views.cmp(&a_stats.views))
                .then_with(|| b_stats.likes.cmp(&a_stats.likes))
                .then_with(|| b_stats.comments.cmp(&a_stats.comments))
                .then_with(|| {
                    b.reviewed_at
                        .as_deref()
                        .or(b.created_at.as_deref())
                        .unwrap_or("")
                        .cmp(
                            a.reviewed_at
                                .as_deref()
                                .or(a.created_at.as_deref())
                                .unwrap_or(""),
                        )
                })
                .then_with(|| b.id.cmp(&a.id))
        });
        ranked_rows
            .into_iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .collect()
    } else if let Some(seed) = seed_used {
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
    let display_popularity_range = if sort == "popular" || sort == "likes" {
        popularity_range
    } else {
        PopularityRange::History
    };
    if sort != "popular" && sort != "likes" {
        popularity_by_id = popularity_stats(&state, PopularityRange::History).await?;
        for row in &rows {
            let stats = popularity_by_id.entry(row.id).or_default();
            stats.likes = stats.likes.max(row.like_total.max(0));
            stats.score = popularity_score(stats.views, stats.likes, stats.comments);
        }
    }
    // 批量取作者公会徽章，避免逐行 N+1（一次 IN 查询、不建档）。
    let uids: Vec<String> = rows.iter().filter_map(|r| r.uploader_uid.clone()).collect();
    let (guild_map, names, dimensions) = tokio::join!(
        super::art_guild::guild_summaries_for_uids(&state, &uids),
        member_display_names(&state.pools.core, &uids),
        artwork_dimensions(&state, &rows),
    );
    let data: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut value = map_artwork_row(row);
            insert_artwork_dimensions(&mut value, dimensions.get(&row.id).copied());
            let uid = row.uploader_uid.as_deref().unwrap_or("").trim();
            if let Some(obj) = value.as_object_mut() {
                if !uid.is_empty() {
                    if let Some(summary) = guild_map.get(uid) {
                        obj.insert("guild".into(), summary.clone());
                    }
                    if let Some(n) = names.get(uid) {
                        obj.insert("uploader_display_name".into(), json!(n));
                    }
                }
            }
            insert_popularity(
                &mut value,
                popularity_by_id.get(&row.id).copied().unwrap_or_default(),
                display_popularity_range,
            );
            value
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "data": data,
        "total": total,
    })))
}

// 5.5 个性化推荐：内容画像 + 质量/新鲜度 + 探索，多路召回后做去重和多样性约束。
const RECOMMENDATION_VERSION: &str = "hybrid-v1";
const RECOMMENDATION_HISTORY_DAYS: i64 = 180;
const RECOMMENDATION_DAILY_EVENT_LIMIT: i64 = 2_000;

#[derive(sqlx::FromRow)]
struct RecommendationHistoryRow {
    artwork_id: i64,
    event_type: String,
    dwell_ms: Option<i64>,
    batch_id: Option<String>,
    created_at: String,
    tags_json: Option<String>,
    content_type: Option<String>,
    source_type: Option<String>,
    uploader_uid: Option<String>,
}

#[derive(serde::Deserialize)]
struct RecommendationQuery {
    limit: Option<i64>,
    #[serde(default, alias = "feedId")]
    feed_id: Option<String>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    source_type: Option<String>,
}

#[derive(serde::Deserialize)]
struct RecommendationEventsBody {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    events: Vec<RecommendationEventInput>,
}

#[derive(serde::Deserialize)]
struct RecommendationEventInput {
    artwork_id: i64,
    event_type: String,
    #[serde(default)]
    batch_id: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    position: Option<i64>,
    #[serde(default)]
    dwell_ms: Option<i64>,
}

struct RecommendationCandidate {
    row_index: usize,
    quality: f64,
    freshness: f64,
    exploration: f64,
    blended: f64,
    creator: String,
    primary_tag: String,
}

fn recommendation_tags(raw: Option<&str>) -> Vec<String> {
    safe_json_arr(raw)
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.trim().to_lowercase()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn insert_related_text_segment(tokens: &mut std::collections::HashSet<String>, segment: &[char]) {
    if segment.is_empty() {
        return;
    }
    if segment.iter().all(|ch| ch.is_ascii_alphanumeric()) {
        if segment.len() >= 2 {
            tokens.insert(segment.iter().collect::<String>());
        }
        return;
    }
    if (2..=12).contains(&segment.len()) {
        tokens.insert(segment.iter().collect::<String>());
    }
    for pair in segment.windows(2) {
        let token = pair.iter().collect::<String>();
        if !matches!(token.as_str(), "作品" | "一个" | "这个" | "的是" | "什么") {
            tokens.insert(token);
        }
    }
}

fn related_text_tokens(
    title: Option<&str>,
    description: Option<&str>,
) -> std::collections::HashSet<String> {
    let text = format!(
        "{} {}",
        title.unwrap_or_default(),
        description.unwrap_or_default()
    )
    .to_lowercase();
    let mut tokens = std::collections::HashSet::new();
    let mut segment = Vec::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            segment.push(ch);
        } else {
            insert_related_text_segment(&mut tokens, &segment);
            segment.clear();
        }
    }
    insert_related_text_segment(&mut tokens, &segment);
    tokens
}

fn recommendation_bounded_text(raw: &str, max_chars: usize) -> String {
    safe_text(Some(raw)).chars().take(max_chars).collect()
}

fn recommendation_age_days(raw: Option<&str>) -> f64 {
    let Some(raw) = raw.filter(|s| !s.trim().is_empty()) else {
        return 365.0;
    };
    let parsed = chrono::DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc())
        });
    parsed
        .map(|dt| (chrono::Utc::now() - dt).num_seconds().max(0) as f64 / 86_400.0)
        .unwrap_or(365.0)
}

fn recommendation_event_weight(event_type: &str, dwell_ms: Option<i64>) -> Option<f64> {
    match event_type {
        "open" => Some(1.0),
        "like" => Some(6.0),
        "favorite" => Some(7.0),
        "comment" => Some(8.0),
        "hide" => Some(-8.0),
        "dwell" => match dwell_ms.unwrap_or(0) {
            0..=2_999 => Some(-0.5),
            3_000..=9_999 => Some(0.4),
            10_000..=29_999 => Some(1.5),
            30_000..=89_999 => Some(2.5),
            _ => Some(3.5),
        },
        _ => None,
    }
}

fn recommendation_event_cap(event_type: &str) -> usize {
    match event_type {
        "like" | "favorite" | "hide" => 1,
        "comment" => 2,
        "open" | "dwell" => 3,
        _ => 0,
    }
}

fn add_profile_weight(
    profile: &mut std::collections::HashMap<String, f64>,
    key: String,
    weight: f64,
) {
    *profile.entry(key).or_insert(0.0) += weight;
}

fn recommendation_noise(entropy: &str, artwork_id: i64) -> f64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entropy.hash(&mut hasher);
    artwork_id.hash(&mut hasher);
    (hasher.finish() % 1_000_000) as f64 / 999_999.0
}

fn recommendation_channel_score(candidate: &RecommendationCandidate, channel: &str) -> f64 {
    match channel {
        "quality" => {
            candidate.quality * 0.65 + candidate.freshness * 0.20 + candidate.exploration * 0.15
        }
        "fresh" => {
            candidate.freshness * 0.70 + candidate.quality * 0.20 + candidate.exploration * 0.10
        }
        "explore" => {
            candidate.exploration * 0.75 + candidate.quality * 0.15 + candidate.freshness * 0.10
        }
        _ => candidate.blended,
    }
}

async fn recommendation_history(
    state: &AppState,
    user_id: Option<i64>,
    anon_id: &str,
) -> AppResult<Vec<RecommendationHistoryRow>> {
    Ok(sqlx::query_as::<_, RecommendationHistoryRow>(
        "SELECT re.artwork_id, re.event_type, re.dwell_ms, re.batch_id, re.created_at, \
                a.tags_json, a.content_type, a.source_type, a.uploader_uid \
         FROM recommendation_events re \
         JOIN artworks a ON a.id=re.artwork_id \
         WHERE (re.user_id=? OR re.anon_id=?) \
           AND datetime(re.created_at) >= datetime('now', ?) \
         ORDER BY datetime(re.created_at) DESC, re.id DESC LIMIT 5000",
    )
    .bind(user_id)
    .bind(anon_id)
    .bind(format!("-{RECOMMENDATION_HISTORY_DAYS} days"))
    .fetch_all(&state.pools.art)
    .await?)
}

async fn maybe_cleanup_recommendation_events(state: &AppState) {
    let now = now_iso();
    let claimed = sqlx::query(
        "UPDATE recommendation_maintenance SET last_cleanup_at=? \
         WHERE id=1 AND (last_cleanup_at IS NULL OR datetime(last_cleanup_at) < datetime('now', '-1 day'))",
    )
    .bind(&now)
    .execute(&state.pools.art)
    .await;
    let should_cleanup = claimed
        .as_ref()
        .map(|result| result.rows_affected() > 0)
        .unwrap_or(false);
    if should_cleanup {
        if let Err(error) = sqlx::query(
            "DELETE FROM recommendation_events \
             WHERE datetime(created_at) < datetime('now', '-400 days')",
        )
        .execute(&state.pools.art)
        .await
        {
            tracing::warn!(?error, "清理过期画廊推荐事件失败");
        }
    } else if let Err(error) = claimed {
        tracing::warn!(?error, "检查画廊推荐事件清理周期失败");
    }
}

async fn recommend_artworks(
    State(state): State<AppState>,
    Query(q): Query<RecommendationQuery>,
    user: Option<AuthUser>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let limit = q.limit.unwrap_or(8).clamp(4, 24) as usize;
    let (anon_id, set) = resolve_anon(&state.cfg.art_cookie_secret, &headers);
    let user_id = user.map(|u| u.id);
    let actor_entropy = user_id
        .map(|id| format!("user:{id}"))
        .unwrap_or_else(|| format!("anon:{anon_id}"));
    let batch_id = uuid::Uuid::new_v4().to_string();

    let content_type = q
        .content_type
        .as_deref()
        .filter(|value| matches!(*value, "haruhi" | "other"))
        .map(str::to_string);
    let source_type = q
        .source_type
        .as_deref()
        .filter(|value| matches!(*value, "personal" | "network"))
        .map(str::to_string);
    let filter_key = format!(
        "content:{}:source:{}",
        content_type.as_deref().unwrap_or("all"),
        source_type.as_deref().unwrap_or("all")
    );
    let requested_feed_id = q
        .feed_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let (feed_id, feed_seen_ids, cache_reset) =
        state
            .recommendation_feed
            .begin_batch(requested_feed_id, &actor_entropy, &filter_key);

    let mut where_sql = String::from("WHERE a.status='approved'");
    if content_type.is_some() {
        where_sql.push_str(" AND a.content_type=?");
    }
    if source_type.is_some() {
        where_sql.push_str(" AND a.source_type=?");
    }
    let rows_sql = format!("{SELECT_ART} {where_sql} ORDER BY a.id DESC LIMIT 5000");
    let mut rows_query = sqlx::query_as::<_, ArtRow>(&rows_sql);
    if let Some(value) = content_type.as_deref() {
        rows_query = rows_query.bind(value);
    }
    if let Some(value) = source_type.as_deref() {
        rows_query = rows_query.bind(value);
    }
    let rows: Vec<ArtRow> = rows_query.fetch_all(&state.pools.art).await?;
    if rows.is_empty() {
        return Ok(json_with_cookie(
            json!({
                "ok": true, "data": [], "total": 0, "batchId": batch_id,
                "feedId": feed_id, "hasMore": false, "cacheReset": cache_reset,
                "algorithmVersion": RECOMMENDATION_VERSION, "personalized": false
            }),
            set,
        ));
    }

    let mut quality_stats = popularity_stats(&state, PopularityRange::History).await?;
    for row in &rows {
        let stats = quality_stats.entry(row.id).or_default();
        stats.likes = stats.likes.max(row.like_total.max(0));
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }
    let history = recommendation_history(&state, user_id, &anon_id).await?;

    let mut profile: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut signal_counts: std::collections::HashMap<(i64, String), usize> =
        std::collections::HashMap::new();
    let mut learned_signals = 0usize;
    let mut recent_batch_ids = Vec::new();
    let mut recent_batch_seen = std::collections::HashSet::new();
    let mut last_seen_age: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();

    for event in &history {
        let age_days = recommendation_age_days(Some(&event.created_at));
        last_seen_age
            .entry(event.artwork_id)
            .and_modify(|age| *age = age.min(age_days))
            .or_insert(age_days);

        // 只有真正进入视口的作品才算近期已展示；推荐 GET 本身保持只读，
        // 避免公开接口被循环调用时制造无上限的数据库写入。
        if event.event_type == "impression" {
            if let Some(batch) = event.batch_id.as_ref() {
                if !recent_batch_ids.contains(batch) && recent_batch_ids.len() < 3 {
                    recent_batch_ids.push(batch.clone());
                }
                if recent_batch_ids.contains(batch) {
                    recent_batch_seen.insert(event.artwork_id);
                }
            }
        }

        let Some(base_weight) = recommendation_event_weight(&event.event_type, event.dwell_ms)
        else {
            continue;
        };
        let cap = recommendation_event_cap(&event.event_type);
        let count = signal_counts
            .entry((event.artwork_id, event.event_type.clone()))
            .or_insert(0);
        if *count >= cap {
            continue;
        }
        *count += 1;
        learned_signals += 1;

        let decay = 0.5_f64.powf(age_days / 45.0);
        let weight = base_weight * decay;
        let tags = recommendation_tags(event.tags_json.as_deref());
        let tag_scale = (tags.len().max(1) as f64).sqrt();
        for tag in tags {
            add_profile_weight(&mut profile, format!("tag:{tag}"), weight / tag_scale);
        }
        if let Some(uid) = event.uploader_uid.as_deref().filter(|s| !s.is_empty()) {
            add_profile_weight(&mut profile, format!("creator:{uid}"), weight * 0.65);
        }
        if let Some(content) = event.content_type.as_deref().filter(|s| !s.is_empty()) {
            add_profile_weight(&mut profile, format!("content:{content}"), weight * 0.35);
        }
        if let Some(source) = event.source_type.as_deref().filter(|s| !s.is_empty()) {
            add_profile_weight(&mut profile, format!("source:{source}"), weight * 0.25);
        }
    }

    let personalized = learned_signals >= 2;
    let max_quality_raw = rows
        .iter()
        .map(|row| {
            let score = quality_stats
                .get(&row.id)
                .map(|stats| stats.score)
                .unwrap_or(0.0);
            (score + 1.0).ln()
        })
        .fold(1.0_f64, f64::max);
    let entropy = format!("{actor_entropy}:{feed_id}:{batch_id}");
    let own_uid = user_id.map(crate::auth_routes::member_uid);
    let can_exclude_recent = rows
        .iter()
        .filter(|row| !feed_seen_ids.contains(&row.id) && !recent_batch_seen.contains(&row.id))
        .count()
        >= limit;

    let mut candidates = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        if feed_seen_ids.contains(&row.id) {
            continue;
        }
        if can_exclude_recent && recent_batch_seen.contains(&row.id) {
            continue;
        }
        let tags = recommendation_tags(row.tags_json.as_deref());
        let tag_affinity = if tags.is_empty() {
            0.0
        } else {
            tags.iter()
                .map(|tag| *profile.get(&format!("tag:{tag}")).unwrap_or(&0.0))
                .sum::<f64>()
                / (tags.len() as f64).sqrt()
        };
        let creator = row.uploader_uid.clone().unwrap_or_default();
        let creator_affinity = if creator.is_empty() {
            0.0
        } else {
            *profile.get(&format!("creator:{creator}")).unwrap_or(&0.0)
        };
        let content_affinity = row
            .content_type
            .as_deref()
            .and_then(|v| profile.get(&format!("content:{v}")))
            .copied()
            .unwrap_or(0.0);
        let source_affinity = row
            .source_type
            .as_deref()
            .and_then(|v| profile.get(&format!("source:{v}")))
            .copied()
            .unwrap_or(0.0);
        let affinity_raw = tag_affinity + creator_affinity + content_affinity + source_affinity;
        let affinity = (affinity_raw / 8.0).tanh();
        let quality_raw = quality_stats
            .get(&row.id)
            .map(|stats| stats.score)
            .unwrap_or(0.0);
        let quality = ((quality_raw + 1.0).ln() / max_quality_raw).clamp(0.0, 1.0);
        let age_days =
            recommendation_age_days(row.reviewed_at.as_deref().or(row.created_at.as_deref()));
        let freshness = (-age_days / 120.0).exp().clamp(0.0, 1.0);
        let exploration = recommendation_noise(&entropy, row.id);
        let seen_penalty = match last_seen_age.get(&row.id).copied() {
            Some(age) if age < 1.0 => 0.45,
            Some(age) if age < 7.0 => 0.25,
            Some(age) if age < 30.0 => 0.10,
            _ => 0.0,
        };
        let own_work_penalty = if own_uid.as_deref() == Some(creator.as_str()) {
            0.10
        } else {
            0.0
        };
        let blended = if personalized {
            affinity * 0.50 + quality * 0.20 + freshness * 0.12 + exploration * 0.18
        } else {
            quality * 0.40 + freshness * 0.30 + exploration * 0.30
        } - seen_penalty
            - own_work_penalty;

        candidates.push(RecommendationCandidate {
            row_index,
            quality,
            freshness,
            exploration,
            blended,
            creator,
            primary_tag: tags.first().cloned().unwrap_or_default(),
        });
    }

    let mut selected = Vec::with_capacity(limit);
    let mut selected_ids = std::collections::HashSet::new();
    let mut creator_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let channel_pattern = [
        "blend", "blend", "quality", "blend", "fresh", "blend", "explore", "blend",
    ];

    for position in 0..limit.min(candidates.len()) {
        let channel = channel_pattern[position % channel_pattern.len()];
        let mut best: Option<(usize, f64)> = None;
        for relaxed in [false, true] {
            for (candidate_index, candidate) in candidates.iter().enumerate() {
                let row = &rows[candidate.row_index];
                if selected_ids.contains(&row.id) {
                    continue;
                }
                let creator_count = if candidate.creator.is_empty() {
                    0
                } else {
                    *creator_counts.get(&candidate.creator).unwrap_or(&0)
                };
                let tag_count = if candidate.primary_tag.is_empty() {
                    0
                } else {
                    *tag_counts.get(&candidate.primary_tag).unwrap_or(&0)
                };
                if !relaxed && (creator_count >= 2 || tag_count >= 3) {
                    continue;
                }
                let diversity_penalty = creator_count as f64 * 0.12 + tag_count as f64 * 0.05;
                let score = recommendation_channel_score(candidate, channel) - diversity_penalty;
                if best.map(|(_, current)| score > current).unwrap_or(true) {
                    best = Some((candidate_index, score));
                }
            }
            if best.is_some() {
                break;
            }
        }
        let Some((candidate_index, _)) = best else {
            break;
        };
        let candidate = &candidates[candidate_index];
        let row = &rows[candidate.row_index];
        selected_ids.insert(row.id);
        if !candidate.creator.is_empty() {
            *creator_counts.entry(candidate.creator.clone()).or_insert(0) += 1;
        }
        if !candidate.primary_tag.is_empty() {
            *tag_counts.entry(candidate.primary_tag.clone()).or_insert(0) += 1;
        }
        selected.push(candidate.row_index);
    }

    let selected_rows: Vec<ArtRow> = selected.iter().map(|index| rows[*index].clone()).collect();
    state
        .recommendation_feed
        .record_batch(&feed_id, selected_rows.iter().map(|row| row.id));
    let previously_seen_in_pool = rows
        .iter()
        .filter(|row| feed_seen_ids.contains(&row.id))
        .count();
    let has_more = previously_seen_in_pool.saturating_add(selected_rows.len()) < rows.len();
    let uids: Vec<String> = selected_rows
        .iter()
        .filter_map(|row| row.uploader_uid.clone())
        .collect();
    let (guild_map, names, dimensions) = tokio::join!(
        super::art_guild::guild_summaries_for_uids(&state, &uids),
        member_display_names(&state.pools.core, &uids),
        artwork_dimensions(&state, &selected_rows),
    );
    let data: Vec<Value> = selected_rows
        .iter()
        .enumerate()
        .map(|(position, row)| {
            let mut value = map_artwork_row(row);
            insert_artwork_dimensions(&mut value, dimensions.get(&row.id).copied());
            let uid = row.uploader_uid.as_deref().unwrap_or("").trim();
            if let Some(obj) = value.as_object_mut() {
                if !uid.is_empty() {
                    if let Some(summary) = guild_map.get(uid) {
                        obj.insert("guild".into(), summary.clone());
                    }
                    if let Some(name) = names.get(uid) {
                        obj.insert("uploader_display_name".into(), json!(name));
                    }
                }
                obj.insert(
                    "recommendation".into(),
                    json!({ "batch_id": batch_id, "position": position }),
                );
            }
            insert_popularity(
                &mut value,
                quality_stats.get(&row.id).copied().unwrap_or_default(),
                PopularityRange::History,
            );
            value
        })
        .collect();

    Ok(json_with_cookie(
        json!({
            "ok": true,
            "data": data,
            "total": rows.len(),
            "batchId": batch_id,
            "feedId": feed_id,
            "hasMore": has_more,
            "cacheReset": cache_reset,
            "algorithmVersion": RECOMMENDATION_VERSION,
            "personalized": personalized
        }),
        set,
    ))
}

async fn record_recommendation_events(
    State(state): State<AppState>,
    user: Option<AuthUser>,
    headers: HeaderMap,
    Json(body): Json<RecommendationEventsBody>,
) -> AppResult<Response> {
    let (anon_id, set) = resolve_anon(&state.cfg.art_cookie_secret, &headers);
    maybe_cleanup_recommendation_events(&state).await;
    let user_id = user.map(|u| u.id);
    let session_id = body
        .session_id
        .as_deref()
        .map(|value| recommendation_bounded_text(value, 80))
        .filter(|value| !value.is_empty());
    let already_recorded: i64 = if let Some(id) = user_id {
        sqlx::query_scalar(
            "SELECT COUNT(1) FROM recommendation_events \
             WHERE user_id=? AND date(created_at)=date('now')",
        )
        .bind(id)
        .fetch_one(&state.pools.art)
        .await?
    } else {
        sqlx::query_scalar(
            "SELECT COUNT(1) FROM recommendation_events \
             WHERE anon_id=? AND date(created_at)=date('now')",
        )
        .bind(&anon_id)
        .fetch_one(&state.pools.art)
        .await?
    };
    let remaining = (RECOMMENDATION_DAILY_EVENT_LIMIT - already_recorded).max(0) as usize;
    let now = now_iso();
    let mut accepted = 0u64;
    let mut tx = state.pools.art.begin().await?;

    for event in body.events.into_iter().take(50).take(remaining) {
        let event_type = event.event_type.trim().to_lowercase();
        if event.artwork_id <= 0
            || !matches!(
                event_type.as_str(),
                "impression" | "open" | "dwell" | "hide"
            )
        {
            continue;
        }
        let batch_id = event
            .batch_id
            .as_deref()
            .map(|value| recommendation_bounded_text(value, 80))
            .filter(|value| !value.is_empty());
        let source = event
            .source
            .as_deref()
            .map(|value| recommendation_bounded_text(value, 32))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "gallery".to_string());
        let position = event.position.map(|value| value.clamp(0, 200));
        let dwell_ms = if event_type == "dwell" {
            Some(event.dwell_ms.unwrap_or(0).clamp(0, 600_000))
        } else {
            None
        };
        let result = sqlx::query(
            "INSERT OR IGNORE INTO recommendation_events(\
                user_id, anon_id, session_id, artwork_id, batch_id, event_type, source, \
                position, dwell_ms, created_at\
             ) SELECT ?,?,?,?,?,?,?,?,?,? \
               WHERE EXISTS(SELECT 1 FROM artworks WHERE id=? AND status='approved')",
        )
        .bind(user_id)
        .bind(&anon_id)
        .bind(session_id.as_deref())
        .bind(event.artwork_id)
        .bind(batch_id)
        .bind(event_type)
        .bind(source)
        .bind(position)
        .bind(dwell_ms)
        .bind(&now)
        .bind(event.artwork_id)
        .execute(&mut *tx)
        .await?;
        accepted += result.rows_affected();
    }
    tx.commit().await?;

    Ok(json_with_cookie(
        json!({ "ok": true, "accepted": accepted }),
        set,
    ))
}

async fn record_recommendation_signal(
    state: &AppState,
    user_id: i64,
    artwork_id: i64,
    event_type: &str,
) {
    let actor_uid = crate::auth_routes::member_uid(user_id);
    let result = sqlx::query(
        "INSERT INTO recommendation_events(\
            user_id, anon_id, artwork_id, event_type, source, created_at\
         ) SELECT ?,?,?,?,?,? \
           WHERE EXISTS(SELECT 1 FROM artworks WHERE id=? AND status='approved') \
             AND NOT EXISTS(\
                SELECT 1 FROM recommendation_events \
                WHERE user_id=? AND artwork_id=? AND event_type=?\
             )",
    )
    .bind(user_id)
    .bind(&actor_uid)
    .bind(artwork_id)
    .bind(event_type)
    .bind("interaction")
    .bind(now_iso())
    .bind(artwork_id)
    .bind(user_id)
    .bind(artwork_id)
    .bind(event_type)
    .execute(&state.pools.art)
    .await;
    if let Err(error) = result {
        tracing::warn!(
            ?error,
            user_id,
            artwork_id,
            event_type,
            "记录画廊推荐信号失败"
        );
    }
}

// 6. 作品详情
async fn get_artwork(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: Option<AuthUser>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let (anon_id, set) = resolve_anon(&state.cfg.art_cookie_secret, &headers);
    let user_id = user.as_ref().map(|value| value.id);

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

    if status == "approved" {
        record_artwork_view(&state, user_id, &anon_id, id).await;
    }
    super::art_guild::record_user_event(&state, user, "browse_artwork", Some(id)).await;

    let mut value = map_artwork_row(&row);
    let author_uid = row.uploader_uid.as_deref().unwrap_or("").trim();
    if !author_uid.is_empty() {
        let names = member_display_names(&state.pools.core, &[author_uid.to_string()]).await;
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "guild".into(),
                super::art_guild::guild_summary_for_uid(&state, author_uid).await,
            );
            if let Some(name) = names.get(author_uid) {
                obj.insert("uploader_display_name".into(), json!(name));
            }
        }
    }
    let mut stats_by_id = popularity_stats(&state, PopularityRange::History).await?;
    let stats = stats_by_id.entry(row.id).or_default();
    stats.likes = stats.likes.max(row.like_total.max(0));
    stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    insert_popularity(&mut value, *stats, PopularityRange::History);
    let favorite_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM artwork_favorites WHERE artwork_id=?")
            .bind(id)
            .fetch_one(&state.pools.art)
            .await?;
    let favorited = match user_id {
        Some(user_id) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(1) FROM artwork_favorites WHERE user_id=? AND artwork_id=?",
            )
            .bind(user_id)
            .bind(id)
            .fetch_one(&state.pools.art)
            .await?
                > 0
        }
        None => false,
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert("favorite_count".into(), json!(favorite_count));
        obj.insert("favorited".into(), json!(favorited));
    }
    Ok(json_with_cookie(json!({ "ok": true, "data": value }), set))
}

// 6.1 与当前作品相关：以作品标签和内容语义为主，历史质量为辅；同作者作品由前端
// 独立成栏，因此这里优先排除同作者，避免两个发现模块重复。
async fn related_artworks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<RecommendationQuery>,
) -> AppResult<Json<Value>> {
    let limit = q.limit.unwrap_or(8).clamp(4, 16) as usize;
    let source: Option<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.id=? AND a.status='approved'"
    ))
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some(source) = source else {
        return Ok(Json(json!({
            "ok": false,
            "message": "Artwork not found",
            "data": []
        })));
    };

    let candidates: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.id<>? ORDER BY a.id DESC LIMIT 1000"
    ))
    .bind(id)
    .fetch_all(&state.pools.art)
    .await?;
    let source_tags: std::collections::HashSet<String> = safe_json_arr(source.tags_json.as_deref())
        .into_iter()
        .filter_map(|value| value.as_str().map(|tag| tag.trim().to_lowercase()))
        .filter(|tag| !tag.is_empty())
        .collect();
    let source_text = related_text_tokens(source.title.as_deref(), source.description.as_deref());
    let source_age = recommendation_age_days(
        source
            .reviewed_at
            .as_deref()
            .or(source.created_at.as_deref()),
    );
    let related_entropy = format!(
        "related:{id}:{}",
        source.title.as_deref().unwrap_or_default()
    );
    let source_uid = source.uploader_uid.as_deref().unwrap_or("").trim();
    let mut stats_by_id = popularity_stats(&state, PopularityRange::History).await?;
    for row in &candidates {
        let stats = stats_by_id.entry(row.id).or_default();
        stats.likes = stats.likes.max(row.like_total.max(0));
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }

    let mut ranked: Vec<(ArtRow, f64)> = candidates
        .into_iter()
        .filter(|row| {
            let uid = row.uploader_uid.as_deref().unwrap_or("").trim();
            source_uid.is_empty() || uid.is_empty() || uid != source_uid
        })
        .map(|row| {
            let tags: std::collections::HashSet<String> = safe_json_arr(row.tags_json.as_deref())
                .into_iter()
                .filter_map(|value| value.as_str().map(|tag| tag.trim().to_lowercase()))
                .filter(|tag| !tag.is_empty())
                .collect();
            let overlap = source_tags.intersection(&tags).count() as f64;
            let union = source_tags.union(&tags).count().max(1) as f64;
            let tag_affinity = overlap * 8.0 + (overlap / union) * 6.0;
            let text_tokens = related_text_tokens(row.title.as_deref(), row.description.as_deref());
            let text_overlap = source_text.intersection(&text_tokens).count() as f64;
            let text_union = source_text.union(&text_tokens).count().max(1) as f64;
            let text_affinity = text_overlap * 3.5 + (text_overlap / text_union) * 8.0;
            let content_affinity = if row.content_type == source.content_type {
                2.4
            } else {
                0.0
            };
            let source_affinity = if row.source_type == source.source_type {
                0.8
            } else {
                0.0
            };
            let quality = stats_by_id
                .get(&row.id)
                .map(|stats| (1.0 + stats.score.max(0.0)).ln() * 0.12)
                .unwrap_or_default();
            let candidate_age =
                recommendation_age_days(row.reviewed_at.as_deref().or(row.created_at.as_deref()));
            let temporal_affinity = (-(source_age - candidate_age).abs() / 365.0).exp() * 0.8;
            let exploration = recommendation_noise(&related_entropy, row.id) * 1.35;
            (
                row,
                tag_affinity
                    + text_affinity
                    + content_affinity
                    + source_affinity
                    + temporal_affinity
                    + quality
                    + exploration,
            )
        })
        .collect();
    ranked.sort_by(|(a_row, a_score), (b_row, b_score)| {
        b_score
            .total_cmp(a_score)
            .then_with(|| b_row.id.cmp(&a_row.id))
    });

    let mut creator_counts = std::collections::HashMap::<String, usize>::new();
    let mut selected = Vec::with_capacity(limit);
    for (row, _) in ranked {
        let creator_key = row
            .uploader_uid
            .as_deref()
            .map(str::trim)
            .filter(|uid| !uid.is_empty())
            .unwrap_or("anonymous")
            .to_string();
        let count = creator_counts.entry(creator_key).or_default();
        if *count >= 2 {
            continue;
        }
        *count += 1;
        selected.push(row);
        if selected.len() >= limit {
            break;
        }
    }

    let uids: Vec<String> = selected
        .iter()
        .filter_map(|row| row.uploader_uid.clone())
        .collect();
    let names = member_display_names(&state.pools.core, &uids).await;
    let guild_map = super::art_guild::guild_summaries_for_uids(&state, &uids).await;
    let data: Vec<Value> = selected
        .iter()
        .map(|row| {
            let mut value = map_artwork_row(row);
            let uid = row.uploader_uid.as_deref().unwrap_or("").trim();
            if let Some(obj) = value.as_object_mut() {
                if let Some(name) = names.get(uid) {
                    obj.insert("uploader_display_name".into(), json!(name));
                }
                if let Some(guild) = guild_map.get(uid) {
                    obj.insert("guild".into(), guild.clone());
                }
            }
            insert_popularity(
                &mut value,
                stats_by_id.get(&row.id).copied().unwrap_or_default(),
                PopularityRange::History,
            );
            value
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "data": data,
        "strategy": "artwork-affinity-v1"
    })))
}

// 6.2 同创作者投稿时间轴：精确返回当前作品左右各一件相邻稿件。
// 返回顺序固定为 [较新, 当前, 较旧]，边界作品缺少一侧时自然缩短。
async fn creator_neighbor_artworks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let source: Option<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.id=? AND a.status='approved'"
    ))
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some(source) = source else {
        return Err(AppError::not_found("作品不存在"));
    };
    let uid = source.uploader_uid.as_deref().unwrap_or("").trim();
    if uid.is_empty() {
        return Ok(Json(json!({
            "ok": true,
            "data": [map_artwork_row(&source)],
            "currentIndex": 0
        })));
    }

    let source_time = source
        .created_at
        .as_deref()
        .or(source.reviewed_at.as_deref())
        .unwrap_or("1970-01-01T00:00:00Z");
    let time_expr =
        "COALESCE(datetime(a.created_at), datetime(a.reviewed_at), datetime('1970-01-01'))";
    let newer: Option<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.uploader_uid=? AND a.id<>? \
         AND ({time_expr} > datetime(?) OR ({time_expr}=datetime(?) AND a.id>?)) \
         ORDER BY {time_expr} ASC, a.id ASC LIMIT 1"
    ))
    .bind(uid)
    .bind(id)
    .bind(source_time)
    .bind(source_time)
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let older: Option<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.uploader_uid=? AND a.id<>? \
         AND ({time_expr} < datetime(?) OR ({time_expr}=datetime(?) AND a.id<?)) \
         ORDER BY {time_expr} DESC, a.id DESC LIMIT 1"
    ))
    .bind(uid)
    .bind(id)
    .bind(source_time)
    .bind(source_time)
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;

    let current_index = i64::from(newer.is_some());
    let mut rows = Vec::with_capacity(3);
    if let Some(row) = newer {
        rows.push(row);
    }
    rows.push(source);
    if let Some(row) = older {
        rows.push(row);
    }
    let data = map_artworks_with_names(&state, &rows).await;
    Ok(Json(json!({
        "ok": true,
        "data": data,
        "currentIndex": current_index
    })))
}

// 同创作者完整投稿时间线：按发布时间从新到旧返回全部公开作品。
// 前端以当前作品为锚点居中，因此左侧自然是更新作品，右侧是更早作品。
async fn creator_artwork_timeline(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let source: Option<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.id=? AND a.status='approved'"
    ))
    .bind(id)
    .fetch_optional(&state.pools.art)
    .await?;
    let Some(source) = source else {
        return Err(AppError::not_found("作品不存在"));
    };
    let uid = source.uploader_uid.as_deref().unwrap_or("").trim();
    if uid.is_empty() {
        return Ok(Json(json!({
            "ok": true,
            "data": [map_artwork_row(&source)],
            "currentIndex": 0,
            "total": 1
        })));
    }

    let time_expr =
        "COALESCE(datetime(a.created_at), datetime(a.reviewed_at), datetime('1970-01-01'))";
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.status='approved' AND a.uploader_uid=? \
         ORDER BY {time_expr} DESC, a.id DESC"
    ))
    .bind(uid)
    .fetch_all(&state.pools.art)
    .await?;
    let current_index = rows.iter().position(|row| row.id == id).unwrap_or(0);
    let total = rows.len();
    let data = map_artworks_with_names(&state, &rows).await;
    Ok(Json(json!({
        "ok": true,
        "data": data,
        "currentIndex": current_index,
        "total": total
    })))
}

// 6.5 缩略图：GET /thumb?path=art/2026-02/x.webp&w=640
// 生成 + 磁盘缓存（uploads/art/.thumbs/<w>/<path>.<ext>.webp）。生产由 nginx 对
// .thumbs/ 静态直出、未命中才回源本端点；缓存全量预热（deploy/backfill-thumbs.sh）
// 后，本端点几乎只在新图首访时被命中。生成走 libvips 子进程（内存有界）。

/// 允许的缩略尺寸白名单（防御任意 w 撑爆缓存目录）。1920 档仅供超长图预览按需生成，
/// 不参与全量预热，避免普通作品产生额外磁盘和计算开销。
const THUMB_WIDTHS: &[u32] = &[320, 640, 960, 1920];
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
        let dimension_path = disp.path.clone();
        let dimensions =
            tokio::task::spawn_blocking(move || haruhi_media::image_dimensions(&dimension_path))
                .await
                .ok()
                .and_then(Result::ok);
        let mut image = json!({ "path": rel_disp, "original": rel_orig });
        if let (Some((width, height)), Some(obj)) = (dimensions, image.as_object_mut()) {
            obj.insert("width".into(), json!(width));
            obj.insert("height".into(), json!(height));
        }
        images_list.push(image);
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
    let origin_url = match normalize_origin_url(get("origin_url")) {
        Ok(url) => url,
        Err(error) => {
            cleanup_saved_art_uploads(&display_files, &original_files).await;
            return Err(error);
        }
    };

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
        record_recommendation_signal(&state, user.id, artwork_id, "comment").await;
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
        record_recommendation_signal(&state, user.id, id, "like").await;
    }
    Ok((status, Json(body)).into_response())
}

async fn toggle_artwork_favorite(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let artwork_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM artworks WHERE id=? AND status='approved'")
            .bind(id)
            .fetch_one(&state.pools.art)
            .await?;
    if artwork_exists == 0 {
        return Err(AppError::not_found("作品不存在"));
    }

    let mut tx = state.pools.art.begin().await?;
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM artwork_favorites WHERE user_id=? AND artwork_id=?")
            .bind(user.id)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let favorited = if exists.is_some() {
        sqlx::query("DELETE FROM artwork_favorites WHERE user_id=? AND artwork_id=?")
            .bind(user.id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        false
    } else {
        sqlx::query("INSERT INTO artwork_favorites(user_id, artwork_id, created_at) VALUES(?,?,?)")
            .bind(user.id)
            .bind(id)
            .bind(now_iso())
            .execute(&mut *tx)
            .await?;
        true
    };
    let favorite_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM artwork_favorites WHERE artwork_id=?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;

    if favorited {
        record_recommendation_signal(&state, user.id, id, "favorite").await;
        super::art_guild::record_user_event(&state, Some(user), "favorite_artwork", Some(id)).await;
    }
    Ok(Json(json!({
        "ok": true,
        "favorited": favorited,
        "favoriteCount": favorite_count
    })))
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

async fn effective_creator_exhibit_ids(
    state: &AppState,
    author_user_id: i64,
) -> AppResult<std::collections::HashSet<i64>> {
    let rows: Vec<ArtRow> = sqlx::query_as(&format!(
        "{SELECT_ART} WHERE a.author_user_id=? ORDER BY a.id DESC"
    ))
    .bind(author_user_id)
    .fetch_all(&state.pools.art)
    .await?;
    if rows.iter().any(|row| row.exhibit_enabled.is_some()) {
        return Ok(rows
            .iter()
            .filter(|row| row.exhibit_enabled == Some(1))
            .map(|row| row.id)
            .collect());
    }

    let mut eligible: Vec<&ArtRow> = rows
        .iter()
        .filter(|row| {
            row.status.as_deref() == Some("approved")
                && row.source_type.as_deref() == Some("personal")
        })
        .collect();
    let mut stats_by_id = popularity_stats(state, PopularityRange::History).await?;
    for row in &eligible {
        let stats = stats_by_id.entry(row.id).or_default();
        stats.likes = stats.likes.max(row.like_total.max(0));
        stats.score = popularity_score(stats.views, stats.likes, stats.comments);
    }
    eligible.sort_by(|a, b| {
        let a_stats = stats_by_id.get(&a.id).copied().unwrap_or_default();
        let b_stats = stats_by_id.get(&b.id).copied().unwrap_or_default();
        b_stats
            .score
            .total_cmp(&a_stats.score)
            .then_with(|| b.like_total.cmp(&a.like_total))
            .then_with(|| b.id.cmp(&a.id))
    });
    Ok(eligible.into_iter().take(3).map(|row| row.id).collect())
}

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
    let exhibit_ids = effective_creator_exhibit_ids(&state, user.id).await?;
    let data: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut value = map_artwork_row(row);
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "exhibit_enabled".into(),
                    json!(exhibit_ids.contains(&row.id)),
                );
                obj.insert(
                    "exhibit_eligible".into(),
                    json!(
                        row.status.as_deref() == Some("approved")
                            && row.source_type.as_deref() == Some("personal")
                    ),
                );
            }
            value
        })
        .collect();

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

    if let Some(exhibit_enabled) = body.get("exhibit_enabled") {
        let Some(enabled) = exhibit_enabled.as_bool() else {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "message": "展位状态格式不正确" })),
            )
                .into_response());
        };
        let eligibility: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT status, source_type FROM artworks WHERE id=? AND author_user_id=?",
        )
        .bind(id)
        .bind(user.id)
        .fetch_optional(&state.pools.art)
        .await?;
        if !matches!(eligibility, Some((Some(ref status), Some(ref source))) if status == "approved" && source == "personal")
        {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "message": "仅已发布的个人作品可加入创作者展位" })),
            )
                .into_response());
        }

        let configured: i64 = sqlx::query_scalar(
            "SELECT COUNT(1) FROM artworks WHERE author_user_id=? AND exhibit_enabled IS NOT NULL",
        )
        .bind(user.id)
        .fetch_one(&state.pools.art)
        .await?;
        let defaults = if configured == 0 {
            effective_creator_exhibit_ids(&state, user.id).await?
        } else {
            std::collections::HashSet::new()
        };
        let mut tx = state.pools.art.begin().await?;
        if configured == 0 {
            sqlx::query("UPDATE artworks SET exhibit_enabled=0 WHERE author_user_id=?")
                .bind(user.id)
                .execute(&mut *tx)
                .await?;
            for default_id in defaults {
                sqlx::query(
                    "UPDATE artworks SET exhibit_enabled=1 WHERE id=? AND author_user_id=?",
                )
                .bind(default_id)
                .bind(user.id)
                .execute(&mut *tx)
                .await?;
            }
        }
        sqlx::query("UPDATE artworks SET exhibit_enabled=? WHERE id=? AND author_user_id=?")
            .bind(if enabled { 1 } else { 0 })
            .bind(id)
            .bind(user.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(Json(json!({ "ok": true, "exhibit_enabled": enabled })).into_response());
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
    let origin_url = normalize_origin_url(s("origin_url"))?;

    let updated = sqlx::query(
        "UPDATE artworks SET title=?, description=?, tags_json=?, tags_norm=?, origin_url=? \
         WHERE id=? AND author_user_id=?",
    )
    .bind(&title)
    .bind(&description)
    .bind(&tags_json)
    .bind(&tags_norm)
    .bind(opt(&origin_url))
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
    let origin_url = normalize_origin_url(s("origin_url"))?;

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
    .bind(opt(&origin_url))
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

async fn admin_points_penalize(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "art", Action::Manage).await?;
    let uid = body
        .get("uid")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::bad_request("UID 不能为空"))?
        .to_string();
    let divisor = body
        .get("divisor")
        .and_then(json_num_i64)
        .ok_or_else(|| AppError::bad_request("惩罚倍率不能为空"))?;
    if !matches!(divisor, 5 | 10) {
        return Err(AppError::bad_request("惩罚倍率只支持 5 或 10"));
    }
    let note = body
        .get("note")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::bad_request("惩罚原因不能为空"))?
        .to_string();

    let now = now_iso();
    let mut tx = state.pools.art.begin().await?;
    let uid_exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 WHERE EXISTS(SELECT 1 FROM guild_profiles WHERE uid=?)
            OR EXISTS(SELECT 1 FROM creators WHERE uid=?)
            OR EXISTS(SELECT 1 FROM points_ledger WHERE uid=?)",
    )
    .bind(&uid)
    .bind(&uid)
    .bind(&uid)
    .fetch_optional(&mut *tx)
    .await?;
    if uid_exists.is_none() {
        return Err(AppError::bad_request("UID 不存在"));
    }
    let previous_total: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE uid=?")
            .bind(&uid)
            .fetch_one(&mut *tx)
            .await?;
    let target_total = if previous_total > 0 {
        previous_total / divisor
    } else {
        previous_total
    };
    let deducted_points = (previous_total - target_total).max(0);
    let auto_note = format!("积分惩罚自动取消：{note}");
    let cancelled_redemptions = sqlx::query(
        "UPDATE guild_reward_redemptions
         SET status='cancelled', admin_note=?, review_note=?, reviewed_at=?
         WHERE uid=? AND status='pending'",
    )
    .bind(&auto_note)
    .bind(&auto_note)
    .bind(&now)
    .bind(&uid)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if deducted_points > 0 {
        let ledger_note =
            format!("积分惩罚：{note}（当前金币按 1/{divisor} 折算，扣除 {deducted_points}G）");
        sqlx::query(
            "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at)
             VALUES(?,?,?,?,?,?,?)",
        )
        .bind(&uid)
        .bind(None::<i64>)
        .bind(-deducted_points)
        .bind(&ledger_note)
        .bind("penalty")
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Json(json!({
        "ok": true,
        "previousTotal": previous_total,
        "targetTotal": target_total,
        "deductedPoints": deducted_points,
        "cancelledRedemptions": cancelled_redemptions
    })))
}

// ============================================================
// 杂项辅助
// ============================================================

/// 清洗作品出处链接并限制为可安全导航的 Web URL。
/// 空值表示未填写；协议校验必须在写入层完成，不能只依赖前端。
fn normalize_origin_url(raw: Option<&str>) -> AppResult<String> {
    let value = safe_text(raw);
    if value.is_empty() {
        return Ok(String::new());
    }
    if value.chars().count() > 500 {
        return Err(AppError::bad_request("作品出处链接不能超过 500 字"));
    }
    let parsed =
        reqwest::Url::parse(&value).map_err(|_| AppError::bad_request("作品出处链接格式不正确"))?;
    if !matches!(parsed.scheme(), "http" | "https") || !parsed.has_host() {
        return Err(AppError::bad_request(
            "作品出处链接仅支持 http 或 https 协议",
        ));
    }
    Ok(value)
}

/// 空串映射为 NULL（对齐旧 `x || null`）。
fn opt(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod origin_url_tests {
    use super::normalize_origin_url;

    #[test]
    fn only_accepts_http_and_https_urls() {
        assert_eq!(
            normalize_origin_url(Some(" https://example.com/art?id=1 ")).unwrap(),
            "https://example.com/art?id=1"
        );
        assert!(normalize_origin_url(Some("javascript:alert(1)")).is_err());
        assert!(normalize_origin_url(Some("data:text/html,hello")).is_err());
        assert!(normalize_origin_url(Some("/relative/path")).is_err());
        assert_eq!(normalize_origin_url(None).unwrap(), "");
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
