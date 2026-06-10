//! shop 杂项：site-config（支付二维码配置）、联系留言、埋点、统计（销售趋势/商品销量/转化漏斗/看板）。

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Value};

use super::common::*;
use super::products::{bad, not_found};
use crate::state::AppState;

// ============================================================
// site-config
// ============================================================

fn sanitize_config_url(value: &str) -> String {
    let clean = sanitize_config_text(value, 500);
    if clean.is_empty() {
        return String::new();
    }
    let lower = clean.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return clean;
    }
    if clean.starts_with(&format!("{UPLOAD_URL_PREFIX}/")) {
        return clean;
    }
    if let Some(rest) = clean.strip_prefix(LEGACY_API_UPLOADS_PREFIX) {
        return format!("{UPLOAD_URL_PREFIX}/{rest}");
    }
    String::new()
}

/// normalizeSiteConfig：保证 payment.{wechatQr,alipayQr,friendQr} 三字段，URL 经过校验。
fn normalize_site_config(raw: &Value) -> Value {
    let payment = raw.get("payment").cloned().unwrap_or(json!({}));
    json!({
        "payment": {
            "wechatQr": sanitize_config_url(payment.get("wechatQr").and_then(|v| v.as_str()).unwrap_or("")),
            "alipayQr": sanitize_config_url(payment.get("alipayQr").and_then(|v| v.as_str()).unwrap_or("")),
            "friendQr": sanitize_config_url(payment.get("friendQr").and_then(|v| v.as_str()).unwrap_or("")),
        }
    })
}

async fn load_site_config(pool: &sqlx::SqlitePool) -> AppResult<Value> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT value FROM site_settings WHERE key = ?")
            .bind("site_config")
            .fetch_optional(pool)
            .await?;
    let parsed = safe_parse(row.and_then(|r| r.0).as_deref(), Value::Null);
    Ok(normalize_site_config(&parsed))
}

async fn save_site_config(pool: &sqlx::SqlitePool, config: &Value) -> AppResult<Value> {
    let normalized = normalize_site_config(config);
    sqlx::query(
        "INSERT INTO site_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
    )
    .bind("site_config")
    .bind(serde_json::to_string(&normalized).unwrap_or_else(|_| "{}".into()))
    .execute(pool)
    .await?;
    Ok(normalized)
}

// GET /site-config（公开）
pub async fn get_site_config(State(state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(load_site_config(&state.pools.shop).await?))
}

// GET /admin/site-config（Read）
pub async fn admin_get_site_config(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;
    Ok(Json(load_site_config(&state.pools.shop).await?))
}

// PUT /admin/site-config（Manage）
pub async fn admin_put_site_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "shop", Action::Manage).await?;
    let config = save_site_config(&state.pools.shop, &body).await?;
    Ok(Json(json!({ "success": true, "config": config })))
}

// ============================================================
// 联系留言
// ============================================================

// POST /contact/messages（公开）
pub async fn create_contact_message(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let name = sanitize_config_text(body.get("name").and_then(|v| v.as_str()).unwrap_or(""), 60);
    let contact = sanitize_config_text(
        body.get("contact").and_then(|v| v.as_str()).unwrap_or(""),
        80,
    );
    let order_id = sanitize_config_text(
        body.get("orderId").and_then(|v| v.as_str()).unwrap_or(""),
        60,
    );
    let content = sanitize_config_text(
        body.get("content").and_then(|v| v.as_str()).unwrap_or(""),
        2000,
    );

    if name.is_empty() {
        return Ok(bad("请填写您的称呼"));
    }
    if contact.is_empty() {
        return Ok(bad("请填写联系方式"));
    }
    if content.is_empty() {
        return Ok(bad("请填写留言内容"));
    }

    let res = sqlx::query(
        "INSERT INTO contact_messages (name, contact, orderId, content, status) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(&name)
    .bind(&contact)
    .bind(if order_id.is_empty() { None } else { Some(order_id) })
    .bind(&content)
    .execute(&state.pools.shop)
    .await?;
    Ok(Json(json!({ "success": true, "id": res.last_insert_rowid() })).into_response())
}

#[derive(sqlx::FromRow)]
struct ContactRow {
    id: i64,
    name: Option<String>,
    contact: Option<String>,
    #[sqlx(rename = "orderId")]
    order_id: Option<String>,
    content: Option<String>,
    status: Option<String>,
    handled_at: Option<String>,
    created_at: Option<String>,
}

// GET /admin/contact-messages（Read）
pub async fn list_contact_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let status = q.get("status").cloned().unwrap_or_else(|| "all".into());
    let keyword = sanitize_config_text(q.get("keyword").map(|s| s.as_str()).unwrap_or(""), 80);
    let sort_by = q
        .get("sortBy")
        .cloned()
        .unwrap_or_else(|| "created_at".into());
    let sort_dir = if q.get("sortDir").map(|s| s.to_lowercase()) == Some("asc".into()) {
        "ASC"
    } else {
        "DESC"
    };
    let page = parse_int_or(q.get("page").map(|s| s.as_str()), 1).max(1);
    let page_size = parse_int_or(q.get("pageSize").map(|s| s.as_str()), 20).clamp(1, 100);

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();
    if status != "all" {
        let n: i64 = status.parse().unwrap_or(-99);
        if ![0, 1].contains(&n) {
            return Ok(bad("status 参数无效"));
        }
        conditions.push("status = ?".into());
        params.push(n.to_string());
    }
    if !keyword.is_empty() {
        conditions
            .push("(name LIKE ? OR contact LIKE ? OR orderId LIKE ? OR content LIKE ?)".into());
        let term = format!("%{keyword}%");
        for _ in 0..4 {
            params.push(term.clone());
        }
    }
    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let order_by = match sort_by.as_str() {
        "created_at" | "status" | "id" | "handled_at" => sort_by.as_str(),
        _ => "created_at",
    };
    let offset = (page - 1) * page_size;

    let count_sql = format!("SELECT COUNT(*) FROM contact_messages {where_sql}");
    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        cq = cq.bind(p);
    }
    let total: i64 = cq.fetch_one(&state.pools.shop).await?;

    let list_sql = format!(
        "{CONTACT_SELECT} {where_sql} ORDER BY {order_by} {sort_dir}, id DESC LIMIT ? OFFSET ?"
    );
    let mut lq = sqlx::query_as::<_, ContactRow>(&list_sql);
    for p in &params {
        lq = lq.bind(p);
    }
    lq = lq.bind(page_size).bind(offset);
    let rows: Vec<ContactRow> = lq.fetch_all(&state.pools.shop).await?;

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "contact": r.contact,
                "orderId": r.order_id,
                "content": r.content,
                "status": r.status.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
                "handled_at": r.handled_at,
                "created_at": r.created_at,
            })
        })
        .collect();

    Ok(Json(json!({
        "items": items,
        "pagination": crate::pagination::page_meta(page, page_size, total),
        "sort": { "by": order_by, "dir": sort_dir.to_lowercase() },
        "filters": { "status": status, "keyword": keyword },
    }))
    .into_response())
}

// PUT /admin/contact-messages/:id/status（Write）
pub async fn set_contact_message_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let message_id = parse_int_radix10(&id).unwrap_or(0);
    let status = num(body.get("status").unwrap_or(&Value::Null)) as i64;
    if message_id <= 0 {
        return Ok(bad("留言ID无效"));
    }
    if ![0, 1].contains(&status) {
        return Ok(bad("状态仅支持 0 或 1"));
    }

    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM contact_messages WHERE id = ?")
        .bind(message_id)
        .fetch_optional(&state.pools.shop)
        .await?;
    if existing.is_none() {
        return Ok(not_found("留言不存在"));
    }

    sqlx::query(
        "UPDATE contact_messages SET status = ?, handled_at = CASE WHEN ? = 1 THEN CURRENT_TIMESTAMP ELSE NULL END WHERE id = ?",
    )
    .bind(status)
    .bind(status)
    .bind(message_id)
    .execute(&state.pools.shop)
    .await?;
    Ok(Json(json!({ "success": true, "status": status })).into_response())
}

// ============================================================
// 埋点
// ============================================================

// POST /analytics/event（公开）
pub async fn analytics_event(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let event_key = body
        .get("eventKey")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().chars().take(80).collect::<String>())
        .unwrap_or_default();
    if event_key.is_empty() {
        return Ok(bad("eventKey 不能为空"));
    }

    let session_id = match body.get("sessionId").and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim().chars().take(120).collect::<String>(),
        _ => "anonymous".to_string(),
    };
    let page = body
        .get("page")
        .and_then(|v| v.as_str())
        .map(|s| s.chars().take(200).collect::<String>())
        .unwrap_or_default();
    let meta = match body.get("meta") {
        Some(m) if m.is_object() => serde_json::to_string(m)
            .unwrap_or_else(|_| "{}".into())
            .chars()
            .take(2000)
            .collect::<String>(),
        _ => "{}".to_string(),
    };

    sqlx::query(
        "INSERT INTO analytics_events (sessionId, eventKey, page, meta) VALUES (?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(&event_key)
    .bind(&page)
    .bind(&meta)
    .execute(&state.pools.shop)
    .await?;
    Ok(Json(json!({ "success": true })).into_response())
}

// ============================================================
// 统计：看板 / 销售趋势 / 商品销量 / 转化漏斗
// ============================================================

// GET /admin/dashboard-summary（Read）
pub async fn dashboard_summary(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let pending_verify: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE CAST(status AS INTEGER) = 5")
            .fetch_one(&state.pools.shop)
            .await?;
    let pending_shipment: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE CAST(status AS INTEGER) = 2")
            .fetch_one(&state.pools.shop)
            .await?;
    let total_orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&state.pools.shop)
        .await?;
    let total_sales: Option<f64> = sqlx::query_scalar(&format!(
        "SELECT CAST(COALESCE(SUM(CAST(total AS REAL)), 0) AS REAL) FROM orders WHERE CAST(status AS INTEGER) IN ({})",
        placeholders(SALES_VALID_STATUSES.len())
    ))
    .bind(SALES_VALID_STATUSES[0])
    .bind(SALES_VALID_STATUSES[1])
    .bind(SALES_VALID_STATUSES[2])
    .bind(SALES_VALID_STATUSES[3])
    .fetch_one(&state.pools.shop)
    .await?;

    Ok(Json(json!({
        "pendingVerify": pending_verify,
        "pendingShipment": pending_shipment,
        "totalSales": round_money(total_sales.unwrap_or(0.0)),
        "totalOrders": total_orders,
    })))
}

fn placeholders(n: usize) -> String {
    std::iter::repeat("?")
        .take(n)
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------- 日期工具（对齐 server.cjs 的 getToday/parseDateInput/buildDateRange/resolveStatsRange）----------

fn today_local() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn format_date(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn parse_date_input(value: &str) -> Option<chrono::NaiveDate> {
    if value.len() != 10 {
        return None;
    }
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn build_date_range(start: &str, end: &str) -> Vec<String> {
    let mut list = Vec::new();
    let (s, e) = match (parse_date_input(start), parse_date_input(end)) {
        (Some(s), Some(e)) => (s, e),
        _ => return list,
    };
    if s > e {
        return list;
    }
    let mut cur = s;
    while cur <= e {
        list.push(format_date(cur));
        match cur.succ_opt() {
            Some(next) if next > cur => cur = next,
            _ => break, // 到达日期上限，避免死循环
        }
        if list.len() > 100_000 {
            break; // 兜底防御
        }
    }
    list
}

struct StatsRange {
    period: String,
    start_date: Option<String>,
    end_date: Option<String>,
}

fn resolve_stats_range(
    period: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<StatsRange, String> {
    let selected = period.unwrap_or("30").to_string();
    let today = today_local();

    if selected == "all" {
        return Ok(StatsRange {
            period: "all".into(),
            start_date: None,
            end_date: None,
        });
    }
    if selected == "custom" {
        let s = parse_date_input(start_date.unwrap_or("")).ok_or("自定义时间格式错误")?;
        let e = parse_date_input(end_date.unwrap_or("")).ok_or("自定义时间格式错误")?;
        if s > e {
            return Err("开始日期不能晚于结束日期".into());
        }
        return Ok(StatsRange {
            period: "custom".into(),
            start_date: Some(format_date(s)),
            end_date: Some(format_date(e)),
        });
    }
    let day_span: i64 = selected.parse().unwrap_or(-1);
    if ![7, 30, 90].contains(&day_span) {
        return Err("period 参数无效".into());
    }
    let start = today - chrono::Duration::days(day_span - 1);
    Ok(StatsRange {
        period: day_span.to_string(),
        start_date: Some(format_date(start)),
        end_date: Some(format_date(today)),
    })
}

fn period_range_for_product_report(period: Option<&str>) -> Result<StatsRange, String> {
    let p = period.unwrap_or("week").to_string();
    let today = today_local();
    if p == "all" {
        return Ok(StatsRange {
            period: "all".into(),
            start_date: None,
            end_date: None,
        });
    }
    if p == "week" {
        // 周一为一周起始（对齐旧：day===0?7:day，start - (day-1)）
        let weekday = today.weekday().num_days_from_sunday() as i64; // Sun=0..Sat=6
        let day = if weekday == 0 { 7 } else { weekday };
        let start = today - chrono::Duration::days(day - 1);
        return Ok(StatsRange {
            period: "week".into(),
            start_date: Some(format_date(start)),
            end_date: Some(format_date(today)),
        });
    }
    if p == "month" {
        let start =
            chrono::NaiveDate::from_ymd_opt(today.year_ce().1 as i32, today.month0() + 1, 1)
                .unwrap_or(today);
        return Ok(StatsRange {
            period: "month".into(),
            start_date: Some(format_date(start)),
            end_date: Some(format_date(today)),
        });
    }
    Err("period 参数无效".into())
}

use chrono::Datelike;

// GET /admin/stats/sales-trend（Read）
pub async fn sales_trend(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let range = match resolve_stats_range(
        q.get("period").map(|s| s.as_str()),
        q.get("startDate").map(|s| s.as_str()),
        q.get("endDate").map(|s| s.as_str()),
    ) {
        Ok(r) => r,
        Err(e) => return Ok(bad(&e)),
    };

    let ph = placeholders(SALES_VALID_STATUSES.len());
    let (mut start_date, mut end_date) = (range.start_date.clone(), range.end_date.clone());

    if range.period == "all" {
        let span: (Option<String>, Option<String>) = sqlx::query_as(&format!(
            "SELECT MIN(DATE(created_at)), MAX(DATE(created_at)) FROM orders WHERE CAST(status AS INTEGER) IN ({ph})"
        ))
        .bind(SALES_VALID_STATUSES[0]).bind(SALES_VALID_STATUSES[1]).bind(SALES_VALID_STATUSES[2]).bind(SALES_VALID_STATUSES[3])
        .fetch_one(&state.pools.shop)
        .await?;
        match span {
            (Some(s), e) => {
                start_date = Some(s);
                end_date = e;
            }
            _ => {
                return Ok(Json(json!({
                    "period": "all", "startDate": null, "endDate": null, "points": [],
                    "summary": { "salesAmount": 0, "orderCount": 0, "avgOrderAmount": 0 }
                }))
                .into_response());
            }
        }
    }

    let sd = start_date.clone().unwrap_or_default();
    let ed = end_date.clone().unwrap_or_default();

    let rows: Vec<(Option<String>, Option<f64>, i64)> = {
        let sql = format!(
            "SELECT DATE(created_at) AS day, SUM(CAST(total AS REAL)) AS salesAmount, COUNT(*) AS orderCount \
             FROM orders WHERE CAST(status AS INTEGER) IN ({ph}) AND DATE(created_at) BETWEEN DATE(?) AND DATE(?) \
             GROUP BY DATE(created_at) ORDER BY DATE(created_at) ASC"
        );
        sqlx::query_as(&sql)
            .bind(SALES_VALID_STATUSES[0])
            .bind(SALES_VALID_STATUSES[1])
            .bind(SALES_VALID_STATUSES[2])
            .bind(SALES_VALID_STATUSES[3])
            .bind(&sd)
            .bind(&ed)
            .fetch_all(&state.pools.shop)
            .await?
    };
    let mut row_map: HashMap<String, (f64, i64)> = HashMap::new();
    for (day, amt, cnt) in rows {
        if let Some(d) = day {
            row_map.insert(d, (amt.unwrap_or(0.0), cnt));
        }
    }

    let mut points = Vec::new();
    let mut sum_sales = 0.0_f64;
    let mut sum_count = 0_i64;
    for day in build_date_range(&sd, &ed) {
        let (amt, cnt) = row_map.get(&day).copied().unwrap_or((0.0, 0));
        sum_sales += amt;
        sum_count += cnt;
        points.push(json!({ "date": day, "salesAmount": amt, "orderCount": cnt }));
    }
    let avg = if sum_count > 0 {
        round_money(sum_sales / sum_count as f64)
    } else {
        0.0
    };

    Ok(Json(json!({
        "period": range.period,
        "startDate": start_date,
        "endDate": end_date,
        "points": points,
        "summary": { "salesAmount": sum_sales, "orderCount": sum_count, "avgOrderAmount": avg },
    }))
    .into_response())
}

// GET /admin/stats/product-sales（Read）
pub async fn product_sales(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let range = match period_range_for_product_report(q.get("period").map(|s| s.as_str())) {
        Ok(r) => r,
        Err(e) => return Ok(bad(&e)),
    };

    let ph = placeholders(SALES_VALID_STATUSES.len());
    let rows: Vec<Option<String>> = if range.period != "all" {
        let sql = format!(
            "SELECT items FROM orders WHERE CAST(status AS INTEGER) IN ({ph}) AND DATE(created_at) BETWEEN DATE(?) AND DATE(?)"
        );
        sqlx::query_scalar(&sql)
            .bind(SALES_VALID_STATUSES[0])
            .bind(SALES_VALID_STATUSES[1])
            .bind(SALES_VALID_STATUSES[2])
            .bind(SALES_VALID_STATUSES[3])
            .bind(range.start_date.clone().unwrap_or_default())
            .bind(range.end_date.clone().unwrap_or_default())
            .fetch_all(&state.pools.shop)
            .await?
    } else {
        let sql = format!("SELECT items FROM orders WHERE CAST(status AS INTEGER) IN ({ph})");
        sqlx::query_scalar(&sql)
            .bind(SALES_VALID_STATUSES[0])
            .bind(SALES_VALID_STATUSES[1])
            .bind(SALES_VALID_STATUSES[2])
            .bind(SALES_VALID_STATUSES[3])
            .fetch_all(&state.pools.shop)
            .await?
    };

    // 聚合（保持插入顺序的稳定排序后再用 quantity/amount 排）
    let mut order: Vec<String> = Vec::new();
    let mut map: HashMap<String, (Option<i64>, String, i64, f64)> = HashMap::new();
    for items_text in rows {
        let items = safe_parse(items_text.as_deref(), json!([]));
        if let Some(arr) = items.as_array() {
            for item in arr {
                let quantity = num(item.get("quantity").unwrap_or(&Value::Null)) as i64;
                if quantity <= 0 {
                    continue;
                }
                let id_val = item.get("id");
                let key = match id_val {
                    Some(v) if !v.is_null() => format!("id:{}", num(v) as i64),
                    _ => format!(
                        "name:{}",
                        item.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                    ),
                };
                let entry = map.entry(key.clone()).or_insert_with(|| {
                    order.push(key.clone());
                    (
                        id_val.filter(|v| !v.is_null()).map(|v| num(v) as i64),
                        item.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未命名商品")
                            .to_string(),
                        0,
                        0.0,
                    )
                });
                entry.2 += quantity;
                entry.3 += num(item.get("price").unwrap_or(&Value::Null)) * quantity as f64;
            }
        }
    }

    let mut items: Vec<Value> = order
        .iter()
        .filter_map(|k| map.get(k))
        .map(|(pid, name, qty, amt)| {
            json!({ "productId": pid, "name": name, "quantity": qty, "amount": amt })
        })
        .collect();
    // 排序：quantity desc, amount desc
    items.sort_by(|a, b| {
        let qa = a.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
        let qb = b.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
        let aa = a.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let ab = b.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        qb.cmp(&qa)
            .then(ab.partial_cmp(&aa).unwrap_or(std::cmp::Ordering::Equal))
    });

    let total_quantity: i64 = items
        .iter()
        .map(|i| i.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0))
        .sum();
    let total_amount: f64 = items
        .iter()
        .map(|i| i.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
        .sum();

    Ok(Json(json!({
        "period": range.period,
        "startDate": range.start_date,
        "endDate": range.end_date,
        "items": items,
        "totalQuantity": total_quantity,
        "totalAmount": total_amount,
    }))
    .into_response())
}

// GET /admin/stats/conversion（Read）
pub async fn conversion(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let range = match resolve_stats_range(
        q.get("period").map(|s| s.as_str()),
        q.get("startDate").map(|s| s.as_str()),
        q.get("endDate").map(|s| s.as_str()),
    ) {
        Ok(r) => r,
        Err(e) => return Ok(bad(&e)),
    };

    let (mut start_date, mut end_date) = (range.start_date.clone(), range.end_date.clone());

    if range.period == "all" {
        let span: (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT MIN(DATE(created_at)), MAX(DATE(created_at)) FROM analytics_events",
        )
        .fetch_one(&state.pools.shop)
        .await?;
        match span {
            (Some(s), e) => {
                start_date = Some(s);
                end_date = e;
            }
            _ => {
                let empty_steps: Vec<Value> = FUNNEL_STEPS
                    .iter()
                    .enumerate()
                    .map(|(i, (k, l))| json!({ "key": k, "label": l, "visitors": 0, "conversionRate": if i == 0 { 100 } else { 0 } }))
                    .collect();
                return Ok(Json(json!({
                    "period": "all", "startDate": null, "endDate": null,
                    "steps": empty_steps, "overallConversion": 0
                }))
                .into_response());
            }
        }
    }

    let sd = start_date.clone().unwrap_or_default();
    let ed = end_date.clone().unwrap_or_default();

    let keys: Vec<&str> = FUNNEL_STEPS.iter().map(|(k, _)| *k).collect();
    let ph = placeholders(keys.len());
    let sql = format!(
        "SELECT eventKey, COUNT(DISTINCT sessionId) AS visitors FROM analytics_events \
         WHERE eventKey IN ({ph}) AND DATE(created_at) BETWEEN DATE(?) AND DATE(?) GROUP BY eventKey"
    );
    let mut query = sqlx::query_as::<_, (String, i64)>(&sql);
    for k in &keys {
        query = query.bind(*k);
    }
    query = query.bind(&sd).bind(&ed);
    let rows: Vec<(String, i64)> = query.fetch_all(&state.pools.shop).await?;
    let visitor_map: HashMap<String, i64> = rows.into_iter().collect();

    let mut steps = Vec::new();
    for (i, (k, l)) in FUNNEL_STEPS.iter().enumerate() {
        let visitors = *visitor_map.get(*k).unwrap_or(&0);
        let conversion_rate = if i == 0 {
            100.0
        } else {
            let prev = *visitor_map.get(FUNNEL_STEPS[i - 1].0).unwrap_or(&0);
            if prev > 0 {
                round_money((visitors as f64 / prev as f64) * 100.0)
            } else {
                0.0
            }
        };
        steps.push(json!({ "key": k, "label": l, "visitors": visitors, "conversionRate": conversion_rate }));
    }

    let first = steps
        .first()
        .and_then(|s| s.get("visitors"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let last = steps
        .last()
        .and_then(|s| s.get("visitors"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let overall = if first > 0 {
        round_money((last as f64 / first as f64) * 100.0)
    } else {
        0.0
    };

    Ok(Json(json!({
        "period": range.period,
        "startDate": start_date,
        "endDate": end_date,
        "steps": steps,
        "overallConversion": overall,
    }))
    .into_response())
}
