//! shop 优惠券：公开预查 + 后台列表/批量生成/状态/删除。

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

use super::common::*;
use super::pricing::*;
use super::products::{bad, conflict, not_found};
use crate::state::AppState;

/// 把 coupons 行（SELECT *）读为 JSON（松类型）。
#[derive(sqlx::FromRow)]
struct CouponRow {
    id: i64,
    code: Option<String>,
    name: Option<String>,
    #[sqlx(rename = "batchNo")]
    batch_no: Option<String>,
    #[sqlx(rename = "minSpend")]
    min_spend: Option<String>,
    #[sqlx(rename = "discountType")]
    discount_type: Option<String>,
    #[sqlx(rename = "discountValue")]
    discount_value: Option<String>,
    #[sqlx(rename = "maxDiscount")]
    max_discount: Option<String>,
    status: Option<String>,
    #[sqlx(rename = "expiresAt")]
    expires_at: Option<String>,
    #[sqlx(rename = "usedOrderId")]
    used_order_id: Option<String>,
    used_at: Option<String>,
    created_at: Option<String>,
}

fn coupon_row_to_value(r: &CouponRow) -> Value {
    json!({
        "id": r.id,
        "code": r.code,
        "name": r.name,
        "batchNo": r.batch_no,
        "minSpend": r.min_spend.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0),
        "discountType": r.discount_type,
        "discountValue": r.discount_value.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0),
        "maxDiscount": r.max_discount.as_deref().filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok()),
        "status": r.status.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
        "expiresAt": r.expires_at,
        "usedOrderId": r.used_order_id,
        "used_at": r.used_at,
        "created_at": r.created_at,
    })
}

/// 公开按 code 查券（非事务）。
pub async fn get_coupon_by_code(pool: &sqlx::SqlitePool, code: &str) -> AppResult<Option<Value>> {
    let row: Option<CouponRow> = sqlx::query_as(&format!("{COUPON_SELECT} WHERE code = ?"))
        .bind(normalize_coupon_code(code))
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(coupon_row_to_value))
}

/// 事务内按 code 查券（下单核销用）。
pub async fn get_coupon_by_code_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    code: &str,
) -> AppResult<Option<Value>> {
    let row: Option<CouponRow> = sqlx::query_as(&format!("{COUPON_SELECT} WHERE code = ?"))
        .bind(normalize_coupon_code(code))
        .fetch_optional(&mut **tx)
        .await?;
    Ok(row.as_ref().map(coupon_row_to_value))
}

// POST /coupons/preview（公开）
pub async fn preview_coupon(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let code = normalize_coupon_code(body.get("code").and_then(|v| v.as_str()).unwrap_or(""));
    let order_amount = num(body.get("orderAmount").unwrap_or(&Value::Null));

    if code.is_empty() {
        return Ok(bad("请输入优惠券码"));
    }
    if !order_amount.is_finite() || order_amount <= 0.0 {
        return Ok(bad("订单金额无效"));
    }

    let coupon = get_coupon_by_code(&state.pools.shop, &code).await?;
    match evaluate_coupon(coupon.as_ref(), order_amount) {
        Err(e) => Ok(bad(&e)),
        Ok(result) => {
            let c = &result.coupon;
            Ok(Json(json!({
                "valid": true,
                "code": c.get("code"),
                "name": c.get("name"),
                "minSpend": c.get("minSpend"),
                "discountType": c.get("discountType"),
                "discountValue": c.get("discountValue"),
                "maxDiscount": c.get("maxDiscount"),
                "benefitText": coupon_benefit_text(c),
                "orderAmount": round_money(order_amount),
                "discountAmount": result.discount_amount,
                "payableAmount": result.payable_amount,
            }))
            .into_response())
        }
    }
}

// GET /admin/coupons（Read）：分页列表
pub async fn list_coupons(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Read).await?;

    let status = q.get("status").cloned().unwrap_or_else(|| "all".into());
    let batch_no = sanitize_config_text(q.get("batchNo").map(|s| s.as_str()).unwrap_or(""), 80);
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
        if ![COUPON_DISABLED, COUPON_UNUSED, COUPON_USED].contains(&n) {
            return Ok(bad("status 参数无效"));
        }
        conditions.push("status = ?".into());
        params.push(n.to_string());
    }
    if !batch_no.is_empty() {
        conditions.push("batchNo = ?".into());
        params.push(batch_no.clone());
    }
    if !keyword.is_empty() {
        conditions.push("(code LIKE ? OR name LIKE ?)".into());
        params.push(format!("%{keyword}%"));
        params.push(format!("%{keyword}%"));
    }
    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let order_by = match sort_by.as_str() {
        "created_at" | "expiresAt" | "minSpend" | "status" | "id" => sort_by.as_str(),
        _ => "created_at",
    };
    let offset = (page - 1) * page_size;

    let count_sql = format!("SELECT COUNT(*) FROM coupons {where_sql}");
    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        cq = cq.bind(p);
    }
    let total: i64 = cq.fetch_one(&state.pools.shop).await?;
    let total_pages = if total > 0 {
        (total + page_size - 1) / page_size
    } else {
        1
    };

    let list_sql = format!(
        "{COUPON_SELECT} {where_sql} ORDER BY {order_by} {sort_dir}, id DESC LIMIT ? OFFSET ?"
    );
    let mut lq = sqlx::query_as::<_, CouponRow>(&list_sql);
    for p in &params {
        lq = lq.bind(p);
    }
    lq = lq.bind(page_size).bind(offset);
    let rows: Vec<CouponRow> = lq.fetch_all(&state.pools.shop).await?;

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            let coupon = format_coupon(&coupon_row_to_value(r));
            let mut obj: Map<String, Value> = coupon.as_object().cloned().unwrap_or_default();
            obj.insert("benefitText".into(), json!(coupon_benefit_text(&coupon)));
            Value::Object(obj)
        })
        .collect();

    Ok(Json(json!({
        "items": items,
        "pagination": { "page": page, "pageSize": page_size, "total": total, "totalPages": total_pages },
        "sort": { "by": order_by, "dir": sort_dir.to_lowercase() },
        "filters": { "status": status, "batchNo": batch_no, "keyword": keyword },
    }))
    .into_response())
}

// POST /admin/coupons/batch（Manage）：批量生成
pub async fn batch_coupons(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Manage).await?;

    let quantity = num(body.get("quantity").unwrap_or(&Value::Null));
    let prefix = normalize_coupon_prefix(body.get("prefix").and_then(|v| v.as_str()).unwrap_or(""));
    let custom_batch = sanitize_config_text(
        body.get("batchNo").and_then(|v| v.as_str()).unwrap_or(""),
        30,
    )
    .to_uppercase();
    let batch_no = if custom_batch.is_empty() {
        generate_batch_no()
    } else {
        custom_batch
    };

    let rule = match normalize_coupon_rule_input(&body) {
        Ok(r) => r,
        Err(e) => return Ok(bad(&e)),
    };
    if quantity.fract() != 0.0 || quantity <= 0.0 || quantity > 2000.0 {
        return Ok(bad("优惠券数量必须为1-2000的整数"));
    }
    let quantity = quantity as i64;

    let mut tx = state.pools.shop.begin().await?;
    let mut generated: Vec<String> = Vec::new();
    for _ in 0..quantity {
        let mut inserted = false;
        for _ in 0..10 {
            let code = generate_coupon_code(&prefix);
            let res = sqlx::query(
                "INSERT INTO coupons (code, name, batchNo, minSpend, discountType, discountValue, maxDiscount, status, expiresAt) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&code)
            .bind(rule.get("name").and_then(|v| v.as_str()))
            .bind(&batch_no)
            .bind(rule.get("minSpend").and_then(|v| v.as_f64()))
            .bind(rule.get("discountType").and_then(|v| v.as_str()))
            .bind(rule.get("discountValue").and_then(|v| v.as_f64()))
            .bind(rule.get("maxDiscount").and_then(|v| v.as_f64()))
            .bind(COUPON_UNUSED)
            .bind(rule.get("expiresAt").and_then(|v| v.as_str()))
            .execute(&mut *tx)
            .await;
            match res {
                Ok(_) => {
                    generated.push(code);
                    inserted = true;
                    break;
                }
                Err(sqlx::Error::Database(db)) if db.is_unique_violation() => continue,
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(e.into());
                }
            }
        }
        if !inserted {
            let _ = tx.rollback().await;
            return Ok((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "优惠券码生成冲突，请重试" })),
            )
                .into_response());
        }
    }
    tx.commit().await?;

    Ok(Json(json!({
        "success": true,
        "batchNo": batch_no,
        "quantity": generated.len(),
        "couponRule": rule,
        "codes": generated,
    }))
    .into_response())
}

// PUT /admin/coupons/:id/status（Write）：禁用/启用
pub async fn set_coupon_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let coupon_id = parse_int_radix10(&id).unwrap_or(0);
    let status = num(body.get("status").unwrap_or(&Value::Null)) as i64;
    if coupon_id <= 0 {
        return Ok(bad("优惠券ID无效"));
    }
    if ![COUPON_DISABLED, COUPON_UNUSED].contains(&status) {
        return Ok(bad("仅支持设置为禁用或可用"));
    }

    let coupon: Option<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, CAST(status AS TEXT) FROM coupons WHERE id = ?")
            .bind(coupon_id)
            .fetch_optional(&state.pools.shop)
            .await?;
    let coupon = match coupon {
        Some(c) => c,
        None => return Ok(not_found("优惠券不存在")),
    };
    let cur_status = coupon
        .1
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if cur_status == COUPON_USED {
        return Ok(bad("已使用优惠券不可更改状态"));
    }

    sqlx::query("UPDATE coupons SET status = ? WHERE id = ?")
        .bind(status)
        .bind(coupon_id)
        .execute(&state.pools.shop)
        .await?;
    Ok(Json(json!({ "success": true, "status": status })).into_response())
}

// DELETE /admin/coupons/:id（Manage）
pub async fn delete_coupon(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Manage).await?;

    let coupon_id = parse_int_radix10(&id).unwrap_or(0);
    if coupon_id <= 0 {
        return Ok(bad("优惠券ID无效"));
    }
    let coupon: Option<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, CAST(status AS TEXT) FROM coupons WHERE id = ?")
            .bind(coupon_id)
            .fetch_optional(&state.pools.shop)
            .await?;
    let coupon = match coupon {
        Some(c) => c,
        None => return Ok(not_found("优惠券不存在")),
    };
    let cur_status = coupon
        .1
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if cur_status == COUPON_USED {
        return Ok(bad("已使用优惠券不可删除"));
    }
    let _ = conflict; // 保留工具引用

    sqlx::query("DELETE FROM coupons WHERE id = ?")
        .bind(coupon_id)
        .execute(&state.pools.shop)
        .await?;
    Ok(Json(json!({ "success": true })).into_response())
}

// ---------- 工具：批次号 / 券码 / 规则校验 ----------

fn normalize_coupon_prefix(prefix: &str) -> String {
    let clean: String = prefix
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        .take(8)
        .collect();
    if clean.is_empty() {
        "CPN".to_string()
    } else {
        clean
    }
}

fn generate_batch_no() -> String {
    // 本地时间 B + YYYYMMDDHHMMSS（对齐旧 generateBatchNo 用 local time）。
    let now = chrono::Local::now();
    format!("B{}", now.format("%Y%m%d%H%M%S"))
}

fn generate_coupon_code(prefix: &str) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: String = (0..4).map(|_| format!("{:02X}", rng.gen::<u8>())).collect();
    let tail: String = {
        // Math.random().toString(36).slice(2,6) 风格：4 位 base36 大写
        const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        (0..4)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect()
    };
    format!("{prefix}-{random}{tail}")
}

/// normalizeCouponRuleInput：返回规则对象或错误。
fn normalize_coupon_rule_input(payload: &Value) -> Result<Value, String> {
    let name = {
        let n = sanitize_config_text(
            payload
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("优惠券"),
            60,
        );
        if n.is_empty() {
            "优惠券".to_string()
        } else {
            n
        }
    };
    let min_spend = num(payload.get("minSpend").unwrap_or(&json!(0)));
    if !min_spend.is_finite() || min_spend < 0.0 {
        return Err("优惠门槛金额无效".into());
    }
    let discount_type = payload
        .get("discountType")
        .and_then(|v| v.as_str())
        .unwrap_or("amount")
        .trim()
        .to_string();
    if !COUPON_DISCOUNT_TYPES.contains(&discount_type.as_str()) {
        return Err("优惠类型无效".into());
    }
    let discount_value = num(payload.get("discountValue").unwrap_or(&Value::Null));
    if !discount_value.is_finite() || discount_value <= 0.0 {
        return Err("优惠力度无效".into());
    }

    let mut max_discount: Value = Value::Null;
    if discount_type == "percent" {
        if discount_value >= 100.0 {
            return Err("折扣比例必须小于100%".into());
        }
        let md = payload.get("maxDiscount");
        let provided = match md {
            None | Some(Value::Null) => false,
            Some(Value::String(s)) if s.is_empty() => false,
            _ => true,
        };
        if provided {
            let m = num(md.unwrap());
            if !m.is_finite() || m <= 0.0 {
                return Err("最高优惠金额无效".into());
            }
            max_discount = json!(round_money(m));
        }
    }

    // expiresAt 解析
    let expires_raw = payload.get("expiresAt");
    let expires_present = match expires_raw {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) if s.is_empty() => false,
        _ => true,
    };
    let expires_iso = if expires_present {
        let s = match expires_raw.unwrap() {
            Value::String(s) => s.clone(),
            v => v.to_string(),
        };
        match parse_datetime_ms(&s) {
            Some(ms) => {
                if ms <= chrono::Utc::now().timestamp_millis() {
                    return Err("过期时间必须晚于当前时间".into());
                }
                // 转 ISO8601（对齐 new Date().toISOString()）
                Some(
                    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
                        .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                        .unwrap_or(s),
                )
            }
            None => return Err("过期时间格式错误".into()),
        }
    } else {
        None
    };

    Ok(json!({
        "name": name,
        "minSpend": round_money(min_spend),
        "discountType": discount_type,
        "discountValue": round_money(discount_value),
        "maxDiscount": max_discount,
        "expiresAt": expires_iso,
    }))
}
