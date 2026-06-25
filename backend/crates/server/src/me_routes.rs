//! 跨模块个人总览：聚合当前登录用户在各业务库（art/news/exam）的内容与积分概况。
//! 跨库无法 JOIN，这里在应用层分别查询各 pool 后于 server 层合并，供个人控制台概览页使用。

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use haruhi_auth::AuthUser;
use haruhi_core::AppResult;
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/me/summary", get(summary))
}

/// 把 (status, count) 列表汇总为 {total, published, pending, hidden}。
/// `published_key` 指定哪个 status 计入「已发布」（art=approved，news/exam=published）；
/// status 为 NULL/空一律视为已发布（news 历史文章 status 为 NULL 即已发布）。
fn tally(rows: &[(Option<String>, i64)], published_key: &str) -> Value {
    let (mut total, mut published, mut pending, mut hidden) = (0i64, 0i64, 0i64, 0i64);
    for (status, count) in rows {
        let s = status.as_deref().unwrap_or("");
        total += count;
        match s {
            "" => published += count,
            k if k == published_key => published += count,
            "pending" | "flagged" => pending += count,
            "hidden" => hidden += count,
            _ => {}
        }
    }
    json!({ "total": total, "published": published, "pending": pending, "hidden": hidden })
}

/// GET /api/me/summary —— 个人控制台概览：跨 art/news/exam 聚合本人内容计数与两套积分余额。
async fn summary(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let uid = crate::auth_routes::member_uid(user.id);

    // ---- art：作品按状态分组、评论数、画廊创作激励积分 ----
    let art_status: Vec<(Option<String>, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM artworks WHERE author_user_id = ? GROUP BY status")
            .bind(user.id)
            .fetch_all(&state.pools.art)
            .await?;
    let art_comments: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE author_user_id = ?")
            .bind(user.id)
            .fetch_one(&state.pools.art)
            .await?;
    let art_points: i64 = sqlx::query_scalar(
        "SELECT COALESCE(CAST(SUM(CAST(NULLIF(TRIM(points), '') AS INTEGER)) AS INTEGER), 0) \
         FROM points_ledger WHERE uid = ?",
    )
    .bind(&uid)
    .fetch_one(&state.pools.art)
    .await?;

    // ---- news：文章按状态、团报积分、兑换记录数 ----
    let news_status: Vec<(Option<String>, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM articles WHERE author_user_id = ? GROUP BY status")
            .bind(user.id)
            .fetch_all(&state.pools.news)
            .await?;
    let news_points: i64 = sqlx::query_scalar(
        "SELECT COALESCE(CAST(NULLIF(TRIM(total), '') AS INTEGER), 0) FROM users WHERE id = ?",
    )
    .bind(&uid)
    .fetch_optional(&state.pools.news)
    .await?
    .unwrap_or(0);
    let news_redemptions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM redemptions WHERE user_id = ?")
            .bind(&uid)
            .fetch_one(&state.pools.news)
            .await?;

    // ---- exam：试卷按状态 ----
    let exam_status: Vec<(Option<String>, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM exams WHERE author_user_id = ? GROUP BY status")
            .bind(user.id)
            .fetch_all(&state.pools.exam)
            .await?;

    Ok(Json(json!({
        "ok": true,
        "artworks": tally(&art_status, "approved"),
        "comments": art_comments,
        "articles": tally(&news_status, "published"),
        "exams": tally(&exam_status, "published"),
        "points": { "art": art_points, "news": news_points },
        "redemptions": news_redemptions,
    })))
}
