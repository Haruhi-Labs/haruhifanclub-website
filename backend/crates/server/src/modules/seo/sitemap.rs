//! robots.txt 与 sitemap：从各库查询公开内容生成收录清单。
//!
//! 量级小（全站数百 URL），每请求现算，靠 Cache-Control 让上游缓存一小时；
//! 未来章节数逼近 sitemap 单文件 5 万上限时再拆分页（查询已加 LIMIT 护栏）。

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use haruhi_core::AppResult;

use super::esc;
use crate::state::AppState;

/// 单个 sitemap 文件的 URL 数护栏（规范上限 50_000，留余量）。
const MAX_URLS: i64 = 45_000;

fn base(state: &AppState) -> String {
    state.cfg.public_site_url.trim_end_matches('/').to_string()
}

fn xml_response(body: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        body,
    )
        .into_response()
}

/// 各库时间格式不一（RFC3339 / "YYYY-MM-DD HH:MM:SS" / 迁移遗留自由文本）：
/// 统一只认合法的 `YYYY-MM-DD` 前缀作 lastmod，认不出就省略该字段（坏数据不炸 XML）。
fn lastmod_date(s: &str) -> Option<&str> {
    let b = s.as_bytes();
    if b.len() < 10 {
        return None;
    }
    let ok = b[..10].iter().enumerate().all(|(i, c)| match i {
        4 | 7 => *c == b'-',
        _ => c.is_ascii_digit(),
    });
    ok.then(|| &s[..10])
}

fn push_url(out: &mut String, loc: &str, lastmod: Option<&str>) {
    out.push_str("  <url><loc>");
    out.push_str(&esc(loc));
    out.push_str("</loc>");
    if let Some(d) = lastmod.and_then(lastmod_date) {
        out.push_str("<lastmod>");
        out.push_str(d);
        out.push_str("</lastmod>");
    }
    out.push_str("</url>\n");
}

fn urlset(urls: String) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
    )
}

// ---- robots.txt ----

/// 收敛爬虫抓取面：后台/账号/编辑器等页面 Disallow（`/*/` 通配是 Google/Bing/百度
/// 事实标准）；/uploads/ 保持可抓（og:image 与图片收录依赖）。/console、/library、
/// /design-system 同时有页面级 noindex 作纵深防御。
pub async fn robots_txt(State(state): State<AppState>) -> impl IntoResponse {
    let body = format!(
        "User-agent: *\n\
         Disallow: /api/\n\
         Disallow: /console/\n\
         Disallow: /design-system/\n\
         Disallow: /library/\n\
         Disallow: /*/login\n\
         Disallow: /*/account\n\
         Disallow: /*/admin\n\
         Disallow: /*/verify-email\n\
         Disallow: /*/reset-password\n\
         Disallow: /*/write\n\
         Disallow: /*/submit\n\
         Disallow: /*/upload\n\
         Disallow: /*/search\n\
         Disallow: /*/bookmarks\n\
         Disallow: /*/create\n\
         Disallow: /*/terminal\n\
         Disallow: /*/feedback\n\
         Disallow: /shop/cart\n\
         Disallow: /shop/checkout\n\
         Disallow: /shop/payment\n\
         Disallow: /shop/success\n\
         Disallow: /shop/query\n\
         Disallow: /shop/account-portal\n\
         \n\
         Sitemap: {}/sitemap.xml\n",
        base(&state)
    );
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        body,
    )
}

// ---- sitemap ----

pub async fn sitemap_index(State(state): State<AppState>) -> impl IntoResponse {
    let b = base(&state);
    let items: String = ["static", "news", "novel", "exam", "shop"]
        .iter()
        .map(|s| format!("  <sitemap><loc>{b}/sitemap-{s}.xml</loc></sitemap>\n"))
        .collect();
    xml_response(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{items}</sitemapindex>\n"
    ))
}

/// 各 app 可收录的列表页/固定页（无数据库来源，硬编码维护）。
pub async fn sitemap_static(State(state): State<AppState>) -> impl IntoResponse {
    const PAGES: &[&str] = &[
        "/news/",
        "/news/handbook",
        "/news/activity",
        "/news/store",
        "/novel/",
        "/novel/library",
        "/exam/",
        "/shop/",
        "/art/",
        "/art/gallery",
        "/art/announcements",
        "/download/",
    ];
    let b = base(&state);
    let mut urls = String::new();
    for p in PAGES {
        push_url(&mut urls, &format!("{b}{p}"), None);
    }
    xml_response(urlset(urls))
}

pub async fn sitemap_news(State(state): State<AppState>) -> AppResult<Response> {
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, COALESCE(created_at, date) FROM articles \
         WHERE status IS NULL OR status = 'published' ORDER BY id DESC",
    )
    .fetch_all(&state.pools.news)
    .await?;
    let b = base(&state);
    let mut urls = String::new();
    for (id, lm) in &rows {
        push_url(&mut urls, &format!("{b}/news/blog/{id}"), lm.as_deref());
    }
    Ok(xml_response(urlset(urls)))
}

/// fiction（URL 前缀 /novel/）：已发布且至少一章的作品 + 其已发布章节。
pub async fn sitemap_novel(State(state): State<AppState>) -> AppResult<Response> {
    let stories: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, COALESCE(last_chapter_at, updated_at) FROM stories \
         WHERE status = 'published' AND chapter_count > 0 ORDER BY id",
    )
    .fetch_all(&state.pools.fiction)
    .await?;
    let chapters: Vec<(i64, i64, Option<String>)> = sqlx::query_as(
        "SELECT c.story_id, c.id, COALESCE(c.published_at, c.updated_at) \
         FROM chapters c JOIN stories s ON s.id = c.story_id \
         WHERE c.status = 'published' AND s.status = 'published' AND s.chapter_count > 0 \
         ORDER BY c.story_id, c.position LIMIT ?",
    )
    .bind(MAX_URLS)
    .fetch_all(&state.pools.fiction)
    .await?;
    let b = base(&state);
    let mut urls = String::new();
    for (id, lm) in &stories {
        push_url(&mut urls, &format!("{b}/novel/story/{id}"), lm.as_deref());
    }
    for (sid, cid, lm) in &chapters {
        push_url(
            &mut urls,
            &format!("{b}/novel/story/{sid}/chapter/{cid}"),
            lm.as_deref(),
        );
    }
    Ok(xml_response(urlset(urls)))
}

pub async fn sitemap_exam(State(state): State<AppState>) -> AppResult<Response> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, updated_at FROM exams WHERE status = 'published' ORDER BY created_at DESC",
    )
    .fetch_all(&state.pools.exam)
    .await?;
    let b = base(&state);
    let mut urls = String::new();
    for (id, lm) in &rows {
        // exam id 为 TEXT：进 URL 前做字符白名单校验，异常 id 直接跳过
        if !id.is_empty()
            && id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            push_url(&mut urls, &format!("{b}/exam/exam/{id}"), lm.as_deref());
        }
    }
    Ok(xml_response(urlset(urls)))
}

/// products 表无可见性字段（下架即硬删），存在即公开。
pub async fn sitemap_shop(State(state): State<AppState>) -> AppResult<Response> {
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT id FROM products ORDER BY sortOrder, id")
        .fetch_all(&state.pools.shop)
        .await?;
    let b = base(&state);
    let mut urls = String::new();
    for (id,) in &rows {
        push_url(&mut urls, &format!("{b}/shop/product/{id}"), None);
    }
    Ok(xml_response(urlset(urls)))
}
