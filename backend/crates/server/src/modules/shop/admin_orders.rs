//! shop 后台订单：列表 / IDs / 批量导出标记 / 现货导出 / 子订单发货 /
//! CSV 导入物流 / 改状态（状态机+库存回滚+邮件入队）/ 删除（库存回补+券回滚）。

use std::collections::HashMap;

use axum::extract::{Multipart, Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Value};

use super::common::*;
use super::email;
use super::orders::fetch_order_pub;
use super::pricing::normalize_coupon_code;
use super::products::bad;
use crate::state::AppState;

type Shop = sqlx::Pool<sqlx::Sqlite>;

// ============================================================
// GET /orders（Read）：分页列表 + 子订单
// ============================================================
pub async fn list_orders(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let status = q.get("status").cloned().unwrap_or_else(|| "all".into());
    let keyword = sanitize_config_text(q.get("keyword").map(|s| s.as_str()).unwrap_or(""), 80);
    let sort_by = q.get("sortBy").cloned().unwrap_or_else(|| "created_at".into());
    let sort_dir = if q.get("sortDir").map(|s| s.to_lowercase()) == Some("asc".into()) {
        "ASC"
    } else {
        "DESC"
    };
    let page = parse_int_or(q.get("page").map(|s| s.as_str()), 1).max(1);
    let page_size = parse_int_or(q.get("pageSize").map(|s| s.as_str()), 20).clamp(1, 100);

    let order_by = match sort_by.as_str() {
        "created_at" | "total" | "status" | "id" => sort_by.as_str(),
        _ => "created_at",
    };

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();
    if status != "all" {
        let n: i64 = status.parse().unwrap_or(-99);
        if !ORDER_STATUS_VALUES.contains(&n) {
            return Ok(bad("status 参数无效"));
        }
        conditions.push("CAST(status AS INTEGER) = ?".into());
        params.push(n.to_string());
    }
    if !keyword.is_empty() {
        conditions.push("(id LIKE ? OR contactName LIKE ? OR contactPhone LIKE ? OR mergeMeta LIKE ?)".into());
        let term = format!("%{keyword}%");
        for _ in 0..4 {
            params.push(term.clone());
        }
    }
    if q.get("hasPresale").map(|s| s.as_str()) == Some("1") {
        conditions.push("CAST(hasPresaleItems AS INTEGER) = 1".into());
    }
    if q.get("hasSpot").map(|s| s.as_str()) == Some("1") {
        conditions.push("CAST(hasSpotItems AS INTEGER) = 1".into());
    }
    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let offset = (page - 1) * page_size;

    let count_sql = format!("SELECT COUNT(*) FROM orders {where_sql}");
    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        cq = cq.bind(p);
    }
    let total: i64 = cq.fetch_one(&state.pools.shop).await?;
    let total_pages = if total > 0 { (total + page_size - 1) / page_size } else { 1 };

    let list_sql = format!(
        "{ORDER_SELECT} {where_sql} ORDER BY {order_by} {sort_dir}, id DESC LIMIT ? OFFSET ?"
    );
    let mut lq = sqlx::query_as::<_, OrderRow>(&list_sql);
    for p in &params {
        lq = lq.bind(p);
    }
    lq = lq.bind(page_size).bind(offset);
    let rows: Vec<OrderRow> = lq.fetch_all(&state.pools.shop).await?;

    // 批量加载子订单
    let order_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let mut subs_by_order: HashMap<String, Vec<SubOrderRow>> = HashMap::new();
    if !order_ids.is_empty() {
        let ph = order_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sub_sql = format!(
            "SELECT id, orderId, subKey, label, items, trackingCompany, trackingNo, CAST(shipped AS TEXT) AS shipped, shipped_at FROM sub_orders WHERE orderId IN ({ph}) ORDER BY id ASC"
        );
        let mut sq = sqlx::query_as::<_, SubOrderWithOrderId>(&sub_sql);
        for oid in &order_ids {
            sq = sq.bind(oid);
        }
        let sub_rows: Vec<SubOrderWithOrderId> = sq.fetch_all(&state.pools.shop).await?;
        for sr in sub_rows {
            subs_by_order.entry(sr.order_id.clone()).or_default().push(sr.into_sub_order_row());
        }
    }

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            let subs = subs_by_order.get(&r.id).map(|v| v.as_slice());
            map_order_row(r, subs, None)
        })
        .collect();

    Ok(Json(json!({
        "items": items,
        "pagination": { "page": page, "pageSize": page_size, "total": total, "totalPages": total_pages },
        "sort": { "by": order_by, "dir": sort_dir.to_lowercase() },
        "filters": { "status": status, "keyword": keyword },
    }))
    .into_response())
}

#[derive(sqlx::FromRow)]
struct SubOrderWithOrderId {
    id: i64,
    #[sqlx(rename = "orderId")]
    order_id: String,
    #[sqlx(rename = "subKey")]
    sub_key: String,
    label: String,
    items: Option<String>,
    #[sqlx(rename = "trackingCompany")]
    tracking_company: Option<String>,
    #[sqlx(rename = "trackingNo")]
    tracking_no: Option<String>,
    shipped: Option<String>,
    shipped_at: Option<String>,
}

impl SubOrderWithOrderId {
    fn into_sub_order_row(self) -> SubOrderRow {
        SubOrderRow {
            id: self.id,
            sub_key: self.sub_key,
            label: self.label,
            items: self.items,
            tracking_company: self.tracking_company,
            tracking_no: self.tracking_no,
            shipped: self.shipped,
            shipped_at: self.shipped_at,
        }
    }
}

// ============================================================
// GET /orders/ids（Read）：跨页批量勾选用 ID 列表
// ============================================================
pub async fn list_order_ids(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let status = q.get("status").cloned().unwrap_or_else(|| "all".into());
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<i64> = Vec::new();
    if status != "all" {
        let n: i64 = status.parse().unwrap_or(-99);
        if !ORDER_STATUS_VALUES.contains(&n) {
            return Ok(bad("status 参数无效"));
        }
        conditions.push("CAST(status AS INTEGER) = ?".into());
        params.push(n);
    }
    if let Some(exp) = q.get("exported").filter(|s| !s.is_empty()) {
        conditions.push("CAST(exported AS INTEGER) = ?".into());
        params.push(if exp.parse::<f64>().unwrap_or(0.0) != 0.0 { 1 } else { 0 });
    }
    if let Some(sp) = q.get("spotExported").filter(|s| !s.is_empty()) {
        conditions.push("CAST(spotExported AS INTEGER) = ?".into());
        params.push(if sp.parse::<f64>().unwrap_or(0.0) != 0.0 { 1 } else { 0 });
    }
    if let Some(hp) = q.get("hasPresale").filter(|s| !s.is_empty()) {
        conditions.push("CAST(hasPresaleItems AS INTEGER) = ?".into());
        params.push(if hp.parse::<f64>().unwrap_or(0.0) != 0.0 { 1 } else { 0 });
    }
    let not_fully_exported = q.get("notFullyExported").map(|s| s.as_str()) == Some("1");
    let select_fields = if not_fully_exported {
        "id, items, CAST(exported AS INTEGER) AS exported, CAST(spotExported AS INTEGER) AS spotExported, presaleExportedProducts, CAST(hasPresaleItems AS INTEGER) AS hasPresaleItems, CAST(hasSpotItems AS INTEGER) AS hasSpotItems"
    } else {
        "id"
    };
    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    if !not_fully_exported {
        let sql = format!("SELECT {select_fields} FROM orders {where_sql}");
        let mut query = sqlx::query_scalar::<_, String>(&sql);
        for p in &params {
            query = query.bind(p);
        }
        let ids: Vec<String> = query.fetch_all(&state.pools.shop).await?;
        return Ok(Json(json!({ "ids": ids })).into_response());
    }

    // notFullyExported：应用层判断未完全导出
    let sql = format!("SELECT {select_fields} FROM orders {where_sql}");
    let mut query = sqlx::query_as::<_, NotFullyRow>(&sql);
    for p in &params {
        query = query.bind(p);
    }
    let rows: Vec<NotFullyRow> = query.fetch_all(&state.pools.shop).await?;
    let mut ids: Vec<String> = Vec::new();
    for row in rows {
        let items = safe_parse(row.items.as_deref(), json!([]));
        let items_arr: Vec<Value> = items.as_array().cloned().unwrap_or_default();
        let has_spot = items_arr.iter().any(|i| !truthy(i.get("isPresale").unwrap_or(&Value::Null)));
        let presale_ids: std::collections::HashSet<i64> = items_arr
            .iter()
            .filter(|i| truthy(i.get("isPresale").unwrap_or(&Value::Null)))
            .map(|i| num(i.get("id").unwrap_or(&Value::Null)) as i64)
            .collect();
        let exported_presale = safe_parse(row.presale_exported_products.as_deref(), json!([]));
        let exported_set: std::collections::HashSet<i64> = exported_presale
            .as_array()
            .map(|a| a.iter().map(|v| num(v) as i64).collect())
            .unwrap_or_default();
        let spot_done = !has_spot || row.spot_exported.unwrap_or(0) != 0 || row.exported.unwrap_or(0) != 0;
        let presale_done = presale_ids.is_empty() || presale_ids.iter().all(|pid| exported_set.contains(pid));
        if !(spot_done && presale_done) {
            ids.push(row.id);
        }
    }
    Ok(Json(json!({ "ids": ids })).into_response())
}

#[derive(sqlx::FromRow)]
struct NotFullyRow {
    id: String,
    items: Option<String>,
    exported: Option<i64>,
    #[sqlx(rename = "spotExported")]
    spot_exported: Option<i64>,
    #[sqlx(rename = "presaleExportedProducts")]
    presale_exported_products: Option<String>,
    #[allow(dead_code)]
    #[sqlx(rename = "hasPresaleItems")]
    has_presale_items: Option<i64>,
    #[allow(dead_code)]
    #[sqlx(rename = "hasSpotItems")]
    has_spot_items: Option<i64>,
}

// ============================================================
// PUT /orders/mark-exported（Write）
// ============================================================
pub async fn mark_exported(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;
    let ids = match body.get("ids").and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a.clone(),
        _ => return Ok(bad("请提供订单 ID 列表")),
    };
    let id_strs: Vec<String> = ids.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    let ph = id_strs.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("UPDATE orders SET exported = 1 WHERE id IN ({ph})");
    let mut query = sqlx::query(&sql);
    for id in &id_strs {
        query = query.bind(id);
    }
    query.execute(&state.pools.shop).await?;
    Ok(Json(json!({ "success": true, "count": ids.len() })).into_response())
}

// ============================================================
// PUT /orders/mark-spot-exported（Write）
// ============================================================
pub async fn mark_spot_exported(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;
    let ids = match body.get("ids").and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a.clone(),
        _ => return Ok(bad("请提供订单 ID 列表")),
    };
    for id_v in &ids {
        let order_id = match id_v.as_str() {
            Some(s) => s,
            None => continue,
        };
        let row: Option<(Option<String>, Option<String>)> =
            sqlx::query_as("SELECT items, presaleExportedProducts FROM orders WHERE id = ?")
                .bind(order_id)
                .fetch_optional(&state.pools.shop)
                .await?;
        let (items_text, exported_text) = match row {
            Some(r) => r,
            None => continue,
        };
        let items = safe_parse(items_text.as_deref(), json!([]));
        let items_arr: Vec<Value> = items.as_array().cloned().unwrap_or_default();
        let exported_presale = safe_parse(exported_text.as_deref(), json!([]));
        let exported_set: std::collections::HashSet<i64> = exported_presale
            .as_array()
            .map(|a| a.iter().map(|v| num(v) as i64).collect())
            .unwrap_or_default();
        let all_presale_ids: std::collections::HashSet<i64> = items_arr
            .iter()
            .filter(|i| truthy(i.get("isPresale").unwrap_or(&Value::Null)))
            .map(|i| num(i.get("id").unwrap_or(&Value::Null)) as i64)
            .collect();
        let presale_done = all_presale_ids.is_empty() || all_presale_ids.iter().all(|pid| exported_set.contains(pid));
        let all_exported = if presale_done { 1 } else { 0 };
        sqlx::query("UPDATE orders SET spotExported = 1, exported = CASE WHEN ? = 1 THEN 1 ELSE exported END WHERE id = ?")
            .bind(all_exported)
            .bind(order_id)
            .execute(&state.pools.shop)
            .await?;
    }
    Ok(Json(json!({ "success": true, "count": ids.len() })).into_response())
}

// ============================================================
// PUT /orders/mark-presale-exported（Write）
// ============================================================
pub async fn mark_presale_exported(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;
    let ids = body.get("ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let product_ids = body.get("productIds").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if ids.is_empty() || product_ids.is_empty() {
        return Ok(bad("请提供订单 ID 列表和商品 ID 列表"));
    }
    let new_pids: Vec<i64> = product_ids.iter().map(|v| num(v) as i64).collect();

    for id_v in &ids {
        let order_id = match id_v.as_str() {
            Some(s) => s,
            None => continue,
        };
        let row: Option<(Option<String>, Option<String>, Option<i64>, Option<i64>)> = sqlx::query_as(
            "SELECT items, presaleExportedProducts, CAST(spotExported AS INTEGER), CAST(exported AS INTEGER) FROM orders WHERE id = ?",
        )
        .bind(order_id)
        .fetch_optional(&state.pools.shop)
        .await?;
        let (items_text, exported_text, spot_exported, exported) = match row {
            Some(r) => r,
            None => continue,
        };
        let existing = safe_parse(exported_text.as_deref(), json!([]));
        let mut merged_set: std::collections::HashSet<i64> = existing
            .as_array()
            .map(|a| a.iter().map(|v| num(v) as i64).collect())
            .unwrap_or_default();
        // 保持插入顺序：先 existing 再 new
        let mut merged_ordered: Vec<i64> = existing
            .as_array()
            .map(|a| a.iter().map(|v| num(v) as i64).collect())
            .unwrap_or_default();
        for pid in &new_pids {
            if merged_set.insert(*pid) {
                merged_ordered.push(*pid);
            }
        }

        let items = safe_parse(items_text.as_deref(), json!([]));
        let items_arr: Vec<Value> = items.as_array().cloned().unwrap_or_default();
        let all_presale_ids: std::collections::HashSet<i64> = items_arr
            .iter()
            .filter(|i| truthy(i.get("isPresale").unwrap_or(&Value::Null)))
            .map(|i| num(i.get("id").unwrap_or(&Value::Null)) as i64)
            .collect();
        let has_spot = items_arr.iter().any(|i| !truthy(i.get("isPresale").unwrap_or(&Value::Null)));
        let spot_done = !has_spot || spot_exported.unwrap_or(0) != 0 || exported.unwrap_or(0) != 0;
        let presale_done = all_presale_ids.iter().all(|pid| merged_set.contains(pid));
        let all_exported = if spot_done && presale_done { 1 } else { 0 };

        sqlx::query("UPDATE orders SET presaleExportedProducts = ?, exported = CASE WHEN ? = 1 THEN 1 ELSE exported END WHERE id = ?")
            .bind(serde_json::to_string(&merged_ordered).unwrap_or_else(|_| "[]".into()))
            .bind(all_exported)
            .bind(order_id)
            .execute(&state.pools.shop)
            .await?;
    }
    Ok(Json(json!({ "success": true, "count": ids.len() })).into_response())
}

// ============================================================
// GET /orders/spot-export-data（Read）
// ============================================================
pub async fn spot_export_data(State(state): State<AppState>, user: AuthUser) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;
    let rows: Vec<OrderRow> = sqlx::query_as(
        &format!("{ORDER_SELECT} WHERE CAST(status AS INTEGER) = 2 AND CAST(exported AS INTEGER) = 0 AND CAST(spotExported AS INTEGER) = 0 ORDER BY created_at DESC"),
    )
    .fetch_all(&state.pools.shop)
    .await?;

    let mut result: Vec<Value> = Vec::new();
    for row in &rows {
        let items = safe_parse(row.items.as_deref(), json!([]));
        let items_arr: Vec<Value> = items.as_array().cloned().unwrap_or_default();
        let spot_items: Vec<Value> = items_arr.into_iter().filter(|i| !truthy(i.get("isPresale").unwrap_or(&Value::Null))).collect();
        if spot_items.is_empty() {
            continue;
        }
        result.push(map_order_row(row, None, Some(Value::Array(spot_items))));
    }
    Ok(Json(json!({ "orders": result })).into_response())
}

// ============================================================
// PUT /orders/:id/sub-orders/:subKey/ship（Write）：子订单发货（事务）
// ============================================================
pub async fn ship_sub_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, sub_key)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;
    let order_id = id.trim().to_string();
    let sub_key = sub_key.trim().to_string();
    let tracking_company = body.get("trackingCompany").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tracking_no = body.get("trackingNo").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if order_id.is_empty() || sub_key.is_empty() {
        return Ok(bad("参数不完整"));
    }

    let mut tx = state.pools.shop.begin().await?;
    let order: Option<(Option<String>,)> =
        sqlx::query_as("SELECT CAST(status AS TEXT) FROM orders WHERE id = ?")
            .bind(&order_id)
            .fetch_optional(&mut *tx)
            .await?;
    let order = match order {
        Some(o) => o,
        None => {
            let _ = tx.rollback().await;
            return Ok(super::products::not_found("订单不存在"));
        }
    };
    if order.0.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0) != 2 {
        let _ = tx.rollback().await;
        return Ok(bad("订单状态不允许发货"));
    }

    let sub: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT CAST(shipped AS INTEGER) FROM sub_orders WHERE orderId = ? AND subKey = ?")
            .bind(&order_id)
            .bind(&sub_key)
            .fetch_optional(&mut *tx)
            .await?;
    let sub = match sub {
        Some(s) => s,
        None => {
            let _ = tx.rollback().await;
            return Ok(super::products::not_found("子订单不存在"));
        }
    };
    if sub.0.unwrap_or(0) != 0 {
        let _ = tx.rollback().await;
        return Ok(bad("该子订单已发货"));
    }

    // 更新子订单
    let mut set_parts = vec!["shipped = 1".to_string(), "shipped_at = CURRENT_TIMESTAMP".to_string()];
    if !tracking_company.is_empty() {
        set_parts.push("trackingCompany = ?".into());
    }
    if !tracking_no.is_empty() {
        set_parts.push("trackingNo = ?".into());
    }
    let upd_sql = format!("UPDATE sub_orders SET {} WHERE orderId = ? AND subKey = ?", set_parts.join(", "));
    let mut upd = sqlx::query(&upd_sql);
    if !tracking_company.is_empty() {
        upd = upd.bind(&tracking_company);
    }
    if !tracking_no.is_empty() {
        upd = upd.bind(&tracking_no);
    }
    upd = upd.bind(&order_id).bind(&sub_key);
    upd.execute(&mut *tx).await?;

    let unshipped: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sub_orders WHERE orderId = ? AND CAST(shipped AS INTEGER) = 0")
        .bind(&order_id)
        .fetch_one(&mut *tx)
        .await?;

    if unshipped == 0 {
        let last: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT trackingCompany, trackingNo FROM sub_orders WHERE orderId = ? ORDER BY shipped_at DESC LIMIT 1",
        )
        .bind(&order_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some((Some(tc), Some(tn))) = last {
            if !tc.is_empty() && !tn.is_empty() {
                sqlx::query("UPDATE orders SET status = 3, trackingCompany = ?, trackingNo = ? WHERE id = ?")
                    .bind(&tc)
                    .bind(&tn)
                    .bind(&order_id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query("UPDATE orders SET status = 3 WHERE id = ?").bind(&order_id).execute(&mut *tx).await?;
            }
        } else {
            sqlx::query("UPDATE orders SET status = 3 WHERE id = ?").bind(&order_id).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;

    // 入队子订单发货邮件
    enqueue_sub_order_email_safe(&state, &order_id, &sub_key).await;

    let all_subs: Vec<SubOrderRow> = sqlx::query_as(
        "SELECT id, subKey, label, items, trackingCompany, trackingNo, CAST(shipped AS TEXT) AS shipped, shipped_at FROM sub_orders WHERE orderId = ? ORDER BY id ASC",
    )
    .bind(&order_id)
    .fetch_all(&state.pools.shop)
    .await?;
    let sub_list: Vec<Value> = all_subs
        .iter()
        .map(|s| {
            json!({
                "subKey": s.sub_key,
                "label": s.label,
                "shipped": if s.shipped.as_deref().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0) != 0.0 { 1 } else { 0 },
                "trackingCompany": s.tracking_company.clone().unwrap_or_default(),
                "trackingNo": s.tracking_no.clone().unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "allShipped": unshipped == 0,
        "subOrders": sub_list,
    }))
    .into_response())
}

/// enqueueSubOrderEmailSafely：取订单 + 子订单 items/物流，入队 order_shipped。
async fn enqueue_sub_order_email_safe(state: &AppState, order_id: &str, sub_key: &str) {
    let order_row = match fetch_order_pub(&state.pools.shop, order_id).await {
        Ok(Some(r)) => r,
        _ => return,
    };
    let sub: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT items, trackingCompany, trackingNo FROM sub_orders WHERE orderId = ? AND subKey = ?",
    )
    .bind(order_id)
    .bind(sub_key)
    .fetch_optional(&state.pools.shop)
    .await
    .ok()
    .flatten();
    let (items_text, tc, tn) = match sub {
        Some(s) => s,
        None => return,
    };
    let order_json = map_order_row(&order_row, None, None);
    let sub_items = safe_parse(items_text.as_deref(), json!([]));
    email::enqueue_sub_order_email(
        &state.cfg,
        &state.pools.shop,
        &order_json,
        sub_items,
        tc.as_deref().unwrap_or(""),
        tn.as_deref().unwrap_or(""),
    )
    .await;
}

// ============================================================
// POST /orders/import-tracking（Write）：CSV 导入物流并发货
// ============================================================
pub async fn import_tracking(
    State(state): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let mut csv_bytes: Option<Vec<u8>> = None;
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| haruhi_core::AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| haruhi_core::AppError::bad_request(format!("读取文件失败: {e}")))?;
            csv_bytes = Some(bytes.to_vec());
        } else {
            let _ = field.bytes().await;
        }
    }
    let csv_bytes = match csv_bytes {
        Some(b) => b,
        None => return Ok(bad("请上传 CSV 文件")),
    };
    let csv_content = String::from_utf8_lossy(&csv_bytes).to_string();

    // 切分记录（尊重引号内换行）
    let records = split_csv_records(&csv_content);
    if records.len() < 2 {
        return Ok(bad("CSV 文件格式错误或没有数据"));
    }

    let header = parse_csv_line(&records[0]);
    let col_order_id = header.iter().position(|h| h == "订单号");
    let col_tracking_company = header.iter().position(|h| h == "物流公司");
    let col_tracking_no = header.iter().position(|h| h == "物流单号");
    let col_items = header.iter().position(|h| h.starts_with("商品明细"));

    let (col_order_id, col_tracking_no) = match (col_order_id, col_tracking_no) {
        (Some(a), Some(b)) => (a, b),
        _ => return Ok(bad("CSV 缺少必要的列: 订单号, 物流单号")),
    };

    let mut success = 0_i64;
    let mut skipped = 0_i64;
    let mut errors: Vec<String> = Vec::new();
    let mut details: Vec<String> = Vec::new();

    for rec in records.iter().skip(1) {
        let fields = parse_csv_line(rec);
        let order_id = fields.get(col_order_id).map(|s| s.trim()).unwrap_or("").to_string();
        let tracking_company = col_tracking_company.and_then(|c| fields.get(c)).map(|s| s.trim()).unwrap_or("").to_string();
        let tracking_no = fields.get(col_tracking_no).map(|s| s.trim()).unwrap_or("").to_string();
        let csv_items_text = col_items.and_then(|c| fields.get(c)).map(|s| s.trim()).unwrap_or("").to_string();

        if order_id.is_empty() || tracking_no.is_empty() {
            skipped += 1;
            continue;
        }

        // 取订单
        let order: Option<(Option<String>, Option<String>)> =
            sqlx::query_as("SELECT CAST(status AS TEXT), items FROM orders WHERE id = ?")
                .bind(&order_id)
                .fetch_optional(&state.pools.shop)
                .await?;
        let order = match order {
            Some(o) => o,
            None => {
                errors.push(format!("{order_id}: 订单不存在"));
                continue;
            }
        };
        if order.0.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0) != 2 {
            skipped += 1;
            details.push(format!("{order_id}: 跳过(状态非待发货)"));
            continue;
        }

        let sub_orders: Vec<SubOrderWithOrderId> = sqlx::query_as(
            "SELECT id, orderId, subKey, label, items, trackingCompany, trackingNo, CAST(shipped AS TEXT) AS shipped, shipped_at FROM sub_orders WHERE orderId = ? ORDER BY id ASC",
        )
        .bind(&order_id)
        .fetch_all(&state.pools.shop)
        .await?;

        if sub_orders.is_empty() {
            sqlx::query("UPDATE orders SET trackingCompany = ?, trackingNo = ?, status = 3 WHERE id = ?")
                .bind(&tracking_company)
                .bind(&tracking_no)
                .bind(&order_id)
                .execute(&state.pools.shop)
                .await?;
            enqueue_order_email_safe(&state, "order_shipped", &order_id).await;
            success += 1;
            details.push(format!("{order_id}: 已发货"));
        } else {
            // CSV 商品名解析："名称 x1；名称 x2" → ["名称", "名称"]
            let csv_item_names: Vec<String> = if csv_items_text.is_empty() {
                Vec::new()
            } else {
                csv_items_text
                    .split(['；', ';'])
                    .map(|s| strip_quantity_suffix(s.trim()))
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            let mut matched: Option<&SubOrderWithOrderId> = None;
            if !csv_item_names.is_empty() {
                let mut best_score = 0;
                for sub in &sub_orders {
                    if sub.shipped.as_deref().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0) != 0.0 {
                        continue;
                    }
                    let sub_items = safe_parse(sub.items.as_deref(), json!([]));
                    let sub_names: Vec<String> = sub_items
                        .as_array()
                        .map(|a| a.iter().map(|it| it.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string()).collect())
                        .unwrap_or_default();
                    let csv_in_sub = csv_item_names.iter().filter(|n| sub_names.iter().any(|sn| name_match(sn, n))).count();
                    let sub_in_csv = sub_names.iter().filter(|sn| csv_item_names.iter().any(|n| name_match(sn, n))).count();
                    let score = csv_in_sub + sub_in_csv;
                    if score > best_score {
                        best_score = score;
                        matched = Some(sub);
                    }
                }
            }
            if matched.is_none() {
                let unshipped: Vec<&SubOrderWithOrderId> = sub_orders
                    .iter()
                    .filter(|s| s.shipped.as_deref().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0) == 0.0)
                    .collect();
                if unshipped.len() == 1 {
                    matched = Some(unshipped[0]);
                }
            }
            let matched = match matched {
                Some(m) => m,
                None => {
                    errors.push(format!("{order_id}: 无法匹配子订单包裹"));
                    continue;
                }
            };

            // 内联发货（事务）
            let result = ship_matched_sub(&state.pools.shop, &order_id, &matched.sub_key, &tracking_company, &tracking_no).await;
            match result {
                Ok(()) => {
                    enqueue_sub_order_email_safe(&state, &order_id, &matched.sub_key).await;
                    success += 1;
                    details.push(format!("{order_id}: {} 已发货", matched.label));
                }
                Err(e) => {
                    errors.push(format!("{order_id}: {e}"));
                }
            }
        }
    }

    Ok(Json(json!({
        "success": success,
        "skipped": skipped,
        "errors": errors,
        "details": details,
    }))
    .into_response())
}

async fn ship_matched_sub(
    pool: &Shop,
    order_id: &str,
    sub_key: &str,
    tracking_company: &str,
    tracking_no: &str,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query(
        "UPDATE sub_orders SET shipped = 1, shipped_at = CURRENT_TIMESTAMP, trackingCompany = ?, trackingNo = ? WHERE orderId = ? AND subKey = ?",
    )
    .bind(tracking_company)
    .bind(tracking_no)
    .bind(order_id)
    .bind(sub_key)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let unshipped: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sub_orders WHERE orderId = ? AND CAST(shipped AS INTEGER) = 0")
        .bind(order_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    if unshipped == 0 {
        sqlx::query("UPDATE orders SET status = 3, trackingCompany = ?, trackingNo = ? WHERE id = ?")
            .bind(tracking_company)
            .bind(tracking_no)
            .bind(order_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

fn strip_quantity_suffix(s: &str) -> String {
    // 去掉结尾的 "x数字"（忽略大小写、前后空格），对齐 /\s*x\d+\s*$/i
    let trimmed = s.trim();
    let lower = trimmed.to_lowercase();
    if let Some(pos) = lower.rfind('x') {
        let after = &trimmed[pos + 1..];
        if !after.is_empty() && after.trim().chars().all(|c| c.is_ascii_digit()) {
            // 确认 x 前面允许空格
            return trimmed[..pos].trim_end().to_string();
        }
    }
    trimmed.to_string()
}

fn name_match(a: &str, b: &str) -> bool {
    a == b || a.contains(b) || b.contains(a)
}

/// 按逗号切分一行（处理引号转义 ""），对齐 parseCSVLine。
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if in_quotes {
            if ch == '"' && chars.get(i + 1) == Some(&'"') {
                current.push('"');
                i += 1;
            } else if ch == '"' {
                in_quotes = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
        i += 1;
    }
    fields.push(current.trim().to_string());
    fields
}

/// 按换行切记录（尊重引号内换行），对齐旧逐字符逻辑。
fn split_csv_records(content: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in content.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        }
        if (ch == '\n' || ch == '\r') && !in_quotes {
            if !current.trim().is_empty() {
                records.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        records.push(current.trim().to_string());
    }
    records
}

// ============================================================
// GET /admin/dashboard-summary 在 misc.rs。改状态/删除在此。
// ============================================================

// PUT /orders/:id/status（Moderate）：状态机 + 库存回滚 + 邮件入队（事务）
pub async fn update_order_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Moderate).await?;

    let order_id = id;
    let tracking_company = body.get("trackingCompany").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tracking_no = body.get("trackingNo").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let new_status = num(body.get("status").unwrap_or(&Value::Null));
    if new_status.fract() != 0.0 || !ORDER_STATUS_VALUES.contains(&(new_status as i64)) {
        return Ok(bad("订单状态值无效"));
    }
    let new_status = new_status as i64;

    let mut tx = state.pools.shop.begin().await?;
    let order: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT CAST(status AS TEXT), items FROM orders WHERE id = ?")
            .bind(&order_id)
            .fetch_optional(&mut *tx)
            .await?;
    let order = match order {
        Some(o) => o,
        None => {
            let _ = tx.rollback().await;
            return Ok(super::products::not_found("Order not found"));
        }
    };
    let old_status = order.0.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    let allowed = status_transitions(old_status);
    if new_status != old_status && !allowed.contains(&new_status) {
        let _ = tx.rollback().await;
        return Ok(bad(&format!("非法状态流转: {old_status} -> {new_status}")));
    }

    // 含子订单时阻止直接发货
    if new_status == 3 && old_status == 2 {
        let sub_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sub_orders WHERE orderId = ?")
            .bind(&order_id)
            .fetch_one(&mut *tx)
            .await?;
        if sub_count > 0 {
            let _ = tx.rollback().await;
            return Ok(bad("该订单包含子订单，请通过子订单逐个发货"));
        }
    }

    let mut notify_event = "";
    if new_status != old_status {
        notify_event = match new_status {
            0 => "order_cancelled",
            2 => "order_confirmed",
            3 => "order_shipped",
            4 => "order_completed",
            _ => "",
        };
    }

    // 取消时回补库存（仅现货）
    if new_status == 0 && CANCELLABLE_STATUSES.contains(&old_status) {
        let items = safe_parse(order.1.as_deref(), json!([]));
        if let Some(arr) = items.as_array() {
            for item in arr {
                if truthy(item.get("isPresale").unwrap_or(&Value::Null)) {
                    continue;
                }
                let pid = num(item.get("id").unwrap_or(&Value::Null));
                let qty = num(item.get("quantity").unwrap_or(&Value::Null)) as i64;
                if pid.fract() == 0.0 && qty > 0 {
                    sqlx::query("UPDATE products SET stock = stock + ? WHERE id = ?")
                        .bind(qty)
                        .bind(pid as i64)
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }
    }

    if new_status == 3 && !tracking_company.is_empty() && !tracking_no.is_empty() {
        sqlx::query("UPDATE orders SET status = ?, trackingCompany = ?, trackingNo = ? WHERE id = ?")
            .bind(new_status)
            .bind(&tracking_company)
            .bind(&tracking_no)
            .bind(&order_id)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(new_status)
            .bind(&order_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    if !notify_event.is_empty() {
        enqueue_order_email_safe(&state, notify_event, &order_id).await;
    }

    Ok(Json(json!({ "success": true, "status": new_status })).into_response())
}

// DELETE /orders/:id（Manage）：删除待确认订单（库存回补 + 券回滚，事务）
pub async fn delete_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Manage).await?;

    let order_id = id.trim().to_string();
    if order_id.is_empty() {
        return Ok(bad("订单号不能为空"));
    }

    let mut tx = state.pools.shop.begin().await?;
    let order: Option<(Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT CAST(status AS TEXT), items, couponCode FROM orders WHERE id = ?")
            .bind(&order_id)
            .fetch_optional(&mut *tx)
            .await?;
    let order = match order {
        Some(o) => o,
        None => {
            let _ = tx.rollback().await;
            return Ok(super::products::not_found("订单不存在"));
        }
    };
    if order.0.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0) != 5 {
        let _ = tx.rollback().await;
        return Ok(bad("仅允许删除待确认订单"));
    }

    let items = safe_parse(order.1.as_deref(), json!([]));
    if let Some(arr) = items.as_array() {
        for item in arr {
            if truthy(item.get("isPresale").unwrap_or(&Value::Null)) {
                continue;
            }
            let pid = num(item.get("id").unwrap_or(&Value::Null));
            let qty = num(item.get("quantity").unwrap_or(&Value::Null)) as i64;
            if pid.fract() == 0.0 && qty > 0 {
                sqlx::query("UPDATE products SET stock = stock + ? WHERE id = ?")
                    .bind(qty)
                    .bind(pid as i64)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    if let Some(code) = order.2.as_deref().filter(|s| !s.is_empty()) {
        sqlx::query("UPDATE coupons SET status = ?, usedOrderId = NULL, used_at = NULL WHERE code = ? AND usedOrderId = ? AND status = ?")
            .bind(COUPON_UNUSED)
            .bind(normalize_coupon_code(code))
            .bind(&order_id)
            .bind(COUPON_USED)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM orders WHERE id = ?").bind(&order_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({ "success": true })).into_response())
}

/// enqueueOrderEmailSafely：取订单（mapOrderRow 形态）入队。
async fn enqueue_order_email_safe(state: &AppState, event_key: &str, order_id: &str) {
    if let Some(order_json) = email::load_order_for_email(&state.pools.shop, order_id).await {
        email::enqueue_order_email(&state.cfg, &state.pools.shop, event_key, &order_json).await;
    }
}

