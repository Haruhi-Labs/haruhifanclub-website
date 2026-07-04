//! fiction 模块：凉宫春日同人小说创作站。
//! 一部作品 = 一条 stories + 若干 chapters；统一账号下的 UGC。
//! 提供公开读接口（书库列表 / 作品详情 / 章节正文 / 分类标签 / 首页聚合）与
//! 创作接口（作者本人的作品/章节增删改、草稿↔发布、封面上传）。挂载于 /api/fiction。

use std::collections::HashMap;

use axum::extract::{Multipart, Path, Query, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::parse::{clamp_int, clamp_len};
use haruhi_core::{AppError, AppResult};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::auth_routes::{member_uid, require_verified_member};
use crate::state::AppState;

/// 固定分类（slug → 中文名）。同人创作向春日宇宙取材，前端按此渲染分类导航。
pub const CATEGORIES: &[(&str, &str)] = &[
    ("daily", "日常"),
    ("romance", "恋爱"),
    ("adventure", "冒险"),
    ("parallel", "平行世界"),
    ("fantasy", "幻想"),
    ("mystery", "推理"),
    ("comedy", "欢乐向"),
];

/// 列表/详情统一选取的作品列（顺序与 `StoryRow` 字段一一对应由 sqlx 按列名匹配，无需对齐）。
const STORY_COLS: &str = "id, title, summary, cover_path, category, status, \
     is_completed, featured, author_user_id, author_uid, author_name, word_count, chapter_count, \
     view_count, like_count, bookmark_count, comment_count, created_at, updated_at, published_at, \
     last_chapter_at";

#[derive(sqlx::FromRow)]
struct StoryRow {
    id: i64,
    title: String,
    summary: String,
    cover_path: Option<String>,
    category: String,
    status: String,
    is_completed: i64,
    featured: i64,
    author_user_id: Option<i64>,
    author_uid: Option<String>,
    author_name: String,
    word_count: i64,
    chapter_count: i64,
    view_count: i64,
    like_count: i64,
    bookmark_count: i64,
    comment_count: i64,
    created_at: String,
    updated_at: String,
    published_at: Option<String>,
    last_chapter_at: Option<String>,
}

/// 章节目录项（作品页 / 阅读页 TOC）。
type ChapterMeta = (i64, String, i64, i64, Option<String>);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stories", get(list_stories))
        .route("/stories/{id}", get(get_story))
        .route("/stories/{id}/chapters/{cid}", get(get_chapter))
        .route("/stories/{id}/views", post(bump_view))
        .route(
            "/stories/{id}/comments",
            get(list_comments).post(create_comment),
        )
        .route("/stories/{id}/like", post(toggle_like))
        .route("/stories/{id}/bookmark", post(toggle_bookmark))
        .route("/categories", get(list_categories))
        .route("/tags", get(list_tags))
        .route("/spotlight", get(spotlight))
        // ---- 互动 / 个人中心（需登录）----
        .route("/me/comments", get(my_comments))
        .route("/me/comments/{id}", delete(delete_my_comment))
        .route("/me/bookmarks", get(my_bookmarks))
        .route("/me/stats", get(my_stats))
        .route("/me/progress/{sid}", get(get_progress).put(put_progress))
        // ---- 后台审核（需 fiction 角色）----
        .route("/admin/overview", get(admin_overview))
        .route("/admin/stories", get(admin_list_stories))
        .route(
            "/admin/stories/{id}",
            patch(admin_update_story).delete(admin_delete_story),
        )
        .route("/admin/comments", get(admin_list_comments))
        .route("/admin/comments/{id}", patch(admin_update_comment))
        // ---- 创作（需登录 + 作者本人）----
        .route("/me/stories", get(my_stories).post(create_story))
        .route(
            "/me/stories/{id}",
            get(my_story).patch(update_story).delete(delete_story),
        )
        .route("/me/stories/{id}/restore", post(restore_story))
        .route("/me/stories/{id}/chapters", post(create_chapter))
        .route("/me/stories/{id}/chapters/reorder", post(reorder_chapters))
        .route(
            "/me/stories/{id}/chapters/{cid}",
            get(my_chapter).patch(update_chapter).delete(delete_chapter),
        )
        .route("/me/covers", post(upload_cover))
}

// ---------- 序列化助手 ----------

fn category_label(slug: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(s, _)| *s == slug)
        .map(|(_, l)| *l)
        .unwrap_or("日常")
}

fn story_to_json(s: &StoryRow, tags: Vec<String>) -> Value {
    json!({
        "id": s.id,
        "title": s.title,
        "summary": s.summary,
        "coverPath": s.cover_path,
        "category": s.category,
        "categoryLabel": category_label(&s.category),
        "status": s.status,
        "isCompleted": s.is_completed != 0,
        "featured": s.featured != 0,
        "authorUserId": s.author_user_id,
        "authorUid": s.author_uid,
        "authorName": s.author_name,
        "wordCount": s.word_count,
        "chapterCount": s.chapter_count,
        "viewCount": s.view_count,
        "likeCount": s.like_count,
        "bookmarkCount": s.bookmark_count,
        "commentCount": s.comment_count,
        "createdAt": s.created_at,
        "updatedAt": s.updated_at,
        "publishedAt": s.published_at,
        "lastChapterAt": s.last_chapter_at,
        "tags": tags,
    })
}

fn chapter_meta_json(c: &ChapterMeta) -> Value {
    json!({
        "id": c.0,
        "title": c.1,
        "position": c.2,
        "wordCount": c.3,
        "publishedAt": c.4,
    })
}

/// 批量加载若干作品的标签，返回 story_id → 标签名列表。
async fn load_tags_for(pool: &SqlitePool, ids: &[i64]) -> AppResult<HashMap<i64, Vec<String>>> {
    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    if ids.is_empty() {
        return Ok(map);
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT st.story_id, t.name FROM story_tags st JOIN tags t ON t.id = st.tag_id \
         WHERE st.story_id IN ({placeholders}) ORDER BY t.name"
    );
    let mut q = sqlx::query_as::<_, (i64, String)>(&sql);
    for id in ids {
        q = q.bind(id);
    }
    for (sid, name) in q.fetch_all(pool).await? {
        map.entry(sid).or_default().push(name);
    }
    Ok(map)
}

/// 把一组作品行连同标签组装为 JSON 数组（列表 / 首页聚合复用）。
async fn rows_to_cards(pool: &SqlitePool, rows: Vec<StoryRow>) -> AppResult<Vec<Value>> {
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut tags = load_tags_for(pool, &ids).await?;
    Ok(rows
        .iter()
        .map(|r| story_to_json(r, tags.remove(&r.id).unwrap_or_default()))
        .collect())
}

// ---------- 公开读接口 ----------

/// GET /api/fiction/stories —— 书库列表：分类/标签/完结/分级筛选 + 搜索 + 排序 + 分页。
async fn list_stories(
    State(state): State<AppState>,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let getq = |k: &str| q.get(k).map(|s| s.as_str());
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 1, 48, 12);
    let offset = (page - 1) * page_size;

    // 读者可见的硬条件：未被作者下架，且至少有一个已发布章节；其余为可选过滤。
    let mut where_sql = String::from("WHERE status != 'hidden' AND chapter_count > 0");
    let mut params: Vec<String> = Vec::new();

    if let Some(c) = getq("category") {
        let c = c.trim();
        if !c.is_empty() && c != "all" {
            where_sql.push_str(" AND category = ?");
            params.push(c.to_string());
        }
    }
    match getq("completed") {
        Some("1") | Some("true") => where_sql.push_str(" AND is_completed = 1"),
        Some("0") | Some("false") => where_sql.push_str(" AND is_completed = 0"),
        _ => {}
    }
    if let Some(t) = getq("tag") {
        let t = t.trim();
        if !t.is_empty() {
            where_sql.push_str(
                " AND id IN (SELECT st.story_id FROM story_tags st \
                 JOIN tags t ON t.id = st.tag_id WHERE t.name = ?)",
            );
            params.push(t.to_string());
        }
    }
    if let Some(kw) = getq("q") {
        let kw = kw.trim();
        if !kw.is_empty() {
            let like = format!("%{}%", kw.replace(['%', '_'], ""));
            where_sql.push_str(" AND (title LIKE ? OR summary LIKE ? OR author_name LIKE ?)");
            params.push(like.clone());
            params.push(like.clone());
            params.push(like);
        }
    }

    let order_by = match getq("sort").unwrap_or("latest") {
        "updated" => {
            "ORDER BY datetime(COALESCE(last_chapter_at, published_at, created_at)) DESC, id DESC"
        }
        "popular" => "ORDER BY like_count DESC, view_count DESC, id DESC",
        "views" => "ORDER BY view_count DESC, id DESC",
        "words" => "ORDER BY word_count DESC, id DESC",
        "oldest" => "ORDER BY datetime(COALESCE(published_at, created_at)) ASC, id ASC",
        _ => "ORDER BY datetime(COALESCE(published_at, created_at)) DESC, id DESC",
    };

    let count_sql = format!("SELECT COUNT(*) FROM stories {where_sql}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        count_q = count_q.bind(p);
    }
    let total: i64 = count_q.fetch_one(&state.pools.fiction).await?;

    let list_sql =
        format!("SELECT {STORY_COLS} FROM stories {where_sql} {order_by} LIMIT ? OFFSET ?");
    let mut list_q = sqlx::query_as::<_, StoryRow>(&list_sql);
    for p in &params {
        list_q = list_q.bind(p);
    }
    let rows = list_q
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.fiction)
        .await?;

    let stories = rows_to_cards(&state.pools.fiction, rows).await?;

    Ok(Json(json!({
        "ok": true,
        "stories": stories,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// GET /api/fiction/stories/{id} —— 作品详情：作品 + 已发布章节目录 + 标签 + 阅读者互动态。
async fn get_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: Option<AuthUser>,
) -> AppResult<Json<Value>> {
    let row: Option<StoryRow> = sqlx::query_as(&format!(
        "SELECT {STORY_COLS} FROM stories WHERE id = ? AND status != 'hidden' AND chapter_count > 0"
    ))
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let s = row.ok_or_else(|| AppError::not_found("作品不存在"))?;

    let tags = load_tags_for(&state.pools.fiction, &[s.id])
        .await?
        .remove(&s.id)
        .unwrap_or_default();

    let chapters: Vec<ChapterMeta> = sqlx::query_as(
        "SELECT id, title, position, word_count, published_at FROM chapters \
         WHERE story_id = ? AND status = 'published' ORDER BY position ASC, id ASC",
    )
    .bind(s.id)
    .fetch_all(&state.pools.fiction)
    .await?;

    // 登录用户的点赞/收藏态与作者身份（匿名一律 false）。
    let (mut liked, mut bookmarked, mut is_author) = (false, false, false);
    if let Some(u) = &user {
        is_author = s.author_user_id == Some(u.id);
        let kinds: Vec<(String,)> =
            sqlx::query_as("SELECT kind FROM reactions WHERE user_id = ? AND story_id = ?")
                .bind(u.id)
                .bind(s.id)
                .fetch_all(&state.pools.fiction)
                .await?;
        liked = kinds.iter().any(|k| k.0 == "like");
        bookmarked = kinds.iter().any(|k| k.0 == "bookmark");
    }

    Ok(Json(json!({
        "ok": true,
        "story": story_to_json(&s, tags),
        "chapters": chapters.iter().map(chapter_meta_json).collect::<Vec<_>>(),
        "liked": liked,
        "bookmarked": bookmarked,
        "isAuthor": is_author,
    })))
}

/// GET /api/fiction/stories/{id}/chapters/{cid} —— 章节正文 + 上/下章 + 目录。
async fn get_chapter(
    State(state): State<AppState>,
    Path((id, cid)): Path<(i64, i64)>,
) -> AppResult<Json<Value>> {
    let story: Option<(i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, author_name, author_uid FROM stories WHERE id = ? AND status != 'hidden'",
    )
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let (sid, stitle, sauthor, sauthor_uid) =
        story.ok_or_else(|| AppError::not_found("作品不存在"))?;

    let ch: Option<(i64, String, String, String, i64, i64, Option<String>)> = sqlx::query_as(
        "SELECT id, title, content_html, author_note, word_count, position, published_at \
         FROM chapters WHERE id = ? AND story_id = ? AND status = 'published'",
    )
    .bind(cid)
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let (chid, chtitle, content_html, author_note, word_count, position, published_at) =
        ch.ok_or_else(|| AppError::not_found("章节不存在"))?;

    let prev: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, title FROM chapters WHERE story_id = ? AND status = 'published' \
         AND (position < ? OR (position = ? AND id < ?)) ORDER BY position DESC, id DESC LIMIT 1",
    )
    .bind(id)
    .bind(position)
    .bind(position)
    .bind(chid)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let next: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, title FROM chapters WHERE story_id = ? AND status = 'published' \
         AND (position > ? OR (position = ? AND id > ?)) ORDER BY position ASC, id ASC LIMIT 1",
    )
    .bind(id)
    .bind(position)
    .bind(position)
    .bind(chid)
    .fetch_optional(&state.pools.fiction)
    .await?;

    let toc: Vec<ChapterMeta> = sqlx::query_as(
        "SELECT id, title, position, word_count, published_at FROM chapters \
         WHERE story_id = ? AND status = 'published' ORDER BY position ASC, id ASC",
    )
    .bind(id)
    .fetch_all(&state.pools.fiction)
    .await?;

    Ok(Json(json!({
        "ok": true,
        "story": { "id": sid, "title": stitle, "authorName": sauthor, "authorUid": sauthor_uid },
        "chapter": {
            "id": chid,
            "title": chtitle,
            "contentHtml": content_html,
            "authorNote": author_note,
            "wordCount": word_count,
            "position": position,
            "publishedAt": published_at,
        },
        "prev": prev.map(|(i, t)| json!({ "id": i, "title": t })),
        "next": next.map(|(i, t)| json!({ "id": i, "title": t })),
        "chapters": toc.iter().map(chapter_meta_json).collect::<Vec<_>>(),
    })))
}

/// POST /api/fiction/stories/{id}/views —— 阅读量 +1（公开，无需登录；前端每会话调一次）。
async fn bump_view(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let res = sqlx::query(
        "UPDATE stories SET view_count = view_count + 1 WHERE id = ? AND status != 'hidden'",
    )
    .bind(id)
    .execute(&state.pools.fiction)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::not_found("作品不存在"));
    }
    let v: i64 = sqlx::query_scalar("SELECT view_count FROM stories WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pools.fiction)
        .await?;
    Ok(Json(json!({ "ok": true, "viewCount": v })))
}

/// GET /api/fiction/stories/{id}/comments —— 评论列表（?chapterId 指定章节，否则作品级）。
async fn list_comments(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let getq = |k: &str| q.get(k).map(|s| s.as_str());
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 1, 100, 30);
    let offset = (page - 1) * page_size;
    let chapter_id: Option<i64> = getq("chapterId").and_then(|s| s.trim().parse().ok());

    // 章节级（chapter_id = ?）或作品级（chapter_id IS NULL）。
    let (scope_sql, scope_bind) = match chapter_id {
        Some(c) => ("AND chapter_id = ?", Some(c)),
        None => ("AND chapter_id IS NULL", None),
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM comments WHERE story_id = ? AND status = 'visible' {scope_sql}"
    );
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql).bind(id);
    if let Some(c) = scope_bind {
        count_q = count_q.bind(c);
    }
    let total: i64 = count_q.fetch_one(&state.pools.fiction).await?;

    let list_sql = format!(
        "SELECT id, chapter_id, parent_id, author_user_id, author_uid, author_name, body, created_at \
         FROM comments WHERE story_id = ? AND status = 'visible' {scope_sql} \
         ORDER BY datetime(created_at) ASC, id ASC LIMIT ? OFFSET ?"
    );
    let mut list_q = sqlx::query_as::<
        _,
        (
            i64,
            Option<i64>,
            Option<i64>,
            i64,
            Option<String>,
            String,
            String,
            String,
        ),
    >(&list_sql)
    .bind(id);
    if let Some(c) = scope_bind {
        list_q = list_q.bind(c);
    }
    let rows = list_q
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.fiction)
        .await?;

    let comments: Vec<Value> = rows
        .into_iter()
        .map(|(cid, ch, parent, uid_num, uid, name, body, created)| {
            json!({
                "id": cid,
                "chapterId": ch,
                "parentId": parent,
                "authorUserId": uid_num,
                "authorUid": uid,
                "authorName": name,
                "body": body,
                "createdAt": created,
            })
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "comments": comments,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// GET /api/fiction/categories —— 固定分类 + 各分类已发布作品数。
async fn list_categories(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT category, COUNT(*) FROM stories WHERE status != 'hidden' AND chapter_count > 0 GROUP BY category",
    )
    .fetch_all(&state.pools.fiction)
    .await?;
    let counts: HashMap<String, i64> = rows.into_iter().collect();

    let mut total = 0i64;
    let list: Vec<Value> = CATEGORIES
        .iter()
        .map(|(slug, label)| {
            let count = counts.get(*slug).copied().unwrap_or(0);
            total += count;
            json!({ "slug": slug, "label": label, "count": count })
        })
        .collect();

    Ok(Json(
        json!({ "ok": true, "categories": list, "total": total }),
    ))
}

/// GET /api/fiction/tags —— 热门标签（按已发布作品使用次数，?limit 默认 40）。
async fn list_tags(
    State(state): State<AppState>,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let limit = clamp_int(q.get("limit").map(|s| s.as_str()), 1, 100, 40);
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT t.name, COUNT(*) AS c FROM story_tags st \
         JOIN tags t ON t.id = st.tag_id \
         JOIN stories s ON s.id = st.story_id \
         WHERE s.status != 'hidden' AND s.chapter_count > 0 GROUP BY t.id ORDER BY c DESC, t.name ASC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.pools.fiction)
    .await?;
    let tags: Vec<Value> = rows
        .into_iter()
        .map(|(name, count)| json!({ "name": name, "count": count }))
        .collect();
    Ok(Json(json!({ "ok": true, "tags": tags })))
}

/// GET /api/fiction/spotlight —— 首页聚合：精选 / 最新 / 热门 / 最近更新。
async fn spotlight(State(state): State<AppState>) -> AppResult<Json<Value>> {
    async fn cards(
        pool: &SqlitePool,
        extra_where: &str,
        order: &str,
        limit: i64,
    ) -> AppResult<Vec<Value>> {
        let sql = format!(
            "SELECT {STORY_COLS} FROM stories WHERE status != 'hidden' AND chapter_count > 0 {extra_where} {order} LIMIT ?"
        );
        let rows = sqlx::query_as::<_, StoryRow>(&sql)
            .bind(limit)
            .fetch_all(pool)
            .await?;
        rows_to_cards(pool, rows).await
    }

    let pool = &state.pools.fiction;
    let featured = cards(
        pool,
        "AND featured = 1",
        "ORDER BY datetime(COALESCE(published_at, created_at)) DESC, id DESC",
        6,
    )
    .await?;
    let latest = cards(
        pool,
        "",
        "ORDER BY datetime(COALESCE(published_at, created_at)) DESC, id DESC",
        8,
    )
    .await?;
    let popular = cards(
        pool,
        "",
        "ORDER BY like_count DESC, view_count DESC, id DESC",
        8,
    )
    .await?;
    let updated = cards(
        pool,
        "",
        "ORDER BY datetime(COALESCE(last_chapter_at, published_at, created_at)) DESC, id DESC",
        8,
    )
    .await?;

    Ok(Json(json!({
        "ok": true,
        "featured": featured,
        "latest": latest,
        "popular": popular,
        "updated": updated,
    })))
}

// ================= 创作接口（需登录 + 作者本人）=================

/// 统一 UTC RFC3339 时间戳（与 schema 默认 strftime 同格式，便于排序与展示）。
fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// 分类归一：非 allowlist 一律落为 other，保证公开筛选口径干净。
fn normalize_category(v: Option<&str>) -> String {
    match v.map(str::trim) {
        Some(s) if CATEGORIES.iter().any(|(slug, _)| *slug == s) => s.to_string(),
        _ => "daily".to_string(),
    }
}

/// 封面相对路径校验：必须落在 fiction/covers/ 下，拒绝穿越，防止 cover_path 指向任意文件。
fn valid_cover(raw: &str) -> Option<String> {
    let p = raw.trim().trim_start_matches('/');
    if !p.starts_with("fiction/covers/") || p.contains('\\') || p.contains("..") || p.contains('\0')
    {
        return None;
    }
    Some(p.to_string())
}

/// 解析标签数组：去空、去重、单个截断 24 字、上限 12 个。
fn parse_tags(v: Option<&Value>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(Value::Array(arr)) = v {
        for item in arr {
            if let Some(s) = item.as_str() {
                let t = s.trim();
                if t.is_empty() {
                    continue;
                }
                let t: String = t.chars().take(24).collect();
                if !out.iter().any(|x| x == &t) {
                    out.push(t);
                }
                if out.len() >= 12 {
                    break;
                }
            }
        }
    }
    out
}

/// 章节正文清洗：只保留适合叙事排版的标签，杜绝 script / on* / style / javascript: 等 UGC XSS。
fn sanitize_html(raw: &str) -> String {
    let tags: std::collections::HashSet<&str> = [
        "p",
        "br",
        "strong",
        "b",
        "em",
        "i",
        "u",
        "s",
        "del",
        "h2",
        "h3",
        "h4",
        "blockquote",
        "ul",
        "ol",
        "li",
        "hr",
        "a",
        "img",
        "figure",
        "figcaption",
        "code",
        "pre",
        "mark",
    ]
    .into_iter()
    .collect();
    ammonia::Builder::default()
        .tags(tags)
        .link_rel(Some("noopener noreferrer nofollow"))
        .url_relative(ammonia::UrlRelative::PassThrough)
        .clean(raw)
        .to_string()
}

/// 由 HTML 抽取纯文本（供字数与摘要）：去标签 + 常见实体还原。
fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// 字数（按中文习惯计非空白字符数）。
fn count_words(text: &str) -> i64 {
    text.chars().filter(|c| !c.is_whitespace()).count() as i64
}

/// 是否 fiction 管理员（超管或 fiction「管理」角色）——可创建并统一维护「独立署名」作品。
async fn is_fiction_admin(core: &SqlitePool, user: &AuthUser) -> bool {
    authorize(core, user, "fiction", Action::Manage)
        .await
        .is_ok()
}

/// 确认 story 存在且可被该用户管理：作者本人；或作品未绑定账号（独立署名）且请求者为 fiction 管理员。
async fn assert_owner(state: &AppState, id: i64, user: &AuthUser) -> AppResult<()> {
    let owner: Option<Option<i64>> =
        sqlx::query_scalar("SELECT author_user_id FROM stories WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pools.fiction)
            .await?;
    match owner {
        None => Err(AppError::not_found("作品不存在")),
        Some(Some(a)) if a == user.id => Ok(()),
        // 独立署名作品（author_user_id 为空）交由 fiction 管理员统一管理
        Some(None) if is_fiction_admin(&state.pools.core, user).await => Ok(()),
        Some(_) => Err(AppError::Forbidden),
    }
}

/// 重设作品标签（先清后建，标签表按名去重复用）。
async fn set_story_tags(pool: &SqlitePool, story_id: i64, tags: &[String]) -> AppResult<()> {
    sqlx::query("DELETE FROM story_tags WHERE story_id = ?")
        .bind(story_id)
        .execute(pool)
        .await?;
    for name in tags {
        sqlx::query("INSERT INTO tags (name) VALUES (?) ON CONFLICT(name) DO NOTHING")
            .bind(name)
            .execute(pool)
            .await?;
        let tid: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name = ?")
            .bind(name)
            .fetch_one(pool)
            .await?;
        sqlx::query("INSERT OR IGNORE INTO story_tags (story_id, tag_id) VALUES (?, ?)")
            .bind(story_id)
            .bind(tid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// 依据已发布章节重算作品聚合字段（字数 / 章节数 / 最近更新时间）。
async fn recompute_story(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE stories SET \
         word_count = (SELECT COALESCE(SUM(word_count), 0) FROM chapters WHERE story_id = ? AND status = 'published'), \
         chapter_count = (SELECT COUNT(*) FROM chapters WHERE story_id = ? AND status = 'published'), \
         published_at = COALESCE(published_at, (SELECT MIN(published_at) FROM chapters WHERE story_id = ? AND status = 'published')), \
         last_chapter_at = (SELECT published_at FROM chapters WHERE story_id = ? AND status = 'published' ORDER BY datetime(published_at) DESC, id DESC LIMIT 1), \
         updated_at = ? WHERE id = ?",
    )
    .bind(id)
    .bind(id)
    .bind(id)
    .bind(id)
    .bind(now_iso())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// GET /api/fiction/me/stories —— 我的创作列表（含草稿/下架，附总章节数）。
async fn my_stories(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    // 管理员的创作中心额外纳入「独立署名」作品（author_user_id 为空），便于统一维护
    let where_clause = if is_fiction_admin(&state.pools.core, &user).await {
        "WHERE author_user_id = ? OR author_user_id IS NULL"
    } else {
        "WHERE author_user_id = ?"
    };
    let rows: Vec<StoryRow> = sqlx::query_as(&format!(
        "SELECT {STORY_COLS} FROM stories {where_clause} \
         ORDER BY datetime(updated_at) DESC, id DESC"
    ))
    .bind(user.id)
    .fetch_all(&state.pools.fiction)
    .await?;

    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut tags = load_tags_for(&state.pools.fiction, &ids).await?;

    // 总章节数（含草稿），用于创作中心展示“x 章（含 y 草稿）”。
    let mut totals: HashMap<i64, i64> = HashMap::new();
    if !ids.is_empty() {
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT story_id, COUNT(*) FROM chapters WHERE story_id IN ({placeholders}) GROUP BY story_id"
        );
        let mut q = sqlx::query_as::<_, (i64, i64)>(&sql);
        for id in &ids {
            q = q.bind(id);
        }
        totals = q
            .fetch_all(&state.pools.fiction)
            .await?
            .into_iter()
            .collect();
    }

    let stories: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut v = story_to_json(r, tags.remove(&r.id).unwrap_or_default());
            v["chapterTotal"] = json!(totals.get(&r.id).copied().unwrap_or(0));
            v
        })
        .collect();

    Ok(Json(json!({ "ok": true, "stories": stories })))
}

/// POST /api/fiction/me/stories —— 新建作品（草稿）。
async fn create_story(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::bad_request("请求体须为对象"))?;

    // 署名归属：管理员填写 authorName 可创建「独立署名」作品——不绑定任何账号、
    // 署名为自定义文本；普通成员或留空则以本人昵称署名并绑定账号。
    let custom_author = obj
        .get("authorName")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let (author_user_id, author_uid, author_name): (Option<i64>, Option<String>, String) =
        if let Some(name) = custom_author {
            if !is_fiction_admin(&state.pools.core, &user).await {
                return Err(AppError::Forbidden);
            }
            (None, None, clamp_len(Some(name), 60))
        } else {
            let name = require_verified_member(&state.pools.core, &user).await?;
            (Some(user.id), Some(member_uid(user.id)), name)
        };

    let title = clamp_len(obj.get("title").and_then(|v| v.as_str()), 120);
    if title.trim().is_empty() {
        return Err(AppError::bad_request("标题不能为空"));
    }
    let summary = clamp_len(obj.get("summary").and_then(|v| v.as_str()), 2000);
    let category = normalize_category(obj.get("category").and_then(|v| v.as_str()));
    let is_completed = obj
        .get("isCompleted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let cover = match obj.get("coverPath").and_then(|v| v.as_str()) {
        Some(c) if !c.trim().is_empty() => {
            Some(valid_cover(c).ok_or_else(|| AppError::bad_request("封面路径不合法"))?)
        }
        _ => None,
    };
    let tags = parse_tags(obj.get("tags"));

    let now = now_iso();
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO stories \
         (title, summary, cover_path, category, status, is_completed, \
          author_user_id, author_uid, author_name, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 'draft', ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&title)
    .bind(&summary)
    .bind(&cover)
    .bind(&category)
    .bind(is_completed as i64)
    .bind(author_user_id)
    .bind(&author_uid)
    .bind(&author_name)
    .bind(&now)
    .bind(&now)
    .fetch_one(&state.pools.fiction)
    .await?;

    set_story_tags(&state.pools.fiction, id, &tags).await?;

    Ok(Json(json!({ "ok": true, "id": id })))
}

/// GET /api/fiction/me/stories/{id} —— 作者读取自己作品（含草稿章节）供编辑。
async fn my_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;

    let s: StoryRow = sqlx::query_as(&format!("SELECT {STORY_COLS} FROM stories WHERE id = ?"))
        .bind(id)
        .fetch_one(&state.pools.fiction)
        .await?;
    let tags = load_tags_for(&state.pools.fiction, &[id])
        .await?
        .remove(&id)
        .unwrap_or_default();

    let chapters: Vec<(i64, String, i64, i64, String, String)> = sqlx::query_as(
        "SELECT id, title, position, word_count, status, updated_at FROM chapters \
         WHERE story_id = ? ORDER BY position ASC, id ASC",
    )
    .bind(id)
    .fetch_all(&state.pools.fiction)
    .await?;
    let chapters: Vec<Value> = chapters
        .into_iter()
        .map(|(cid, t, pos, wc, st, up)| {
            json!({ "id": cid, "title": t, "position": pos, "wordCount": wc, "status": st, "updatedAt": up })
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "story": story_to_json(&s, tags),
        "chapters": chapters,
    })))
}

/// PATCH /api/fiction/me/stories/{id} —— 编辑作品元信息（白名单字段）。
async fn update_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::bad_request("请求体须为对象"))?;
    let pool = &state.pools.fiction;

    if let Some(t) = obj.get("title").and_then(|v| v.as_str()) {
        let t = clamp_len(Some(t), 120);
        if t.trim().is_empty() {
            return Err(AppError::bad_request("标题不能为空"));
        }
        sqlx::query("UPDATE stories SET title = ? WHERE id = ?")
            .bind(&t)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(s) = obj.get("summary").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE stories SET summary = ? WHERE id = ?")
            .bind(clamp_len(Some(s), 2000))
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(c) = obj.get("category").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE stories SET category = ? WHERE id = ?")
            .bind(normalize_category(Some(c)))
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(b) = obj.get("isCompleted").and_then(|v| v.as_bool()) {
        sqlx::query("UPDATE stories SET is_completed = ? WHERE id = ?")
            .bind(b as i64)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if obj.contains_key("coverPath") {
        let cover = match obj.get("coverPath").and_then(|v| v.as_str()) {
            Some(c) if !c.trim().is_empty() => {
                Some(valid_cover(c).ok_or_else(|| AppError::bad_request("封面路径不合法"))?)
            }
            _ => None,
        };
        sqlx::query("UPDATE stories SET cover_path = ? WHERE id = ?")
            .bind(&cover)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if obj.contains_key("tags") {
        set_story_tags(pool, id, &parse_tags(obj.get("tags"))).await?;
    }
    // 首页精选：仅 fiction 管理员可切换（精选段展示在首页「精选佳作」）
    if let Some(f) = obj.get("featured").and_then(|v| v.as_bool()) {
        if is_fiction_admin(&state.pools.core, &user).await {
            sqlx::query("UPDATE stories SET featured = ? WHERE id = ?")
                .bind(f as i64)
                .bind(id)
                .execute(pool)
                .await?;
        }
    }
    // 独立署名作品：管理员可修改署名（仅对未绑定账号的作品开放，杜绝篡改成员作品署名）
    if let Some(name) = obj.get("authorName").and_then(|v| v.as_str()) {
        let name = clamp_len(Some(name), 60);
        let bound: Option<i64> =
            sqlx::query_scalar("SELECT author_user_id FROM stories WHERE id = ?")
                .bind(id)
                .fetch_one(pool)
                .await?;
        if !name.trim().is_empty()
            && bound.is_none()
            && is_fiction_admin(&state.pools.core, &user).await
        {
            sqlx::query("UPDATE stories SET author_name = ? WHERE id = ?")
                .bind(&name)
                .bind(id)
                .execute(pool)
                .await?;
        }
    }

    sqlx::query("UPDATE stories SET updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(pool)
        .await?;

    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/fiction/me/stories/{id} —— 作者软删除（下架为 hidden，保留数据）。
async fn delete_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    sqlx::query("UPDATE stories SET status = 'hidden', updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.fiction)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/fiction/me/stories/{id}/restore —— 恢复上架（解除作者下架标记）。
/// 作品是否对读者可见由「是否有已发布章节」自动决定，这里只把 hidden 清回 draft。
async fn restore_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    sqlx::query("UPDATE stories SET status = 'draft', updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.pools.fiction)
        .await?;
    recompute_story(&state.pools.fiction, id).await?;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/fiction/me/stories/{id}/chapters —— 新建章节（草稿，追加到卷末）。
async fn create_chapter(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    let obj = body.as_object().cloned().unwrap_or_default();

    let mut title = clamp_len(obj.get("title").and_then(|v| v.as_str()), 120);
    if title.trim().is_empty() {
        title = "未命名章节".to_string();
    }
    let (content, text, wc) = match obj.get("contentHtml").and_then(|v| v.as_str()) {
        Some(raw) if !raw.trim().is_empty() => {
            let clean = sanitize_html(raw);
            let text = html_to_text(&clean);
            let wc = count_words(&text);
            (clean, text, wc)
        }
        _ => (String::new(), String::new(), 0),
    };

    let next_pos: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM chapters WHERE story_id = ?",
    )
    .bind(id)
    .fetch_one(&state.pools.fiction)
    .await?;
    let now = now_iso();
    let cid: i64 = sqlx::query_scalar(
        "INSERT INTO chapters (story_id, title, content_html, text_plain, word_count, position, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'draft', ?, ?) RETURNING id",
    )
    .bind(id)
    .bind(&title)
    .bind(&content)
    .bind(&text)
    .bind(wc)
    .bind(next_pos)
    .bind(&now)
    .bind(&now)
    .fetch_one(&state.pools.fiction)
    .await?;

    Ok(Json(json!({ "ok": true, "id": cid, "position": next_pos })))
}

/// GET /api/fiction/me/stories/{id}/chapters/{cid} —— 作者读取章节全文供编辑。
async fn my_chapter(
    State(state): State<AppState>,
    Path((id, cid)): Path<(i64, i64)>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    let ch: Option<(
        i64,
        String,
        String,
        String,
        i64,
        i64,
        String,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, title, content_html, author_note, word_count, position, status, published_at \
             FROM chapters WHERE id = ? AND story_id = ?",
    )
    .bind(cid)
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let (cid, title, content, note, wc, pos, status, pub_at) =
        ch.ok_or_else(|| AppError::not_found("章节不存在"))?;
    Ok(Json(json!({
        "ok": true,
        "chapter": {
            "id": cid,
            "title": title,
            "contentHtml": content,
            "authorNote": note,
            "wordCount": wc,
            "position": pos,
            "status": status,
            "publishedAt": pub_at,
        }
    })))
}

/// PATCH /api/fiction/me/stories/{id}/chapters/{cid} —— 编辑章节（正文/标题/作者的话/草稿↔发布）。
async fn update_chapter(
    State(state): State<AppState>,
    Path((id, cid)): Path<(i64, i64)>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::bad_request("请求体须为对象"))?;
    let pool = &state.pools.fiction;

    // 确认章节属于该作品
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM chapters WHERE id = ? AND story_id = ?")
            .bind(cid)
            .bind(id)
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::not_found("章节不存在"));
    }

    if let Some(t) = obj.get("title").and_then(|v| v.as_str()) {
        let t = clamp_len(Some(t), 120);
        sqlx::query("UPDATE chapters SET title = ? WHERE id = ?")
            .bind(if t.trim().is_empty() {
                "未命名章节".into()
            } else {
                t
            })
            .bind(cid)
            .execute(pool)
            .await?;
    }
    if let Some(raw) = obj.get("contentHtml").and_then(|v| v.as_str()) {
        let clean = sanitize_html(raw);
        let text = html_to_text(&clean);
        let wc = count_words(&text);
        sqlx::query(
            "UPDATE chapters SET content_html = ?, text_plain = ?, word_count = ? WHERE id = ?",
        )
        .bind(&clean)
        .bind(&text)
        .bind(wc)
        .bind(cid)
        .execute(pool)
        .await?;
    }
    if let Some(n) = obj.get("authorNote").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE chapters SET author_note = ? WHERE id = ?")
            .bind(clamp_len(Some(n), 1000))
            .bind(cid)
            .execute(pool)
            .await?;
    }
    if let Some(st) = obj.get("status").and_then(|v| v.as_str()) {
        match st {
            "published" => {
                sqlx::query(
                    "UPDATE chapters SET status = 'published', published_at = COALESCE(published_at, ?) WHERE id = ?",
                )
                .bind(now_iso())
                .bind(cid)
                .execute(pool)
                .await?;
            }
            "draft" => {
                sqlx::query("UPDATE chapters SET status = 'draft' WHERE id = ?")
                    .bind(cid)
                    .execute(pool)
                    .await?;
            }
            _ => return Err(AppError::bad_request("章节状态非法")),
        }
    }

    sqlx::query("UPDATE chapters SET updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(cid)
        .execute(pool)
        .await?;
    recompute_story(pool, id).await?;

    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/fiction/me/stories/{id}/chapters/{cid} —— 删除章节（物理删除，仅作者本人）。
async fn delete_chapter(
    State(state): State<AppState>,
    Path((id, cid)): Path<(i64, i64)>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    sqlx::query("DELETE FROM chapters WHERE id = ? AND story_id = ?")
        .bind(cid)
        .bind(id)
        .execute(&state.pools.fiction)
        .await?;
    recompute_story(&state.pools.fiction, id).await?;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/fiction/me/stories/{id}/chapters/reorder —— 重排章节顺序。
async fn reorder_chapters(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    assert_owner(&state, id, &user).await?;
    let order = body
        .get("order")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::bad_request("order 须为章节 id 数组"))?;
    let pool = &state.pools.fiction;
    // 整批重排放进事务：中途任一条失败即回滚，避免章节顺序部分更新错乱
    let mut tx = pool.begin().await?;
    for (idx, item) in order.iter().enumerate() {
        let cid = item
            .as_i64()
            .ok_or_else(|| AppError::bad_request("order 元素须为整数 id"))?;
        sqlx::query("UPDATE chapters SET position = ? WHERE id = ? AND story_id = ?")
            .bind((idx as i64) + 1)
            .bind(cid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE stories SET updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/fiction/me/covers —— 上传封面（转 WebP），返回相对路径供 create/update 使用。
async fn upload_cover(
    State(state): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> AppResult<Json<Value>> {
    // 确认账号有效（登录 + 未注销）
    require_verified_member(&state.pools.core, &user).await?;

    let mut bytes: Option<Vec<u8>> = None;
    let mut original = String::from("cover.png");
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        if field.name() == Some("file") {
            if let Some(n) = field.file_name() {
                original = n.to_string();
            }
            let b = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("读取文件失败: {e}")))?;
            bytes = Some(b.to_vec());
        }
    }
    let bytes = bytes.ok_or_else(|| AppError::bad_request("缺少封面文件"))?;
    let ext = haruhi_media::ext_of(&original, "png");
    haruhi_media::check_image(&ext, bytes.len())
        .map_err(|_| AppError::bad_request("封面图片类型或大小不合法"))?;

    let covers_dir = state.cfg.uploads_subdir("fiction").join("covers");
    let stem = uuid::Uuid::new_v4().simple().to_string();

    // 优先转 WebP（质量 80），失败降级原格式。
    let data_for_webp = bytes.clone();
    let webp = tokio::task::spawn_blocking(move || haruhi_media::encode_webp(&data_for_webp, 80.0))
        .await
        .ok()
        .and_then(|r| r.ok());
    let rel = if let Some(w) = webp {
        let name = format!("{stem}.webp");
        haruhi_media::save_file(&covers_dir, &name, &w).await?;
        format!("fiction/covers/{name}")
    } else {
        let name = format!("{stem}.{ext}");
        haruhi_media::save_file(&covers_dir, &name, &bytes).await?;
        format!("fiction/covers/{name}")
    };

    Ok(Json(json!({ "ok": true, "path": rel })))
}

// ================= 互动 / 个人中心 =================

/// 作品必须对读者可见（未下架且有已发布章节），否则 404（互动/评论只针对公开作品）。
async fn ensure_published(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let ok: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM stories WHERE id = ? AND status != 'hidden' AND chapter_count > 0",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    ok.map(|_| ())
        .ok_or_else(|| AppError::not_found("作品不存在"))
}

/// 按 reactions 表实际值回写作品计数（避免自增漂移）。
async fn sync_reaction_count(pool: &SqlitePool, id: i64, kind: &str) -> AppResult<i64> {
    let col = if kind == "like" {
        "like_count"
    } else {
        "bookmark_count"
    };
    let cnt: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM reactions WHERE story_id = ? AND kind = ?")
            .bind(id)
            .bind(kind)
            .fetch_one(pool)
            .await?;
    sqlx::query(&format!("UPDATE stories SET {col} = ? WHERE id = ?"))
        .bind(cnt)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(cnt)
}

/// 切换点赞 / 收藏（幂等：已存在则取消）。
async fn toggle_reaction(
    state: &AppState,
    id: i64,
    user: &AuthUser,
    kind: &str,
) -> AppResult<(bool, i64)> {
    let pool = &state.pools.fiction;
    ensure_published(pool, id).await?;
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM reactions WHERE user_id = ? AND story_id = ? AND kind = ?",
    )
    .bind(user.id)
    .bind(id)
    .bind(kind)
    .fetch_optional(pool)
    .await?;

    let active = if exists.is_some() {
        sqlx::query("DELETE FROM reactions WHERE user_id = ? AND story_id = ? AND kind = ?")
            .bind(user.id)
            .bind(id)
            .bind(kind)
            .execute(pool)
            .await?;
        false
    } else {
        sqlx::query(
            "INSERT OR IGNORE INTO reactions (user_id, story_id, kind, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(user.id)
        .bind(id)
        .bind(kind)
        .bind(now_iso())
        .execute(pool)
        .await?;
        true
    };
    let count = sync_reaction_count(pool, id, kind).await?;
    Ok((active, count))
}

/// POST /api/fiction/stories/{id}/like —— 点赞/取消点赞。
async fn toggle_like(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let (liked, count) = toggle_reaction(&state, id, &user, "like").await?;
    Ok(Json(
        json!({ "ok": true, "liked": liked, "likeCount": count }),
    ))
}

/// POST /api/fiction/stories/{id}/bookmark —— 收藏/取消收藏。
async fn toggle_bookmark(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let (marked, count) = toggle_reaction(&state, id, &user, "bookmark").await?;
    Ok(Json(
        json!({ "ok": true, "bookmarked": marked, "bookmarkCount": count }),
    ))
}

/// 按 comments 表回写作品评论数（仅计 visible）。
async fn sync_comment_count(pool: &SqlitePool, story_id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE stories SET comment_count = \
         (SELECT COUNT(*) FROM comments WHERE story_id = ? AND status = 'visible') WHERE id = ?",
    )
    .bind(story_id)
    .bind(story_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// POST /api/fiction/stories/{id}/comments —— 发表评论（作品级或章节级，支持楼中楼）。
async fn create_comment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let author_name = require_verified_member(&state.pools.core, &user).await?;
    let pool = &state.pools.fiction;
    ensure_published(pool, id).await?;

    let text = clamp_len(body.get("body").and_then(|v| v.as_str()), 2000);
    if text.trim().is_empty() {
        return Err(AppError::bad_request("评论内容不能为空"));
    }

    // 章节级评论：校验章节属于该作品且已发布
    let chapter_id = body.get("chapterId").and_then(|v| v.as_i64());
    if let Some(c) = chapter_id {
        let ok: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM chapters WHERE id = ? AND story_id = ? AND status = 'published'",
        )
        .bind(c)
        .bind(id)
        .fetch_optional(pool)
        .await?;
        if ok.is_none() {
            return Err(AppError::bad_request("章节不存在"));
        }
    }
    // 回复：校验父评论属于该作品且可见
    let parent_id = body.get("parentId").and_then(|v| v.as_i64());
    if let Some(p) = parent_id {
        let ok: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM comments WHERE id = ? AND story_id = ? AND status = 'visible'",
        )
        .bind(p)
        .bind(id)
        .fetch_optional(pool)
        .await?;
        if ok.is_none() {
            return Err(AppError::bad_request("回复的评论不存在"));
        }
    }

    let now = now_iso();
    let uid = member_uid(user.id);
    let cid: i64 = sqlx::query_scalar(
        "INSERT INTO comments \
         (story_id, chapter_id, parent_id, author_user_id, author_uid, author_name, body, status, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 'visible', ?) RETURNING id",
    )
    .bind(id)
    .bind(chapter_id)
    .bind(parent_id)
    .bind(user.id)
    .bind(&uid)
    .bind(&author_name)
    .bind(&text)
    .bind(&now)
    .fetch_one(pool)
    .await?;
    sync_comment_count(pool, id).await?;

    Ok(Json(json!({
        "ok": true,
        "comment": {
            "id": cid,
            "chapterId": chapter_id,
            "parentId": parent_id,
            "authorUserId": user.id,
            "authorUid": uid,
            "authorName": author_name,
            "body": text,
            "createdAt": now,
        }
    })))
}

/// DELETE /api/fiction/me/comments/{id} —— 删除本人评论（软删为 hidden）。
async fn delete_my_comment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let pool = &state.pools.fiction;
    let row: Option<(i64, i64)> =
        sqlx::query_as("SELECT story_id, author_user_id FROM comments WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    let (story_id, author) = row.ok_or_else(|| AppError::not_found("评论不存在"))?;
    if author != user.id {
        return Err(AppError::Forbidden);
    }
    sqlx::query("UPDATE comments SET status = 'hidden' WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    sync_comment_count(pool, story_id).await?;
    Ok(Json(json!({ "ok": true })))
}

/// GET /api/fiction/me/comments —— 我的评论（含所属作品标题）。
async fn my_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let page = clamp_int(q.get("page").map(|s| s.as_str()), 1, 9999, 1);
    let page_size = clamp_int(q.get("pageSize").map(|s| s.as_str()), 1, 60, 20);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments WHERE author_user_id = ? AND status = 'visible'",
    )
    .bind(user.id)
    .fetch_one(&state.pools.fiction)
    .await?;

    let rows: Vec<(i64, i64, Option<i64>, String, String, String)> = sqlx::query_as(
        "SELECT c.id, c.story_id, c.chapter_id, c.body, c.created_at, s.title \
         FROM comments c JOIN stories s ON s.id = c.story_id \
         WHERE c.author_user_id = ? AND c.status = 'visible' \
         ORDER BY datetime(c.created_at) DESC, c.id DESC LIMIT ? OFFSET ?",
    )
    .bind(user.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pools.fiction)
    .await?;

    let comments: Vec<Value> = rows
        .into_iter()
        .map(|(cid, sid, ch, body, created, title)| {
            json!({
                "id": cid,
                "storyId": sid,
                "storyTitle": title,
                "chapterId": ch,
                "body": body,
                "createdAt": created,
            })
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "comments": comments,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// GET /api/fiction/me/bookmarks —— 我的收藏（按收藏时间倒序）。
async fn my_bookmarks(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    let page = clamp_int(q.get("page").map(|s| s.as_str()), 1, 9999, 1);
    let page_size = clamp_int(q.get("pageSize").map(|s| s.as_str()), 1, 48, 12);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stories WHERE status != 'hidden' AND chapter_count > 0 \
         AND id IN (SELECT story_id FROM reactions WHERE user_id = ? AND kind = 'bookmark')",
    )
    .bind(user.id)
    .fetch_one(&state.pools.fiction)
    .await?;

    let rows: Vec<StoryRow> = sqlx::query_as(&format!(
        "SELECT {STORY_COLS} FROM stories WHERE status != 'hidden' AND chapter_count > 0 \
         AND id IN (SELECT story_id FROM reactions WHERE user_id = ? AND kind = 'bookmark') \
         ORDER BY datetime((SELECT created_at FROM reactions WHERE user_id = ? \
         AND story_id = stories.id AND kind = 'bookmark')) DESC, id DESC LIMIT ? OFFSET ?"
    ))
    .bind(user.id)
    .bind(user.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pools.fiction)
    .await?;

    let stories = rows_to_cards(&state.pools.fiction, rows).await?;
    Ok(Json(json!({
        "ok": true,
        "stories": stories,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// GET /api/fiction/me/stats —— 我的创作数据总览。
async fn my_stats(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let pool = &state.pools.fiction;
    // 作品口径按可见性：下架=hidden；已发布=未下架且有已发布章节；草稿=其余
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stories WHERE author_user_id = ?")
        .bind(user.id)
        .fetch_one(pool)
        .await?;
    let hidden: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stories WHERE author_user_id = ? AND status = 'hidden'",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;
    let published: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stories WHERE author_user_id = ? AND status != 'hidden' AND chapter_count > 0",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;
    let draft = total - hidden - published;

    // 仅统计对读者可见作品的对外数据
    let agg: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT SUM(word_count), SUM(view_count), SUM(like_count), SUM(bookmark_count) \
         FROM stories WHERE author_user_id = ? AND status != 'hidden' AND chapter_count > 0",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;
    let chapters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chapters WHERE story_id IN \
         (SELECT id FROM stories WHERE author_user_id = ?) AND status = 'published'",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    Ok(Json(json!({
        "ok": true,
        "works": { "total": total, "published": published, "draft": draft, "hidden": hidden },
        "publishedChapters": chapters,
        "totalWords": agg.0.unwrap_or(0),
        "totalViews": agg.1.unwrap_or(0),
        "totalLikes": agg.2.unwrap_or(0),
        "totalBookmarks": agg.3.unwrap_or(0),
    })))
}

/// GET /api/fiction/me/progress/{sid} —— 读取阅读进度。
async fn get_progress(
    State(state): State<AppState>,
    Path(sid): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let row: Option<(Option<i64>, f64)> = sqlx::query_as(
        "SELECT chapter_id, progress FROM reading_progress WHERE user_id = ? AND story_id = ?",
    )
    .bind(user.id)
    .bind(sid)
    .fetch_optional(&state.pools.fiction)
    .await?;
    Ok(Json(json!({
        "ok": true,
        "progress": row.map(|(ch, p)| json!({ "chapterId": ch, "progress": p })),
    })))
}

/// PUT /api/fiction/me/progress/{sid} —— 保存阅读进度（每人每作品一行 upsert）。
async fn put_progress(
    State(state): State<AppState>,
    Path(sid): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let chapter_id = body.get("chapterId").and_then(|v| v.as_i64());
    let progress = body
        .get("progress")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    sqlx::query(
        "INSERT INTO reading_progress (user_id, story_id, chapter_id, progress, updated_at) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(user_id, story_id) DO UPDATE SET \
         chapter_id = excluded.chapter_id, progress = excluded.progress, updated_at = excluded.updated_at",
    )
    .bind(user.id)
    .bind(sid)
    .bind(chapter_id)
    .bind(progress)
    .bind(now_iso())
    .execute(&state.pools.fiction)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

// ================= 后台审核（需 fiction 角色）=================

/// GET /api/fiction/admin/stories —— 全部作品（可按状态/关键词过滤）。
/// GET /api/fiction/admin/overview —— 后台总览统计（作品 / 评论 / 对外数据）。
async fn admin_overview(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Moderate).await?;
    let pool = &state.pools.fiction;
    let scalar = |sql: &'static str| sqlx::query_scalar::<_, i64>(sql).fetch_one(pool);

    let total = scalar("SELECT COUNT(*) FROM stories").await?;
    // 可见性口径与前台一致：已发布=未下架且有已发布章节；下架=hidden；其余=草稿
    let published =
        scalar("SELECT COUNT(*) FROM stories WHERE status != 'hidden' AND chapter_count > 0")
            .await?;
    let hidden = scalar("SELECT COUNT(*) FROM stories WHERE status = 'hidden'").await?;
    let draft = total - published - hidden;
    let featured = scalar("SELECT COUNT(*) FROM stories WHERE featured = 1").await?;

    let c_total = scalar("SELECT COUNT(*) FROM comments").await?;
    let c_hidden = scalar("SELECT COUNT(*) FROM comments WHERE status = 'hidden'").await?;
    let c_visible = c_total - c_hidden;

    let agg: (Option<i64>, Option<i64>, Option<i64>) =
        sqlx::query_as("SELECT SUM(view_count), SUM(like_count), SUM(bookmark_count) FROM stories")
            .fetch_one(pool)
            .await?;
    let chapters = scalar("SELECT COUNT(*) FROM chapters WHERE status = 'published'").await?;

    Ok(Json(json!({
        "ok": true,
        "works": { "total": total, "published": published, "draft": draft, "hidden": hidden, "featured": featured },
        "comments": { "total": c_total, "visible": c_visible, "hidden": c_hidden },
        "totals": {
            "views": agg.0.unwrap_or(0),
            "likes": agg.1.unwrap_or(0),
            "bookmarks": agg.2.unwrap_or(0),
            "chapters": chapters,
        },
    })))
}

async fn admin_list_stories(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Moderate).await?;
    let getq = |k: &str| q.get(k).map(|s| s.as_str());
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 1, 60, 20);
    let offset = (page - 1) * page_size;

    let mut where_sql = String::from("WHERE 1 = 1");
    let mut params: Vec<String> = Vec::new();
    // 状态筛选走可见性语义（与前台一致），而非 status 列原值——status 列只有 draft/hidden
    match getq("status").map(str::trim) {
        Some("published") => {
            where_sql.push_str(" AND status != 'hidden' AND chapter_count > 0");
        }
        Some("draft") => {
            where_sql.push_str(" AND status != 'hidden' AND chapter_count = 0");
        }
        Some("hidden") => where_sql.push_str(" AND status = 'hidden'"),
        Some("featured") => where_sql.push_str(" AND featured = 1"),
        _ => {}
    }
    if let Some(kw) = getq("q") {
        let kw = kw.trim();
        if !kw.is_empty() {
            let like = format!("%{}%", kw.replace(['%', '_'], ""));
            where_sql.push_str(" AND (title LIKE ? OR author_name LIKE ?)");
            params.push(like.clone());
            params.push(like);
        }
    }

    let count_sql = format!("SELECT COUNT(*) FROM stories {where_sql}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        count_q = count_q.bind(p);
    }
    let total: i64 = count_q.fetch_one(&state.pools.fiction).await?;

    let list_sql = format!(
        "SELECT {STORY_COLS} FROM stories {where_sql} ORDER BY datetime(updated_at) DESC, id DESC LIMIT ? OFFSET ?"
    );
    let mut list_q = sqlx::query_as::<_, StoryRow>(&list_sql);
    for p in &params {
        list_q = list_q.bind(p);
    }
    let rows = list_q
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.fiction)
        .await?;
    let stories = rows_to_cards(&state.pools.fiction, rows).await?;

    Ok(Json(json!({
        "ok": true,
        "stories": stories,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// PATCH /api/fiction/admin/stories/{id} —— 精选 / 上下架。
async fn admin_update_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Moderate).await?;
    let pool = &state.pools.fiction;
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM stories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("作品不存在"));
    }

    if let Some(f) = body.get("featured").and_then(|v| v.as_bool()) {
        sqlx::query("UPDATE stories SET featured = ? WHERE id = ?")
            .bind(f as i64)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(s) = body.get("status").and_then(|v| v.as_str()) {
        if !matches!(s, "draft" | "published" | "hidden") {
            return Err(AppError::bad_request("状态非法"));
        }
        let now = now_iso();
        if s == "published" {
            sqlx::query(
                "UPDATE stories SET status = 'published', published_at = COALESCE(published_at, ?) WHERE id = ?",
            )
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query("UPDATE stories SET status = ? WHERE id = ?")
                .bind(s)
                .bind(id)
                .execute(pool)
                .await?;
        }
    }
    sqlx::query("UPDATE stories SET updated_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/fiction/admin/stories/{id} —— 物理删除作品及其从属数据（需 Manage）。
async fn admin_delete_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Manage).await?;
    let pool = &state.pools.fiction;
    let cover: Option<Option<String>> =
        sqlx::query_scalar("SELECT cover_path FROM stories WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    let cover = match cover {
        Some(c) => c,
        None => return Err(AppError::not_found("作品不存在")),
    };

    // 六张表整体删除放进事务：部分失败即回滚，避免残留孤儿数据
    let mut tx = pool.begin().await?;
    for sql in [
        "DELETE FROM chapters WHERE story_id = ?",
        "DELETE FROM story_tags WHERE story_id = ?",
        "DELETE FROM comments WHERE story_id = ?",
        "DELETE FROM reactions WHERE story_id = ?",
        "DELETE FROM reading_progress WHERE story_id = ?",
        "DELETE FROM stories WHERE id = ?",
    ] {
        sqlx::query(sql).bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await?;

    // 数据删除成功后再清理封面文件（文件删除失败不影响一致性）
    if let Some(rel) = cover.filter(|c| !c.is_empty()) {
        let _ = tokio::fs::remove_file(state.cfg.uploads_dir.join(&rel)).await;
    }
    Ok(Json(json!({ "ok": true })))
}

/// GET /api/fiction/admin/comments —— 评论审核列表（含作品标题）。
async fn admin_list_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Moderate).await?;
    let getq = |k: &str| q.get(k).map(|s| s.as_str());
    let page = clamp_int(getq("page"), 1, 9999, 1);
    let page_size = clamp_int(getq("pageSize"), 1, 100, 30);
    let offset = (page - 1) * page_size;

    let mut where_sql = String::from("WHERE 1 = 1");
    let mut params: Vec<String> = Vec::new();
    if let Some(s) = getq("status") {
        let s = s.trim();
        if !s.is_empty() && s != "all" {
            where_sql.push_str(" AND c.status = ?");
            params.push(s.to_string());
        }
    }
    if let Some(kw) = getq("q") {
        let kw = kw.trim();
        if !kw.is_empty() {
            let like = format!("%{}%", kw.replace(['%', '_'], ""));
            where_sql.push_str(" AND (c.body LIKE ? OR c.author_name LIKE ?)");
            params.push(like.clone());
            params.push(like);
        }
    }

    let count_sql = format!("SELECT COUNT(*) FROM comments c {where_sql}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        count_q = count_q.bind(p);
    }
    let total: i64 = count_q.fetch_one(&state.pools.fiction).await?;

    let list_sql = format!(
        "SELECT c.id, c.story_id, c.chapter_id, c.author_user_id, c.author_name, c.body, \
         c.status, c.created_at, s.title FROM comments c JOIN stories s ON s.id = c.story_id \
         {where_sql} ORDER BY datetime(c.created_at) DESC, c.id DESC LIMIT ? OFFSET ?"
    );
    let mut list_q = sqlx::query_as::<
        _,
        (
            i64,
            i64,
            Option<i64>,
            i64,
            String,
            String,
            String,
            String,
            String,
        ),
    >(&list_sql);
    for p in &params {
        list_q = list_q.bind(p);
    }
    let rows = list_q
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pools.fiction)
        .await?;

    let comments: Vec<Value> = rows
        .into_iter()
        .map(|(cid, sid, ch, uid, name, body, status, created, title)| {
            json!({
                "id": cid,
                "storyId": sid,
                "storyTitle": title,
                "chapterId": ch,
                "authorUserId": uid,
                "authorName": name,
                "body": body,
                "status": status,
                "createdAt": created,
            })
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "comments": comments,
        "pagination": crate::pagination::page_meta(page, page_size, total),
    })))
}

/// PATCH /api/fiction/admin/comments/{id} —— 隐藏 / 恢复评论。
async fn admin_update_comment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "fiction", Action::Moderate).await?;
    let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if !matches!(status, "visible" | "hidden") {
        return Err(AppError::bad_request("状态非法"));
    }
    let pool = &state.pools.fiction;
    let story_id: Option<i64> = sqlx::query_scalar("SELECT story_id FROM comments WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    let story_id = story_id.ok_or_else(|| AppError::not_found("评论不存在"))?;
    sqlx::query("UPDATE comments SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    sync_comment_count(pool, story_id).await?;
    Ok(Json(json!({ "ok": true })))
}
