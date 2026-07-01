//! fiction 模块：凉宫春日同人小说创作站。
//! 一部作品 = 一条 stories + 若干 chapters；统一账号下的 UGC。
//! 本文件先实现公开读接口（书库列表 / 作品详情 / 章节正文 / 分类标签 / 首页聚合）；
//! 创作（写）与互动接口在后续提交补全。挂载于 /api/fiction。

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::AuthUser;
use haruhi_core::parse::clamp_int;
use haruhi_core::{AppError, AppResult};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::state::AppState;

/// 固定分类（slug → 中文名）。同人创作向春日宇宙取材，前端按此渲染分类导航。
pub const CATEGORIES: &[(&str, &str)] = &[
    ("daily", "日常"),
    ("romance", "恋爱"),
    ("school", "校园"),
    ("supernatural", "超自然"),
    ("scifi", "科幻幻想"),
    ("adventure", "冒险"),
    ("parallel", "平行世界"),
    ("comedy", "欢乐向"),
    ("drama", "剧情"),
    ("healing", "治愈"),
    ("other", "其它"),
];

/// 列表/详情统一选取的作品列（顺序与 `StoryRow` 字段一一对应由 sqlx 按列名匹配，无需对齐）。
const STORY_COLS: &str = "id, title, summary, cover_path, category, content_rating, status, \
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
    content_rating: String,
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
        .route("/stories/{id}/comments", get(list_comments))
        .route("/categories", get(list_categories))
        .route("/tags", get(list_tags))
        .route("/spotlight", get(spotlight))
}

// ---------- 序列化助手 ----------

fn category_label(slug: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(s, _)| *s == slug)
        .map(|(_, l)| *l)
        .unwrap_or("其它")
}

fn story_to_json(s: &StoryRow, tags: Vec<String>) -> Value {
    json!({
        "id": s.id,
        "title": s.title,
        "summary": s.summary,
        "coverPath": s.cover_path,
        "category": s.category,
        "categoryLabel": category_label(&s.category),
        "contentRating": s.content_rating,
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

    // 已发布是公开列表的硬条件，其余为可选过滤。
    let mut where_sql = String::from("WHERE status = 'published'");
    let mut params: Vec<String> = Vec::new();

    if let Some(c) = getq("category") {
        let c = c.trim();
        if !c.is_empty() && c != "all" {
            where_sql.push_str(" AND category = ?");
            params.push(c.to_string());
        }
    }
    if let Some(r) = getq("rating") {
        let r = r.trim();
        if !r.is_empty() && r != "all" {
            where_sql.push_str(" AND content_rating = ?");
            params.push(r.to_string());
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
        "SELECT {STORY_COLS} FROM stories WHERE id = ? AND status = 'published'"
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
        "SELECT id, title, author_name, author_uid FROM stories WHERE id = ? AND status = 'published'",
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
        "UPDATE stories SET view_count = view_count + 1 WHERE id = ? AND status = 'published'",
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
        "SELECT category, COUNT(*) FROM stories WHERE status = 'published' GROUP BY category",
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
         WHERE s.status = 'published' GROUP BY t.id ORDER BY c DESC, t.name ASC LIMIT ?",
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
            "SELECT {STORY_COLS} FROM stories WHERE status = 'published' {extra_where} {order} LIMIT ?"
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
