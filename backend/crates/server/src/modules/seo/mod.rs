//! seo 模块：robots.txt、sitemap 与内容详情页 HTML meta 注入。
//!
//! 与其他业务模块不同，这些路由挂在**根路径**（不在 /api 下、不过 CSRF 层，
//! 见 routes.rs），由 Nginx 把 /robots.txt、/sitemap*.xml 及详情页 HTML 请求
//! 精确转发过来；全部只读 GET。

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

mod sitemap;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/robots.txt", get(sitemap::robots_txt))
        .route("/sitemap.xml", get(sitemap::sitemap_index))
        .route("/sitemap-static.xml", get(sitemap::sitemap_static))
        .route("/sitemap-news.xml", get(sitemap::sitemap_news))
        .route("/sitemap-novel.xml", get(sitemap::sitemap_novel))
        .route("/sitemap-exam.xml", get(sitemap::sitemap_exam))
        .route("/sitemap-shop.xml", get(sitemap::sitemap_shop))
}

/// XML / HTML 属性通用转义（& < > " '）。
pub(crate) fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
