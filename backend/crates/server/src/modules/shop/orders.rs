//! shop 订单：创建（计价/合并/拆单/核销，事务）、查询、改联系信息、提交支付、
//! 后台列表/IDs/导出标记/现货导出/子订单发货/CSV 导入物流/看板、改状态、删除。

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use haruhi_auth::{authorize, decode_token, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Value};

use super::common::*;
use super::email;
use super::pricing::*;
use super::products::{bad, forbidden, not_found};
use crate::state::AppState;

type Shop = sqlx::Pool<sqlx::Sqlite>;

/// 取整张订单。
async fn fetch_order(pool: &Shop, id: &str) -> AppResult<Option<OrderRow>> {
    Ok(sqlx::query_as(&format!("{ORDER_SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

async fn fetch_sub_orders(pool: &Shop, id: &str) -> AppResult<Vec<SubOrderRow>> {
    Ok(sqlx::query_as(&format!("{SUB_ORDER_SELECT} WHERE orderId = ? ORDER BY id ASC"))
        .bind(id)
        .fetch_all(pool)
        .await?)
}

/// 判断请求是否带有效的 shop 后台权限（Bearer JWT + RBAC Read）。
/// 用于公开端点内的“管理员可绕过手机号校验”分支。
async fn is_admin_request(state: &AppState, headers: &HeaderMap) -> bool {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let token = match token {
        Some(t) => t,
        None => return false,
    };
    let claims = match decode_token(&state.cfg.jwt_secret, token) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let user = AuthUser {
        id: claims.sub,
        is_super: claims.is_super,
    };
    authorize(&state.pools.core, &user, "shop", Action::Read).await.is_ok()
}

/// 校验收货信息（对齐 normalizeOrderContactInput）。返回各字段或错误消息。
struct Contact {
    name: String,
    phone: String,
    email: String,
    province: String,
    city: String,
    district: String,
    address_detail: String,
}

fn normalize_contact_input(contact: &Value) -> Result<Contact, String> {
    if !contact.is_object() {
        return Err("收货信息不能为空".into());
    }
    let g = |k: &str| contact.get(k).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let name = g("name");
    let phone = g("phone");
    let email = g("email");
    let province = g("province");
    let city = g("city");
    let district = g("district");
    let address_detail = {
        let a = g("addressDetail");
        if a.is_empty() {
            g("address")
        } else {
            a
        }
    };

    if name.is_empty() {
        return Err("收货人姓名不能为空".into());
    }
    if !is_valid_phone(&phone) {
        return Err("手机号格式错误".into());
    }
    if !is_valid_email(&email) {
        return Err("邮箱格式错误".into());
    }
    if province.is_empty() || city.is_empty() || district.is_empty() {
        return Err("省市区信息不完整".into());
    }
    if address_detail.is_empty() {
        return Err("详细地址不能为空".into());
    }
    Ok(Contact { name, phone, email, province, city, district, address_detail })
}

fn is_valid_phone(s: &str) -> bool {
    // /^1[3-9]\d{9}$/
    let b = s.as_bytes();
    b.len() == 11 && b[0] == b'1' && (b'3'..=b'9').contains(&b[1]) && b[2..].iter().all(|c| c.is_ascii_digit())
}

fn is_valid_email(s: &str) -> bool {
    let v = s.trim();
    let parts: Vec<&str> = v.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || local.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if domain.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    match domain.rsplit_once('.') {
        Some((a, b)) => !a.is_empty() && !b.is_empty(),
        None => false,
    }
}

fn threshold(state: &AppState) -> f64 {
    state.cfg.shop_free_shipping_threshold.max(0) as f64
}

// ============================================================
// POST /orders（公开，下单：事务 + 计价 + 合并 + 拆单 + 核销）
// ============================================================
pub async fn create_order(State(state): State<AppState>, Json(body): Json<Value>) -> AppResult<Response> {
    let thr = threshold(&state);

    // 入参校验
    let id = match body.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return Ok(bad("订单号不能为空")),
    };
    let items = match body.get("items").and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a.clone(),
        _ => return Ok(bad("订单商品不能为空")),
    };
    let contact = match normalize_contact_input(body.get("contact").unwrap_or(&Value::Null)) {
        Ok(c) => c,
        Err(e) => return Ok(bad(&e)),
    };

    // normalizedItems
    let mut normalized_items: Vec<Value> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let pid = num(item.get("id").unwrap_or(&Value::Null));
        let qty = num(item.get("quantity").unwrap_or(&Value::Null));
        if pid.fract() != 0.0 || pid <= 0.0 {
            return Ok(bad(&format!("第 {} 个商品ID无效", index + 1)));
        }
        if qty.fract() != 0.0 || qty <= 0.0 {
            let label = item.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()).unwrap_or_else(|| (pid as i64).to_string());
            return Ok(bad(&format!("商品 {label} 数量无效")));
        }
        let mut it = item.clone();
        if let Some(o) = it.as_object_mut() {
            o.insert("id".into(), json!(pid as i64));
            o.insert("quantity".into(), json!(qty as i64));
        }
        normalized_items.push(it);
    }

    let client_total = num(body.get("total").unwrap_or(&Value::Null));
    if !client_total.is_finite() || client_total < 0.0 {
        return Ok(bad("订单金额无效"));
    }
    let coupon_code = normalize_coupon_code(body.get("couponCode").and_then(|v| v.as_str()).unwrap_or(""));

    let merge_target = body.get("mergeTarget").cloned().unwrap_or(Value::Null);
    let merge_order_id = merge_target.get("orderId").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let merge_phone_last4 = merge_target.get("phoneLast4").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let should_merge = !merge_order_id.is_empty() || !merge_phone_last4.is_empty();
    if should_merge {
        if merge_order_id.is_empty() {
            return Ok(bad("请输入待合并订单号"));
        }
        if merge_phone_last4.len() != 4 || !merge_phone_last4.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(bad("请填写手机号后四位"));
        }
        if merge_order_id == id {
            return Ok(bad("待合并订单号不能与当前订单号相同"));
        }
    }

    let mut tx = state.pools.shop.begin().await?;

    // 取商品
    let unique_ids: Vec<i64> = {
        let mut seen = std::collections::HashSet::new();
        normalized_items
            .iter()
            .filter_map(|i| {
                let id = num(i.get("id").unwrap()) as i64;
                if seen.insert(id) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    };
    let placeholders = unique_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, name, CAST(price AS REAL) AS price, CAST(discountPrice AS REAL) AS discountPrice, \
         CAST(stock AS INTEGER) AS stock, shippingTag, CAST(shippingCost AS REAL) AS shippingCost, \
         presaleMode, CAST(presaleGoalTarget AS INTEGER) AS presaleGoalTarget, presaleFixedDateType, presaleFixedDateValue \
         FROM products WHERE id IN ({placeholders})"
    );
    let mut pq = sqlx::query_as::<_, ProductPriceRow>(&sql);
    for pid in &unique_ids {
        pq = pq.bind(pid);
    }
    let product_rows: Vec<ProductPriceRow> = pq.fetch_all(&mut *tx).await?;
    let mut product_by_id: HashMap<i64, Value> = HashMap::new();
    for r in &product_rows {
        product_by_id.insert(r.id, r.to_value());
    }

    // 数量聚合 + 库存校验
    let mut qty_by_id: Vec<(i64, i64)> = Vec::new();
    {
        let mut map: HashMap<i64, i64> = HashMap::new();
        let mut order: Vec<i64> = Vec::new();
        for it in &normalized_items {
            let pid = num(it.get("id").unwrap()) as i64;
            let q = num(it.get("quantity").unwrap()) as i64;
            if !map.contains_key(&pid) {
                order.push(pid);
            }
            *map.entry(pid).or_insert(0) += q;
        }
        for pid in order {
            qty_by_id.push((pid, map[&pid]));
        }
    }
    for (pid, total_qty) in &qty_by_id {
        let product = match product_by_id.get(pid) {
            Some(p) => p,
            None => {
                let _ = tx.rollback().await;
                return Ok(bad(&format!("商品 {pid} 不存在")));
            }
        };
        let presale = product_presale_snapshot(product);
        if !presale.is_presale && (num(product.get("stock").unwrap_or(&Value::Null)) as i64) < *total_qty {
            let pname = product.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()).unwrap_or_else(|| pid.to_string());
            let _ = tx.rollback().await;
            return Ok(bad(&format!("商品 {pname} 库存不足")));
        }
    }

    // 计价
    let pricing = match calculate_order_pricing(&normalized_items, &product_by_id, thr) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.rollback().await;
            return Ok(bad(&e));
        }
    };

    // 优惠券（当前订单）
    let mut coupon_discount_amount = 0.0_f64;
    let mut applied_coupon: Option<Value> = None;
    if !coupon_code.is_empty() {
        let coupon = super::coupons::get_coupon_by_code_tx(&mut tx, &coupon_code).await?;
        match evaluate_coupon(coupon.as_ref(), pricing.original_total) {
            Err(e) => {
                let _ = tx.rollback().await;
                return Ok(bad(&e));
            }
            Ok(ev) => {
                applied_coupon = Some(ev.coupon);
                coupon_discount_amount = ev.discount_amount;
            }
        }
    }

    let current_order_server_total = round_money((pricing.original_total - coupon_discount_amount).max(0.0));
    if !should_merge && (client_total - current_order_server_total).abs() > 0.01 {
        let _ = tx.rollback().await;
        return Ok(bad("订单金额已变化，请刷新页面后重试"));
    }

    // 合并准备
    let mut final_order_id = id.clone();
    let mut final_priced_items = pricing.priced_items.clone();
    let mut final_original_total = pricing.original_total;
    let mut final_discount_amount = coupon_discount_amount;
    let mut final_server_total = current_order_server_total;
    let mut final_coupon_code: Option<String> = applied_coupon
        .as_ref()
        .and_then(|c| c.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()));
    let mut merge_meta: Value = Value::Null;
    let mut source_order: Option<OrderRow> = None;
    let mut source_coupon_code = String::new();

    if should_merge {
        let src = match fetch_order_tx(&mut tx, &merge_order_id).await? {
            Some(o) => o,
            None => {
                let _ = tx.rollback().await;
                return Ok(bad("待合并订单不存在"));
            }
        };
        let src_status = src.status.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        if !is_self_service_mergeable_source(src_status) {
            let _ = tx.rollback().await;
            return Ok(bad("仅支持合并未发货订单"));
        }
        let expected_last4 = extract_phone_last4(src.contact_phone.as_deref().unwrap_or(""));
        if expected_last4.is_empty() || expected_last4 != merge_phone_last4 {
            let _ = tx.rollback().await;
            return Ok(bad("手机号后四位校验失败"));
        }
        source_coupon_code = normalize_coupon_code(src.coupon_code.as_deref().unwrap_or(""));
        if !source_coupon_code.is_empty() && !coupon_code.is_empty() {
            let _ = tx.rollback().await;
            return Ok(bad("暂不支持两个订单都使用优惠券后再合并"));
        }

        let source_merge_meta = normalize_order_merge_meta(
            src.merge_meta.as_deref().map(|s| Value::String(s.to_string())),
        );
        let source_items_parsed = safe_parse(src.items.as_deref(), json!([]));
        let source_items_arr: Vec<Value> = source_items_parsed.as_array().cloned().unwrap_or_default();
        let source_items = merge_priced_items(&[&source_items_arr]);
        if source_items.is_empty() {
            let _ = tx.rollback().await;
            return Ok(bad("待合并订单商品数据异常，无法合并"));
        }

        let source_pricing = calculate_pricing_from_priced_items(&source_items, thr);
        let source_discount_amount = round_money(src.discount_amount.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0));
        let source_amount = round_money(src.total.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0));
        let merged_items = merge_priced_items(&[&source_items, &pricing.priced_items]);
        let merged_pricing = calculate_pricing_from_priced_items(&merged_items, thr);
        let shipping_adjustment = round_money(merged_pricing.shipping_fee - source_pricing.shipping_fee - pricing.shipping_fee);
        let shipping_saved = round_money((-shipping_adjustment).max(0.0));
        let merge_time = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        final_order_id = generate_unique_order_id_tx(&mut tx).await?;
        final_priced_items = merged_pricing.priced_items.clone();
        final_original_total = merged_pricing.original_total;
        final_discount_amount = round_money(source_discount_amount + coupon_discount_amount);
        final_server_total = round_money((final_original_total - final_discount_amount).max(0.0));
        final_coupon_code = if !source_coupon_code.is_empty() {
            Some(source_coupon_code.clone())
        } else {
            applied_coupon.as_ref().and_then(|c| c.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        };

        // contactChanged
        let source_contact = json!({
            "name": src.contact_name.as_deref().unwrap_or("").trim(),
            "phone": src.contact_phone.as_deref().unwrap_or("").trim(),
            "email": src.contact_email.as_deref().unwrap_or("").trim(),
            "province": src.province.as_deref().unwrap_or("").trim(),
            "city": src.city.as_deref().unwrap_or("").trim(),
            "district": src.district.as_deref().unwrap_or("").trim(),
            "addressDetail": src.address_detail.as_deref().unwrap_or("").trim(),
        });
        let next_contact = json!({
            "name": contact.name, "phone": contact.phone, "email": contact.email,
            "province": contact.province, "city": contact.city, "district": contact.district,
            "addressDetail": contact.address_detail,
        });
        let contact_changed = serde_json::to_string(&source_contact).ok() != serde_json::to_string(&next_contact).ok();

        // parts
        let source_parts: Vec<Value> = match source_merge_meta.get("parts").and_then(|v| v.as_array()) {
            Some(arr) if !arr.is_empty() => arr
                .iter()
                .map(|part| {
                    let status_before = {
                        let raw = part.get("statusBeforeMerge");
                        let n = num(raw.unwrap_or(&Value::Null));
                        if n.fract() == 0.0 && raw.map(|v| !v.is_null()).unwrap_or(false) {
                            json!(n as i64)
                        } else {
                            Value::Null
                        }
                    };
                    json!({
                        "orderId": part.get("orderId").and_then(|v| v.as_str()).unwrap_or("").trim(),
                        "amount": round_money(num(part.get("amount").unwrap_or(&Value::Null))),
                        "statusBeforeMerge": status_before,
                    })
                })
                .filter(|p| !p.get("orderId").and_then(|v| v.as_str()).unwrap_or("").is_empty())
                .collect(),
            _ => vec![json!({
                "orderId": src.id,
                "amount": source_amount,
                "statusBeforeMerge": src_status,
            })],
        };

        let mut parts_order: Vec<String> = Vec::new();
        let mut parts_map: HashMap<String, Value> = HashMap::new();
        for part in &source_parts {
            let oid = part.get("orderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Some(existing) = parts_map.get_mut(&oid) {
                let amt = round_money(num(existing.get("amount").unwrap()) + num(part.get("amount").unwrap()));
                existing["amount"] = json!(amt);
            } else {
                parts_order.push(oid.clone());
                parts_map.insert(oid, part.clone());
            }
        }
        // 加入当前提交订单 id
        if let Some(existing) = parts_map.get_mut(&id) {
            let amt = round_money(num(existing.get("amount").unwrap()) + current_order_server_total);
            existing["amount"] = json!(amt);
        } else {
            parts_order.push(id.clone());
            parts_map.insert(id.clone(), json!({ "orderId": id, "amount": current_order_server_total, "statusBeforeMerge": 1 }));
        }
        let parts: Vec<Value> = parts_order.iter().filter_map(|k| parts_map.get(k).cloned()).collect();

        // history
        let mut merged_history: Vec<Value> = match source_merge_meta.get("history").and_then(|v| v.as_array()) {
            Some(arr) => arr
                .iter()
                .filter_map(|row| {
                    let s = row.get("sourceOrderId").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                    let a = {
                        let x = row.get("appendedOrderId").and_then(|v| v.as_str()).unwrap_or("").trim();
                        if x.is_empty() { row.get("submittedOrderId").and_then(|v| v.as_str()).unwrap_or("").trim() } else { x }
                    }.to_string();
                    let n = {
                        let x = row.get("newOrderId").and_then(|v| v.as_str()).unwrap_or("").trim();
                        if x.is_empty() { row.get("mergedOrderId").and_then(|v| v.as_str()).unwrap_or("").trim() } else { x }
                    }.to_string();
                    if s.is_empty() || a.is_empty() || n.is_empty() {
                        return None;
                    }
                    let src_ship = num(row.get("sourceShippingFee").unwrap_or(&Value::Null));
                    let app_ship = num(row.get("appendedShippingFee").unwrap_or(&Value::Null));
                    let mer_ship = num(row.get("mergedShippingFee").unwrap_or(&Value::Null));
                    let ship_adj = if row.get("shippingAdjustment").map(|v| !v.is_null()).unwrap_or(false) {
                        round_money(num(row.get("shippingAdjustment").unwrap()))
                    } else {
                        round_money(mer_ship - src_ship - app_ship)
                    };
                    let app_amt = num(row.get("appendedAmount").unwrap_or(&Value::Null));
                    let incr = if row.get("incrementalPayable").map(|v| !v.is_null()).unwrap_or(false) {
                        round_money(num(row.get("incrementalPayable").unwrap()))
                    } else {
                        round_money((app_amt + ship_adj).max(0.0))
                    };
                    Some(json!({
                        "sourceOrderId": s, "appendedOrderId": a, "newOrderId": n,
                        "sourceAmount": round_money(num(row.get("sourceAmount").unwrap_or(&Value::Null))),
                        "appendedAmount": round_money(app_amt),
                        "mergedAmount": round_money(num(row.get("mergedAmount").unwrap_or(&Value::Null))),
                        "sourceShippingFee": round_money(src_ship),
                        "appendedShippingFee": round_money(app_ship),
                        "mergedShippingFee": round_money(mer_ship),
                        "shippingAdjustment": ship_adj,
                        "incrementalPayable": incr,
                        "shippingSaved": round_money(num(row.get("shippingSaved").unwrap_or(&Value::Null))),
                        "mergedAt": row.get("mergedAt").and_then(|v| v.as_str()).map(|x| x.to_string()),
                    }))
                })
                .collect(),
            None => Vec::new(),
        };
        let current_history = json!({
            "sourceOrderId": src.id,
            "appendedOrderId": id,
            "newOrderId": final_order_id,
            "sourceAmount": source_amount,
            "appendedAmount": current_order_server_total,
            "mergedAmount": final_server_total,
            "sourceShippingFee": source_pricing.shipping_fee,
            "appendedShippingFee": pricing.shipping_fee,
            "mergedShippingFee": merged_pricing.shipping_fee,
            "shippingAdjustment": shipping_adjustment,
            "incrementalPayable": round_money((current_order_server_total + shipping_adjustment).max(0.0)),
            "shippingSaved": shipping_saved,
            "mergedAt": merge_time,
        });
        merged_history.push(current_history);

        merge_meta = json!({
            "mergedOrderId": final_order_id,
            "sourceOrderId": src.id,
            "submittedOrderId": id,
            "sourceAmount": source_amount,
            "appendedAmount": current_order_server_total,
            "mergedAmount": final_server_total,
            "sourceShippingFee": source_pricing.shipping_fee,
            "appendedShippingFee": pricing.shipping_fee,
            "mergedShippingFee": merged_pricing.shipping_fee,
            "shippingAdjustment": shipping_adjustment,
            "incrementalPayable": round_money((current_order_server_total + shipping_adjustment).max(0.0)),
            "shippingSaved": shipping_saved,
            "contactPolicy": "new_order",
            "contactChanged": contact_changed,
            "mergedAt": merge_time,
            "mergeCount": merged_history.len(),
            "parts": parts,
            "history": merged_history,
        });

        source_order = Some(src);
    }

    if should_merge && (client_total - final_server_total).abs() > 0.01 {
        let _ = tx.rollback().await;
        return Ok(bad("合并后订单金额已变化，请重新校验后重试"));
    }

    // 扣库存（仅现货）
    for (pid, total_qty) in &qty_by_id {
        if let Some(product) = product_by_id.get(pid) {
            let presale = product_presale_snapshot(product);
            if presale.is_presale {
                continue;
            }
            sqlx::query("UPDATE products SET stock = stock - ? WHERE id = ?")
                .bind(total_qty)
                .bind(pid)
                .execute(&mut *tx)
                .await?;
        }
    }

    // 拆单标记
    let presale_items: Vec<Value> = final_priced_items.iter().filter(|i| truthy(i.get("isPresale").unwrap_or(&Value::Null))).cloned().collect();
    let spot_items: Vec<Value> = final_priced_items.iter().filter(|i| !truthy(i.get("isPresale").unwrap_or(&Value::Null))).cloned().collect();
    let has_presale_flag = if !presale_items.is_empty() { 1 } else { 0 };
    let has_spot_flag = if !spot_items.is_empty() { 1 } else { 0 };
    let is_mixed = !presale_items.is_empty() && !spot_items.is_empty();

    // 插入订单（唯一约束冲突→幂等返回已存在订单）
    let insert_res = sqlx::query(
        "INSERT INTO orders (id, total, originalTotal, discountAmount, couponCode, mergeMeta, items, contactName, contactPhone, contactEmail, province, city, district, addressDetail, status, hasPresaleItems, hasSpotItems) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&final_order_id)
    .bind(final_server_total)
    .bind(final_original_total)
    .bind(final_discount_amount)
    .bind(&final_coupon_code)
    .bind(if merge_meta.is_null() { None } else { Some(serde_json::to_string(&merge_meta).unwrap_or_default()) })
    .bind(serde_json::to_string(&final_priced_items).unwrap_or_else(|_| "[]".into()))
    .bind(&contact.name)
    .bind(&contact.phone)
    .bind(&contact.email)
    .bind(&contact.province)
    .bind(&contact.city)
    .bind(&contact.district)
    .bind(&contact.address_detail)
    .bind(has_presale_flag)
    .bind(has_spot_flag)
    .execute(&mut *tx)
    .await;

    if let Err(e) = insert_res {
        // 唯一约束 → 幂等返回已存在订单（仅非合并场景，对齐旧实现以 id 查）
        if let sqlx::Error::Database(db) = &e {
            if db.is_unique_violation() {
                let _ = tx.rollback().await;
                let existing: Option<(String, Option<String>, Option<String>, Option<String>, Option<String>)> =
                    sqlx::query_as("SELECT id, CAST(total AS TEXT), CAST(originalTotal AS TEXT), CAST(discountAmount AS TEXT), couponCode FROM orders WHERE id = ?")
                        .bind(&id)
                        .fetch_optional(&state.pools.shop)
                        .await?;
                if let Some((oid, total, otot, disc, code)) = existing {
                    return Ok(Json(json!({
                        "success": true,
                        "orderId": oid,
                        "total": round_money(total.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)),
                        "originalTotal": round_money(otot.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)),
                        "discountAmount": round_money(disc.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)),
                        "couponCode": code,
                    }))
                    .into_response());
                }
                return Ok(super::products::conflict("订单号已存在，请重新提交订单"));
            }
        }
        let _ = tx.rollback().await;
        return Err(e.into());
    }

    // 子订单（混合订单）
    if is_mixed {
        sqlx::query("INSERT INTO sub_orders (orderId, subKey, label, items) VALUES (?, ?, ?, ?)")
            .bind(&final_order_id)
            .bind("spot")
            .bind("现货包裹")
            .bind(serde_json::to_string(&spot_items).unwrap_or_else(|_| "[]".into()))
            .execute(&mut *tx)
            .await?;
        for pi in &presale_items {
            let pid = num(pi.get("id").unwrap_or(&Value::Null)) as i64;
            let sub_key = format!("presale-{pid}");
            let name = pi.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| format!("商品{pid}"));
            let label = format!("预售: {name}");
            sqlx::query("INSERT INTO sub_orders (orderId, subKey, label, items) VALUES (?, ?, ?, ?)")
                .bind(&final_order_id)
                .bind(&sub_key)
                .bind(&label)
                .bind(serde_json::to_string(&vec![pi.clone()]).unwrap_or_else(|_| "[]".into()))
                .execute(&mut *tx)
                .await?;
        }
    }

    // 源订单券归属转移
    if let Some(src) = &source_order {
        if !source_coupon_code.is_empty() {
            sqlx::query("UPDATE coupons SET usedOrderId = ? WHERE code = ? AND usedOrderId = ? AND status = ?")
                .bind(&final_order_id)
                .bind(&source_coupon_code)
                .bind(&src.id)
                .bind(COUPON_USED)
                .execute(&mut *tx)
                .await?;
        }
    }

    // 核销当前订单券（必须恰好命中一行）
    if let Some(coupon) = &applied_coupon {
        let cid = coupon.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let res = sqlx::query("UPDATE coupons SET status = ?, usedOrderId = ?, used_at = CURRENT_TIMESTAMP WHERE id = ? AND status = ?")
            .bind(COUPON_USED)
            .bind(&final_order_id)
            .bind(cid)
            .bind(COUPON_UNUSED)
            .execute(&mut *tx)
            .await?;
        if res.rows_affected() != 1 {
            let _ = tx.rollback().await;
            return Ok(bad("优惠券已被使用，请更换后重试"));
        }
    }

    // 删除源订单
    if let Some(src) = &source_order {
        sqlx::query("DELETE FROM orders WHERE id = ?").bind(&src.id).execute(&mut *tx).await?;
    }

    tx.commit().await?;

    // 入队邮件（事务外）
    if let Some(order_json) = email::load_order_for_email(&state.pools.shop, &final_order_id).await {
        email::enqueue_order_email(&state.cfg, &state.pools.shop, "order_created", &order_json).await;
    }

    Ok(Json(json!({
        "success": true,
        "orderId": final_order_id,
        "total": final_server_total,
        "originalTotal": final_original_total,
        "discountAmount": final_discount_amount,
        "couponCode": final_coupon_code,
        "mergeMeta": merge_meta,
    }))
    .into_response())
}

/// 事务内取订单。
async fn fetch_order_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: &str,
) -> AppResult<Option<OrderRow>> {
    Ok(sqlx::query_as(&format!("{ORDER_SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?)
}

/// buildOrderId + 唯一性重试（事务内）。
async fn generate_unique_order_id_tx(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> AppResult<String> {
    for _ in 0..10 {
        let candidate = build_order_id();
        let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM orders WHERE id = ?")
            .bind(&candidate)
            .fetch_optional(&mut **tx)
            .await?;
        if exists.is_none() {
            return Ok(candidate);
        }
    }
    Err(haruhi_core::AppError::internal("生成新订单号失败，请稍后重试"))
}

fn build_order_id() -> String {
    use rand::Rng;
    let now = chrono::Local::now();
    let date_part = now.format("%Y%m%d").to_string();
    let time_part = now.format("%H%M%S%3f").to_string();
    let rand_part = format!("{:04}", rand::thread_rng().gen_range(0..10000));
    format!("SOS-{date_part}-{time_part}-{rand_part}")
}

/// 商品计价行。
#[derive(sqlx::FromRow)]
struct ProductPriceRow {
    id: i64,
    name: Option<String>,
    price: Option<f64>,
    #[sqlx(rename = "discountPrice")]
    discount_price: Option<f64>,
    stock: Option<i64>,
    #[sqlx(rename = "shippingTag")]
    shipping_tag: Option<String>,
    #[sqlx(rename = "shippingCost")]
    shipping_cost: Option<f64>,
    #[sqlx(rename = "presaleMode")]
    presale_mode: Option<String>,
    #[sqlx(rename = "presaleGoalTarget")]
    presale_goal_target: Option<i64>,
    #[sqlx(rename = "presaleFixedDateType")]
    presale_fixed_date_type: Option<String>,
    #[sqlx(rename = "presaleFixedDateValue")]
    presale_fixed_date_value: Option<String>,
}

impl ProductPriceRow {
    fn to_value(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "price": self.price,
            "discountPrice": self.discount_price,
            "stock": self.stock,
            "shippingTag": self.shipping_tag,
            "shippingCost": self.shipping_cost,
            "presaleMode": self.presale_mode,
            "presaleGoalTarget": self.presale_goal_target,
            "presaleFixedDateType": self.presale_fixed_date_type,
            "presaleFixedDateValue": self.presale_fixed_date_value,
        })
    }
}

// ============================================================
// GET /orders/:id（公开，手机号后四位校验；管理员可绕过）
// ============================================================
pub async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    let row = match fetch_order(&state.pools.shop, &id).await? {
        Some(r) => r,
        None => return Ok(not_found("订单不存在")),
    };
    let subs = fetch_sub_orders(&state.pools.shop, &id).await?;
    let order = map_order_row(&row, Some(&subs), None);

    if !is_admin_request(&state, &headers).await {
        let phone_last4 = q.get("phoneLast4").map(|s| s.trim().to_string()).unwrap_or_default();
        if phone_last4.len() != 4 || !phone_last4.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(bad("请填写手机号后四位"));
        }
        let expected = extract_phone_last4(row.contact_phone.as_deref().unwrap_or(""));
        if expected.is_empty() || phone_last4 != expected {
            return Ok(forbidden("手机号后四位校验失败"));
        }
    }
    Ok(Json(order).into_response())
}

// ============================================================
// PUT /orders/:id/contact（公开 + 管理员；改收货信息）
// ============================================================
pub async fn update_order_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let order_id = id.trim().to_string();
    if order_id.is_empty() {
        return Ok(bad("订单号不能为空"));
    }
    let existing = match fetch_order(&state.pools.shop, &order_id).await? {
        Some(r) => r,
        None => return Ok(not_found("订单不存在")),
    };

    let is_admin = is_admin_request(&state, &headers).await;
    if !is_admin {
        let phone_last4 = body
            .get("phoneLast4")
            .and_then(|v| v.as_str())
            .or_else(|| q.get("phoneLast4").map(|s| s.as_str()))
            .unwrap_or("")
            .trim()
            .to_string();
        if phone_last4.len() != 4 || !phone_last4.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(bad("请填写手机号后四位"));
        }
        let expected = extract_phone_last4(existing.contact_phone.as_deref().unwrap_or(""));
        if expected.is_empty() || phone_last4 != expected {
            return Ok(forbidden("手机号后四位校验失败"));
        }
        let status = existing.status.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        if !is_self_service_contact_editable(status) {
            return Ok(bad("订单已发货，仅支持联系管理员修改收货信息"));
        }
    }

    let normalized = match normalize_contact_input(body.get("contact").unwrap_or(&Value::Null)) {
        Ok(c) => c,
        Err(e) => return Ok(bad(&e)),
    };
    sqlx::query(
        "UPDATE orders SET contactName=?, contactPhone=?, contactEmail=?, province=?, city=?, district=?, addressDetail=? WHERE id=?",
    )
    .bind(&normalized.name)
    .bind(&normalized.phone)
    .bind(&normalized.email)
    .bind(&normalized.province)
    .bind(&normalized.city)
    .bind(&normalized.district)
    .bind(&normalized.address_detail)
    .bind(&order_id)
    .execute(&state.pools.shop)
    .await?;

    let updated = match fetch_order(&state.pools.shop, &order_id).await? {
        Some(r) => r,
        None => return Ok(not_found("订单不存在")),
    };
    Ok(Json(map_order_row(&updated, None, None)).into_response())
}

// ============================================================
// POST /orders/:id/payment（公开，待付款1 -> 待确认5，事务）
// ============================================================
pub async fn submit_payment(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    let mut tx = state.pools.shop.begin().await?;
    let order: Option<(Option<String>,)> = sqlx::query_as("SELECT CAST(status AS TEXT) FROM orders WHERE id = ?")
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await?;
    let order = match order {
        Some(o) => o,
        None => {
            let _ = tx.rollback().await;
            return Ok(not_found("订单不存在"));
        }
    };
    let old_status = order.0.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    if old_status == 5 {
        tx.commit().await?;
        return Ok(Json(json!({ "success": true, "status": 5 })).into_response());
    }
    if old_status != 1 {
        let _ = tx.rollback().await;
        return Ok(bad("当前订单状态不允许提交支付"));
    }
    sqlx::query("UPDATE orders SET status = 5 WHERE id = ?").bind(&id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({ "success": true, "status": 5 })).into_response())
}

// 后台订单端点（列表/导出/发货/改状态/删除）位于 admin_orders.rs（拆分以控制单文件长度）。

/// 供 admin_orders 复用：取整张订单。
pub async fn fetch_order_pub(pool: &Shop, id: &str) -> AppResult<Option<OrderRow>> {
    fetch_order(pool, id).await
}
