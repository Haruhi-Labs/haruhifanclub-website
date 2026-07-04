//! 内容详情页 HTML meta 注入 handler。
//!
//! Nginx 把 5 类详情页的 HTML 请求转发到这里；按各库可见性口径查内容、
//! 组 PageMeta、注入对应 app 的 dist 模板返回。内容不存在/不可见时返回
//! **404 状态码 + 带 noindex 的同一 SPA 壳**：爬虫拿到真 404（消灭 soft-404），
//! 浏览器端 SPA 照常启动渲染自己的「不存在」页。
//!
//! 注意 SEO 可见性口径可比业务 API 更严（如 fiction API 允许作者预览
//! `status != 'hidden'`，这里只认 published），互不影响。

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use haruhi_core::AppResult;
use serde_json::json;

use super::{meta, meta::PageMeta, template};
use crate::state::AppState;

const DESC_CHARS: usize = 160;

fn base(state: &AppState) -> String {
    state.cfg.public_site_url.trim_end_matches('/').to_string()
}

/// 注入模板并回包；模板读不到回 500（Nginx error_page 兜底静态壳）。
async fn render(
    state: &AppState,
    app: &'static str,
    status: StatusCode,
    page: PageMeta,
) -> Response {
    let Some(tpl) = template::load(state, app).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let body = template::inject(&tpl, &meta::render_head_block(&page));
    (
        status,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        body,
    )
        .into_response()
}

async fn render_404(state: &AppState, app: &'static str, default_title: &str) -> Response {
    render(
        state,
        app,
        StatusCode::NOT_FOUND,
        PageMeta::not_found(default_title),
    )
    .await
}

// ---- news：/news/blog/{id} ----

const NEWS_TITLE: &str = "春日团报 · 凉宫春日应援团";

pub async fn news_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    type Row = (
        Option<String>, // title
        Option<String>, // subtitle
        Option<String>, // summary
        Option<String>, // author
        Option<String>, // image
        Option<String>, // date/created_at
        Option<String>, // status
    );
    let row: Option<Row> = sqlx::query_as(
        "SELECT title, subtitle, summary, author, image, COALESCE(date, created_at), status \
         FROM articles WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pools.news)
    .await?;
    let Some((title, subtitle, summary, author, image, date, status)) = row else {
        return Ok(render_404(&state, "news", NEWS_TITLE).await);
    };
    if !matches!(status.as_deref(), None | Some("published")) {
        return Ok(render_404(&state, "news", NEWS_TITLE).await);
    }

    let b = base(&state);
    let title = title.unwrap_or_else(|| "团报文章".into());
    let desc = summary
        .as_deref()
        .or(subtitle.as_deref())
        .map(|s| meta::excerpt(s, DESC_CHARS))
        .filter(|s| !s.is_empty());
    let og_image = image
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|i| meta::absolutize(&b, i));
    let author = author
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("凉宫春日应援团");
    let mut json_ld = json!({
        "@context": "https://schema.org",
        "@type": "NewsArticle",
        "headline": title,
        "author": { "@type": "Person", "name": author },
    });
    // date 列存在中文自由文本（旧站迁移），只认 YYYY-MM-DD 前缀进 datePublished
    if let Some(d) = date.as_deref().and_then(super::sitemap::lastmod_date) {
        json_ld["datePublished"] = json!(d);
    }
    if let Some(i) = &og_image {
        json_ld["image"] = json!(i);
    }

    let page = PageMeta {
        description: desc,
        canonical: Some(format!("{b}/news/blog/{id}")),
        og_type: "article",
        og_image,
        json_ld: Some(json_ld),
        ..PageMeta::new(format!("{title} · 春日团报"))
    };
    Ok(render(&state, "news", StatusCode::OK, page).await)
}

// ---- fiction：/novel/story/{id}、/novel/story/{id}/chapter/{cid} ----

const FICTION_TITLE: &str = "春日文库 · 凉宫春日应援团";

/// SEO 口径的作品行：published 且至少一章才可见。
async fn fetch_story(
    state: &AppState,
    id: i64,
) -> AppResult<Option<(String, String, Option<String>, String)>> {
    // (title, summary, cover_path, author_name)
    let row = sqlx::query_as(
        "SELECT title, summary, cover_path, author_name FROM stories \
         WHERE id = ? AND status = 'published' AND chapter_count > 0",
    )
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    Ok(row)
}

pub async fn fiction_story(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let Some((title, summary, cover_path, author_name)) = fetch_story(&state, id).await? else {
        return Ok(render_404(&state, "fiction", FICTION_TITLE).await);
    };
    let b = base(&state);
    let author = if author_name.is_empty() {
        "佚名".to_string()
    } else {
        author_name
    };
    let desc = if summary.is_empty() {
        format!("{author}的凉宫春日同人小说《{title}》")
    } else {
        meta::excerpt(&summary, DESC_CHARS)
    };
    let og_image = cover_path
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|p| meta::absolutize(&b, p));
    let mut json_ld = json!({
        "@context": "https://schema.org",
        "@type": "Book",
        "name": title,
        "author": { "@type": "Person", "name": author },
        "inLanguage": "zh-CN",
    });
    if !summary.is_empty() {
        json_ld["description"] = json!(meta::excerpt(&summary, DESC_CHARS));
    }

    let page = PageMeta {
        description: Some(desc),
        canonical: Some(format!("{b}/novel/story/{id}")),
        og_type: "book",
        og_image,
        json_ld: Some(json_ld),
        ..PageMeta::new(format!("{title} · 春日文库"))
    };
    Ok(render(&state, "fiction", StatusCode::OK, page).await)
}

pub async fn fiction_chapter(
    State(state): State<AppState>,
    Path((id, cid)): Path<(i64, i64)>,
) -> AppResult<Response> {
    let Some((story_title, _, cover_path, _)) = fetch_story(&state, id).await? else {
        return Ok(render_404(&state, "fiction", FICTION_TITLE).await);
    };
    let chapter: Option<(String, String)> = sqlx::query_as(
        "SELECT title, content_html FROM chapters \
         WHERE id = ? AND story_id = ? AND status = 'published'",
    )
    .bind(cid)
    .bind(id)
    .fetch_optional(&state.pools.fiction)
    .await?;
    let Some((chapter_title, content_html)) = chapter else {
        return Ok(render_404(&state, "fiction", FICTION_TITLE).await);
    };

    let b = base(&state);
    let excerpt = meta::excerpt(&content_html, DESC_CHARS);
    let desc = if excerpt.is_empty() {
        format!("《{story_title}》{chapter_title}")
    } else {
        excerpt
    };
    let page = PageMeta {
        description: Some(desc),
        canonical: Some(format!("{b}/novel/story/{id}/chapter/{cid}")),
        og_type: "article",
        og_image: cover_path
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|p| meta::absolutize(&b, p)),
        ..PageMeta::new(format!("{chapter_title} · {story_title} · 春日文库"))
    };
    Ok(render(&state, "fiction", StatusCode::OK, page).await)
}

// ---- exam：/exam/exam/{id} ----

const EXAM_TITLE: &str = "春日试卷中心 · 凉宫春日应援团";

pub async fn exam_paper(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    // TEXT 主键：进 SQL 前先做字符白名单校验，异常 id 一律 404（防注入 canonical/og:url）
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
    if !valid {
        return Ok(render_404(&state, "exam", EXAM_TITLE).await);
    }
    let row: Option<(Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT title, subtitle, status FROM exams WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.pools.exam)
            .await?;
    let Some((title, subtitle, status)) = row else {
        return Ok(render_404(&state, "exam", EXAM_TITLE).await);
    };
    if status.as_deref() != Some("published") {
        return Ok(render_404(&state, "exam", EXAM_TITLE).await);
    }

    let b = base(&state);
    let title = title.unwrap_or_else(|| "趣味试卷".into());
    let desc = subtitle
        .as_deref()
        .map(|s| meta::excerpt(s, DESC_CHARS))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("来挑战《{title}》"));
    let page = PageMeta {
        description: Some(desc),
        canonical: Some(format!("{b}/exam/exam/{id}")),
        ..PageMeta::new(format!("{title} · 春日试卷中心"))
    };
    Ok(render(&state, "exam", StatusCode::OK, page).await)
}

// ---- shop：/shop/product/{id} ----

const SHOP_TITLE: &str = "春日商城 · 凉宫春日应援团";

pub async fn shop_product(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    type Row = (
        Option<String>, // name
        Option<String>, // desc
        Option<String>, // image
        Option<i64>,    // price（整数元）
        Option<i64>,    // discountPrice（整数元）
        Option<i64>,    // stock
        Option<String>, // presaleMode
    );
    let row: Option<Row> = sqlx::query_as(
        "SELECT name, \"desc\", image, price, discountPrice, stock, presaleMode \
         FROM products WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pools.shop)
    .await?;
    let Some((name, desc, image, price, discount, stock, presale_mode)) = row else {
        return Ok(render_404(&state, "shop", SHOP_TITLE).await);
    };

    let b = base(&state);
    let name = name.unwrap_or_else(|| "周边商品".into());
    let description = desc
        .as_deref()
        .map(|s| meta::excerpt(s, DESC_CHARS))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("凉宫春日应援团周边 —— {name}"));
    let og_image = image
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|i| meta::absolutize(&b, i));

    // 实际售价：有折扣价用折扣价；products.price 存的是整数「元」
    // （前端 normalizeProduct 直接 Number(price) 展示，无换算）。
    // 预售商品可购买且不受库存限制（与前端 Product JSON-LD 口径一致）。
    let yuan = discount.filter(|d| *d > 0).or(price).unwrap_or(0);
    let availability = if presale_mode.as_deref().unwrap_or("none") != "none" {
        "https://schema.org/PreOrder"
    } else if stock.unwrap_or(0) > 0 {
        "https://schema.org/InStock"
    } else {
        "https://schema.org/OutOfStock"
    };
    let mut json_ld = json!({
        "@context": "https://schema.org",
        "@type": "Product",
        "name": name,
        "description": description,
        "offers": {
            "@type": "Offer",
            "price": format!("{yuan}.00"),
            "priceCurrency": "CNY",
            "availability": availability,
        },
    });
    if let Some(i) = &og_image {
        json_ld["image"] = json!(i);
    }

    let page = PageMeta {
        description: Some(description),
        canonical: Some(format!("{b}/shop/product/{id}")),
        og_type: "product",
        og_image,
        json_ld: Some(json_ld),
        ..PageMeta::new(format!("{name} · 春日商城"))
    };
    Ok(render(&state, "shop", StatusCode::OK, page).await)
}
