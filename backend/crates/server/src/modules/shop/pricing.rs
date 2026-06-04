//! shop 计价 / 预售 / 优惠券 / 合并元信息 —— 忠实移植 server.cjs 的核心业务逻辑。
//! 金额单位：分（旧实现里用元的两位小数，存库为 REAL/INTEGER；这里保持与旧实现一致，按 f64 处理）。

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::common::{
    num, round_money, safe_parse, truthy, COUPON_DISCOUNT_TYPES, COUPON_UNUSED, PRESALE_FIXED,
    PRESALE_GOAL, PRESALE_NONE,
};

// ============================================================
// 商品价格 / 折扣
// ============================================================

/// toPositivePriceOrNull：>0 的有限数四舍五入两位，否则 None。
pub fn to_positive_price_or_null(v: &Value) -> Option<f64> {
    let n = num(v);
    if !n.is_finite() || n <= 0.0 {
        return None;
    }
    Some((n * 100.0).round() / 100.0)
}

/// normalizeDiscountPrice：折扣价须 >0 且 < 原价，否则 None。
pub fn normalize_discount_price(discount: &Value, raw_price: &Value) -> Option<f64> {
    let price = num(raw_price);
    let discount = to_positive_price_or_null(discount)?;
    if price.is_finite() && price > 0.0 && discount >= price {
        return None;
    }
    Some(discount)
}

/// getEffectiveProductPrice：有有效折扣价用折扣价，否则原价。
pub fn effective_product_price(price: &Value, discount_price: &Value) -> f64 {
    match normalize_discount_price(discount_price, price) {
        Some(d) => round_money(d),
        None => round_money(num(price)),
    }
}

pub fn to_shipping_group_key(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.trim().to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if s.is_empty() {
        "default".to_string()
    } else {
        s
    }
}

// ============================================================
// 预售
// ============================================================

const FIXED_MONTH_START: &str = "month_start";
const FIXED_MONTH_END: &str = "month_end";
const FIXED_DATE: &str = "date";

/// 预售规范化结果。
pub struct PresaleNormalized {
    pub mode: String,
    pub goal_target: i64,
    pub fixed_date_type: String,
    pub fixed_date_value: String,
}

fn is_month_value(s: &str) -> bool {
    // YYYY-MM, 月份 01-12
    let bytes = s.as_bytes();
    if s.len() != 7 || bytes[4] != b'-' {
        return false;
    }
    if !s[0..4].bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    matches!(&s[5..7], m if m.bytes().all(|b| b.is_ascii_digit())) && {
        let mm: i32 = s[5..7].parse().unwrap_or(0);
        (1..=12).contains(&mm)
    }
}

fn is_date_value(s: &str) -> bool {
    // YYYY-MM-DD
    if s.len() != 10 {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    if !s.bytes().enumerate().all(|(i, b)| {
        if i == 4 || i == 7 {
            b == b'-'
        } else {
            b.is_ascii_digit()
        }
    }) {
        return false;
    }
    let mm: i32 = parts[1].parse().unwrap_or(0);
    let dd: i32 = parts[2].parse().unwrap_or(0);
    (1..=12).contains(&mm) && (1..=31).contains(&dd)
}

/// normalizeProductPresaleInput：返回 Ok(规范化) 或 Err(错误消息)。
pub fn normalize_product_presale_input(payload: &Value) -> Result<PresaleNormalized, String> {
    let mode_raw = payload
        .get("presaleMode")
        .and_then(|v| v.as_str())
        .unwrap_or(PRESALE_NONE)
        .trim()
        .to_lowercase();
    let mode = if matches!(
        mode_raw.as_str(),
        PRESALE_NONE | PRESALE_GOAL | PRESALE_FIXED
    ) {
        mode_raw
    } else {
        PRESALE_NONE.to_string()
    };
    let goal_target_raw = super::common::parse_int_radix10(
        &payload
            .get("presaleGoalTarget")
            .map(value_to_str)
            .unwrap_or_default(),
    );
    let goal_target = match goal_target_raw {
        Some(n) if n > 0 => n,
        _ => 0,
    };
    let fixed_date_type_raw = payload
        .get("presaleFixedDateType")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let fixed_date_type = if matches!(
        fixed_date_type_raw.as_str(),
        FIXED_MONTH_START | FIXED_MONTH_END | FIXED_DATE
    ) {
        fixed_date_type_raw
    } else {
        String::new()
    };
    let fixed_date_value_raw = payload
        .get("presaleFixedDateValue")
        .map(value_to_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    if mode == PRESALE_GOAL {
        if goal_target <= 0 {
            return Err("预售目标数量必须为大于0的整数".into());
        }
        return Ok(PresaleNormalized {
            mode,
            goal_target,
            fixed_date_type: String::new(),
            fixed_date_value: String::new(),
        });
    }

    if mode == PRESALE_FIXED {
        if fixed_date_type.is_empty() {
            return Err("请设置固定预售日期类型".into());
        }
        if fixed_date_value_raw.is_empty() {
            return Err("请设置固定预售日期".into());
        }
        if (fixed_date_type == FIXED_MONTH_START || fixed_date_type == FIXED_MONTH_END)
            && !is_month_value(&fixed_date_value_raw)
        {
            return Err("固定预售月份格式应为 YYYY-MM".into());
        }
        if fixed_date_type == FIXED_DATE && !is_date_value(&fixed_date_value_raw) {
            return Err("固定预售日期格式应为 YYYY-MM-DD".into());
        }
        return Ok(PresaleNormalized {
            mode,
            goal_target: 0,
            fixed_date_type,
            fixed_date_value: fixed_date_value_raw,
        });
    }

    Ok(PresaleNormalized {
        mode: PRESALE_NONE.to_string(),
        goal_target: 0,
        fixed_date_type: String::new(),
        fixed_date_value: String::new(),
    })
}

fn value_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// getProductPresaleSnapshot：含 isPresale 标记；规范化出错时回退 none。
pub struct PresaleSnapshot {
    pub mode: String,
    pub goal_target: i64,
    pub fixed_date_type: String,
    pub fixed_date_value: String,
    pub is_presale: bool,
}

pub fn product_presale_snapshot(product: &Value) -> PresaleSnapshot {
    let input = json!({
        "presaleMode": product.get("presaleMode"),
        "presaleGoalTarget": product.get("presaleGoalTarget"),
        "presaleFixedDateType": product.get("presaleFixedDateType"),
        "presaleFixedDateValue": product.get("presaleFixedDateValue"),
    });
    match normalize_product_presale_input(&input) {
        Ok(n) => {
            let is_presale = n.mode != PRESALE_NONE;
            PresaleSnapshot {
                mode: n.mode,
                goal_target: n.goal_target,
                fixed_date_type: n.fixed_date_type,
                fixed_date_value: n.fixed_date_value,
                is_presale,
            }
        }
        Err(_) => PresaleSnapshot {
            mode: PRESALE_NONE.to_string(),
            goal_target: 0,
            fixed_date_type: String::new(),
            fixed_date_value: String::new(),
            is_presale: false,
        },
    }
}

// ============================================================
// 订单计价
// ============================================================

pub struct PricingResult {
    pub priced_items: Vec<Value>,
    /// 商品小计（不含运费）。保留以对齐旧 calculateOrderPricing 返回结构，部分调用未读取。
    #[allow(dead_code)]
    pub products_total: f64,
    pub shipping_fee: f64,
    pub original_total: f64,
}

/// calculateOrderPricing：从下单 items + 商品行算价。出错返回 Err(消息)。
/// `normalized_items`：每项含 id(i64)、quantity(i64)、可选 name。
/// `product_by_id`：id -> 商品行 JSON（price/discountPrice/stock/shippingTag/shippingCost/presale*）。
pub fn calculate_order_pricing(
    normalized_items: &[Value],
    product_by_id: &std::collections::HashMap<i64, Value>,
    free_shipping_threshold: f64,
) -> Result<PricingResult, String> {
    let mut priced_items: Vec<Value> = Vec::new();
    let mut shipping_groups: BTreeMap<String, f64> = BTreeMap::new();
    let mut products_total = 0.0_f64;

    for item in normalized_items {
        let id = num(item.get("id").unwrap_or(&Value::Null)) as i64;
        let quantity = num(item.get("quantity").unwrap_or(&Value::Null));
        let item_name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let product = product_by_id.get(&id).ok_or_else(|| {
            let label = if item_name.is_empty() {
                id.to_string()
            } else {
                item_name.clone()
            };
            format!("商品 {label} 不存在")
        })?;
        let presale = product_presale_snapshot(product);

        let product_stock = num(product.get("stock").unwrap_or(&Value::Null));
        if !presale.is_presale && product_stock < quantity {
            let pname = product
                .get("name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.to_string());
            return Err(format!("商品 {pname} 库存不足"));
        }

        let price_v = product.get("price").cloned().unwrap_or(Value::Null);
        let discount_v = product.get("discountPrice").cloned().unwrap_or(Value::Null);
        let price = effective_product_price(&price_v, &discount_v);
        let original_price = {
            let p = num(&price_v);
            round_money(if p != 0.0 { p } else { price })
        };
        let discount_price = normalize_discount_price(&discount_v, &price_v);
        let shipping_tag =
            to_shipping_group_key(product.get("shippingTag").unwrap_or(&Value::Null));
        let shipping_cost = round_money(num(product.get("shippingCost").unwrap_or(&Value::Null)));

        let entry = shipping_groups.entry(shipping_tag.clone()).or_insert(-1.0);
        if *entry < 0.0 || shipping_cost > *entry {
            *entry = shipping_cost;
        }

        products_total += price * quantity;

        let final_name = product
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| {
                if item_name.is_empty() {
                    None
                } else {
                    Some(item_name.clone())
                }
            })
            .unwrap_or_else(|| format!("商品{id}"));

        priced_items.push(json!({
            "id": id,
            "name": final_name,
            "quantity": quantity as i64,
            "price": price,
            "originalPrice": original_price,
            "discountPrice": discount_price,
            "shippingTag": shipping_tag,
            "shippingCost": shipping_cost,
            "isPresale": presale.is_presale,
            "presaleMode": presale.mode,
        }));
    }

    let products_total = round_money(products_total);
    let raw_shipping_fee = round_money(shipping_groups.values().map(|f| f.max(0.0)).sum());
    let shipping_fee = if products_total >= free_shipping_threshold {
        0.0
    } else {
        raw_shipping_fee
    };
    let original_total = round_money(products_total + shipping_fee);

    Ok(PricingResult {
        priced_items,
        products_total,
        shipping_fee,
        original_total,
    })
}

/// normalizePricedItemForMerge：把已计价项规范化（合并用）；无效返回 None。
fn normalize_priced_item_for_merge(item: &Value) -> Option<Value> {
    let quantity_raw = item.get("quantity").map(num).unwrap_or(0.0);
    if quantity_raw.fract() != 0.0 || quantity_raw <= 0.0 {
        return None;
    }
    let quantity = quantity_raw as i64;

    let parsed_id = item.get("id").map(num).unwrap_or(0.0);
    let product_id = if parsed_id.fract() == 0.0 && parsed_id > 0.0 {
        Some(parsed_id as i64)
    } else {
        None
    };
    let price = round_money(item.get("price").map(num).unwrap_or(0.0).max(0.0));
    let original_price = round_money(item.get("originalPrice").map(num).unwrap_or(0.0).max(price));
    let discount_price = normalize_discount_price(
        item.get("discountPrice").unwrap_or(&Value::Null),
        &json!(original_price),
    );
    let shipping_tag = to_shipping_group_key(item.get("shippingTag").unwrap_or(&Value::Null));
    let shipping_cost = round_money(item.get("shippingCost").map(num).unwrap_or(0.0).max(0.0));
    let presale_mode_in = item
        .get("presaleMode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let presale_mode = if matches!(
        presale_mode_in.as_str(),
        PRESALE_NONE | PRESALE_GOAL | PRESALE_FIXED
    ) {
        presale_mode_in
    } else if truthy(item.get("isPresale").unwrap_or(&Value::Null)) {
        PRESALE_GOAL.to_string()
    } else {
        PRESALE_NONE.to_string()
    };
    let is_presale = presale_mode != PRESALE_NONE;

    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| match product_id {
            Some(pid) => format!("商品{pid}"),
            None => "未命名商品".to_string(),
        });

    Some(json!({
        "id": product_id,
        "name": name,
        "quantity": quantity,
        "price": price,
        "originalPrice": original_price,
        "discountPrice": discount_price,
        "shippingTag": shipping_tag,
        "shippingCost": shipping_cost,
        "isPresale": is_presale,
        "presaleMode": presale_mode,
    }))
}

/// mergePricedItems：把多组已计价项规范化后按唯一键合并数量。
pub fn merge_priced_items(groups: &[&[Value]]) -> Vec<Value> {
    // 用 Vec 保持插入顺序（对齐 JS Map）。
    let mut order: Vec<String> = Vec::new();
    let mut merged: std::collections::HashMap<String, Value> = std::collections::HashMap::new();

    for group in groups {
        for raw in group.iter() {
            let item = match normalize_priced_item_for_merge(raw) {
                Some(i) => i,
                None => continue,
            };
            let key = serde_json::to_string(&json!([
                item.get("id")
                    .and_then(|v| v.as_i64())
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                item.get("name"),
                item.get("price"),
                item.get("originalPrice"),
                item.get("discountPrice")
                    .and_then(|v| v.as_f64())
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                item.get("shippingTag"),
                item.get("shippingCost"),
                if truthy(item.get("isPresale").unwrap_or(&Value::Null)) {
                    1
                } else {
                    0
                },
                item.get("presaleMode"),
            ]))
            .unwrap_or_default();

            if let Some(existing) = merged.get_mut(&key) {
                let q = existing
                    .get("quantity")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    + item.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
                existing["quantity"] = json!(q);
            } else {
                order.push(key.clone());
                merged.insert(key, item);
            }
        }
    }

    order
        .into_iter()
        .filter_map(|k| merged.remove(&k))
        .filter(|i| i.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0) > 0)
        .collect()
}

/// calculatePricingFromPricedItems：从已计价项重新算价（合并/拆单/统计用）。
pub fn calculate_pricing_from_priced_items(
    priced_items: &[Value],
    free_shipping_threshold: f64,
) -> PricingResult {
    let normalized = merge_priced_items(&[priced_items]);
    let mut shipping_groups: BTreeMap<String, f64> = BTreeMap::new();
    let mut products_total = 0.0_f64;

    for item in &normalized {
        products_total += item.get("price").map(num).unwrap_or(0.0)
            * item.get("quantity").map(num).unwrap_or(0.0);
        let shipping_tag = to_shipping_group_key(item.get("shippingTag").unwrap_or(&Value::Null));
        let shipping_cost = round_money(item.get("shippingCost").map(num).unwrap_or(0.0));
        let entry = shipping_groups.entry(shipping_tag).or_insert(-1.0);
        if *entry < 0.0 || shipping_cost > *entry {
            *entry = shipping_cost;
        }
    }

    let products_total = round_money(products_total);
    let raw_shipping_fee = round_money(shipping_groups.values().map(|f| f.max(0.0)).sum());
    let shipping_fee = if products_total >= free_shipping_threshold {
        0.0
    } else {
        raw_shipping_fee
    };
    let original_total = round_money(products_total + shipping_fee);

    PricingResult {
        priced_items: normalized,
        products_total,
        shipping_fee,
        original_total,
    }
}

// ============================================================
// 优惠券评估
// ============================================================

/// 优惠券行 JSON（DB 取出后转的）。
pub struct CouponEval {
    pub coupon: Value, // formatCoupon 后的对象
    pub discount_amount: f64,
    pub payable_amount: f64,
}

pub fn normalize_coupon_code(code: &str) -> String {
    code.trim().to_uppercase()
}

/// formatCoupon：规范金额字段、status 转数字、isExpired 计算。
pub fn format_coupon(coupon: &Value) -> Value {
    let mut obj = coupon.as_object().cloned().unwrap_or_default();
    let code = normalize_coupon_code(coupon.get("code").and_then(|v| v.as_str()).unwrap_or(""));
    obj.insert("code".into(), json!(code));
    obj.insert(
        "minSpend".into(),
        json!(round_money(num(coupon
            .get("minSpend")
            .unwrap_or(&Value::Null)))),
    );
    obj.insert(
        "discountValue".into(),
        json!(round_money(num(coupon
            .get("discountValue")
            .unwrap_or(&Value::Null)))),
    );
    let max_discount = match coupon.get("maxDiscount") {
        Some(Value::Null) | None => Value::Null,
        Some(Value::String(s)) if s.is_empty() => Value::Null,
        Some(v) => json!(round_money(num(v))),
    };
    obj.insert("maxDiscount".into(), max_discount);
    obj.insert(
        "status".into(),
        json!(num(coupon.get("status").unwrap_or(&Value::Null)) as i64),
    );
    obj.insert("isExpired".into(), json!(is_coupon_expired(coupon)));
    Value::Object(obj)
}

/// isCouponExpired：expiresAt 过去/无效则过期。无 expiresAt 不过期。
pub fn is_coupon_expired(coupon: &Value) -> bool {
    let expires = coupon.get("expiresAt").and_then(|v| v.as_str());
    match expires {
        None | Some("") => false,
        Some(s) => match parse_datetime_ms(s) {
            Some(ts) => ts <= now_ms(),
            None => true,
        },
    }
}

/// getCouponBenefitText：优惠描述文案。
pub fn coupon_benefit_text(coupon: &Value) -> String {
    let discount_type = coupon
        .get("discountType")
        .and_then(|v| v.as_str())
        .unwrap_or("amount");
    let discount_value = num(coupon.get("discountValue").unwrap_or(&Value::Null));
    if discount_type == "percent" {
        let max = num(coupon.get("maxDiscount").unwrap_or(&Value::Null));
        if max > 0.0 {
            return format!("{}% (最高减¥{})", fmt_num(discount_value), fmt_num(max));
        }
        return format!("{}%", fmt_num(discount_value));
    }
    format!("减¥{}", fmt_num(discount_value))
}

/// getCouponDiscountAmount：算具体优惠金额；返回 Err(消息) 或 (discount, payable)。
fn coupon_discount_amount(coupon: &Value, order_amount: f64) -> Result<(f64, f64), String> {
    let amount = round_money(order_amount);
    let min_spend = round_money(num(coupon.get("minSpend").unwrap_or(&Value::Null)));
    if amount < min_spend {
        return Err(format!("订单金额未达到使用门槛 ¥{}", fmt_num(min_spend)));
    }
    let discount_type = coupon
        .get("discountType")
        .and_then(|v| v.as_str())
        .unwrap_or("amount")
        .to_string();
    let discount_value = num(coupon.get("discountValue").unwrap_or(&Value::Null));
    if !COUPON_DISCOUNT_TYPES.contains(&discount_type.as_str())
        || !discount_value.is_finite()
        || discount_value <= 0.0
    {
        return Err("优惠券配置异常".into());
    }

    let mut discount_amount = if discount_type == "percent" {
        let mut d = (amount * discount_value) / 100.0;
        let max = num(coupon.get("maxDiscount").unwrap_or(&Value::Null));
        if max.is_finite() && max > 0.0 {
            d = d.min(max);
        }
        d
    } else {
        discount_value
    };
    discount_amount = round_money(discount_amount);
    if discount_amount <= 0.0 {
        return Err("优惠券金额无效".into());
    }
    Ok((
        discount_amount.min(amount),
        round_money((amount - discount_amount).max(0.0)),
    ))
}

/// evaluateCoupon：完整评估（不存在/不可用/过期/门槛）。返回 Err(消息) 或 CouponEval。
pub fn evaluate_coupon(coupon: Option<&Value>, order_amount: f64) -> Result<CouponEval, String> {
    let coupon = coupon.ok_or_else(|| "优惠券不存在".to_string())?;
    let row = format_coupon(coupon);
    if row.get("status").and_then(|v| v.as_i64()).unwrap_or(0) != COUPON_UNUSED {
        return Err("优惠券不可用".into());
    }
    if row
        .get("isExpired")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Err("优惠券已过期".into());
    }
    let (discount_amount, payable_amount) = coupon_discount_amount(&row, order_amount)?;
    Ok(CouponEval {
        coupon: row,
        discount_amount,
        payable_amount,
    })
}

// ============================================================
// 时间 / 数字格式
// ============================================================

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// 解析 JS `new Date(value).getTime()`：支持 ISO8601 / 部分日期。失败 None。
pub fn parse_datetime_ms(value: &str) -> Option<i64> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }
    // 完整 RFC3339
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis());
    }
    // SQLite "YYYY-MM-DD HH:MM:SS"（视为 UTC，对齐 toSqliteDateTime 用 UTC 写入）
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(ndt.and_utc().timestamp_millis());
    }
    // 纯日期
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(nd.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis());
    }
    None
}

/// 数字格式化：整数不带小数，否则尽量短（对齐 JS 默认 Number→String）。
pub fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        // 去掉尾随 0
        let s = format!("{n}");
        s
    }
}

// ============================================================
// 合并元信息 normalizeOrderMergeMeta（忠实移植）
// ============================================================

fn to_money(v: &Value) -> f64 {
    round_money(num(v))
}

fn to_status(v: &Value) -> Option<i64> {
    let n = num(v);
    if n.fract() == 0.0 {
        Some(n as i64)
    } else {
        None
    }
}

fn clean_id(v: &Value) -> String {
    match v {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// dedupeParts：按 orderId 去重合并 amount，保留首个非空 statusBeforeMerge。
fn dedupe_parts(parts: &[Value]) -> Vec<Value> {
    let mut order: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for part in parts {
        let order_id = clean_id(part.get("orderId").unwrap_or(&Value::Null));
        if order_id.is_empty() {
            continue;
        }
        let amount = to_money(part.get("amount").unwrap_or(&Value::Null));
        let status = to_status(part.get("statusBeforeMerge").unwrap_or(&Value::Null));
        if let Some(prev) = map.get_mut(&order_id) {
            let new_amount = to_money(&json!(
                num(prev.get("amount").unwrap_or(&Value::Null)) + amount
            ));
            prev["amount"] = json!(new_amount);
            if prev
                .get("statusBeforeMerge")
                .map(|v| v.is_null())
                .unwrap_or(true)
            {
                prev["statusBeforeMerge"] = match status {
                    Some(s) => json!(s),
                    None => Value::Null,
                };
            }
        } else {
            order.push(order_id.clone());
            map.insert(
                order_id,
                json!({
                    "orderId": part.get("orderId").map(clean_id).unwrap_or_default(),
                    "amount": amount,
                    "statusBeforeMerge": status,
                }),
            );
        }
    }
    order.into_iter().filter_map(|k| map.remove(&k)).collect()
}

fn compute_shipping_adjustment(source: f64, appended: f64, merged: f64) -> f64 {
    to_money(&json!(
        to_money(&json!(merged)) - to_money(&json!(source)) - to_money(&json!(appended))
    ))
}

/// 忠实移植 normalizeOrderMergeMeta。输入可为字符串或对象，None/无效返回 Value::Null。
pub fn normalize_order_merge_meta(raw_value: Option<Value>) -> Value {
    let parsed = match raw_value {
        Some(Value::String(s)) => safe_parse(Some(&s), Value::Null),
        Some(v) => v,
        None => Value::Null,
    };
    if !parsed.is_object() {
        return Value::Null;
    }

    // parts
    let mut normalized_parts: Vec<Value> = match parsed.get("parts").and_then(|v| v.as_array()) {
        Some(arr) => dedupe_parts(arr),
        None => Vec::new(),
    };
    if normalized_parts.is_empty() {
        let mut legacy: Vec<Value> = Vec::new();
        let src = clean_id(parsed.get("sourceOrderId").unwrap_or(&Value::Null));
        let sub = clean_id(parsed.get("submittedOrderId").unwrap_or(&Value::Null));
        if !src.is_empty() {
            legacy.push(json!({
                "orderId": src,
                "amount": to_money(parsed.get("sourceAmount").unwrap_or(&Value::Null)),
                "statusBeforeMerge": Value::Null,
            }));
        }
        if !sub.is_empty() {
            legacy.push(json!({
                "orderId": sub,
                "amount": to_money(parsed.get("appendedAmount").unwrap_or(&Value::Null)),
                "statusBeforeMerge": Value::Null,
            }));
        }
        normalized_parts.extend(dedupe_parts(&legacy));
    }

    // history
    let mut normalized_history: Vec<Value> = Vec::new();
    if let Some(arr) = parsed.get("history").and_then(|v| v.as_array()) {
        for row in arr {
            let source_order_id = clean_id(row.get("sourceOrderId").unwrap_or(&Value::Null));
            let appended_order_id = {
                let a = clean_id(row.get("appendedOrderId").unwrap_or(&Value::Null));
                if a.is_empty() {
                    clean_id(row.get("submittedOrderId").unwrap_or(&Value::Null))
                } else {
                    a
                }
            };
            let new_order_id = {
                let n = clean_id(row.get("newOrderId").unwrap_or(&Value::Null));
                if n.is_empty() {
                    clean_id(row.get("mergedOrderId").unwrap_or(&Value::Null))
                } else {
                    n
                }
            };
            if source_order_id.is_empty() || appended_order_id.is_empty() || new_order_id.is_empty()
            {
                continue;
            }
            let source_shipping = to_money(row.get("sourceShippingFee").unwrap_or(&Value::Null));
            let appended_shipping =
                to_money(row.get("appendedShippingFee").unwrap_or(&Value::Null));
            let merged_shipping = to_money(row.get("mergedShippingFee").unwrap_or(&Value::Null));
            let shipping_adjustment = if row.get("shippingAdjustment").is_some()
                && !row.get("shippingAdjustment").unwrap().is_null()
            {
                to_money(row.get("shippingAdjustment").unwrap())
            } else if row.get("shippingAdjustment").is_some() {
                // undefined 在 JS 里走 else；serde 里没有 undefined，仅当字段缺失才算 undefined。
                compute_shipping_adjustment(source_shipping, appended_shipping, merged_shipping)
            } else {
                compute_shipping_adjustment(source_shipping, appended_shipping, merged_shipping)
            };
            let appended_amount = to_money(row.get("appendedAmount").unwrap_or(&Value::Null));
            let incremental = if field_present(row, "incrementalPayable") {
                to_money(row.get("incrementalPayable").unwrap())
            } else {
                to_money(&json!((appended_amount + shipping_adjustment).max(0.0)))
            };
            let shipping_saved = if field_present(row, "shippingSaved") {
                to_money(row.get("shippingSaved").unwrap())
            } else {
                to_money(&json!((-shipping_adjustment).max(0.0)))
            };
            normalized_history.push(json!({
                "sourceOrderId": source_order_id,
                "appendedOrderId": appended_order_id,
                "newOrderId": new_order_id,
                "sourceAmount": to_money(row.get("sourceAmount").unwrap_or(&Value::Null)),
                "appendedAmount": appended_amount,
                "mergedAmount": to_money(row.get("mergedAmount").unwrap_or(&Value::Null)),
                "sourceShippingFee": source_shipping,
                "appendedShippingFee": appended_shipping,
                "mergedShippingFee": merged_shipping,
                "shippingAdjustment": shipping_adjustment,
                "incrementalPayable": incremental,
                "shippingSaved": shipping_saved,
                "mergedAt": str_or_null(row.get("mergedAt")),
            }));
        }
    }

    if normalized_history.is_empty() {
        let source_order_id = clean_id(parsed.get("sourceOrderId").unwrap_or(&Value::Null));
        let appended_order_id = clean_id(parsed.get("submittedOrderId").unwrap_or(&Value::Null));
        let new_order_id = clean_id(parsed.get("mergedOrderId").unwrap_or(&Value::Null));
        if !source_order_id.is_empty() && !appended_order_id.is_empty() && !new_order_id.is_empty()
        {
            let source_shipping = to_money(parsed.get("sourceShippingFee").unwrap_or(&Value::Null));
            let appended_shipping =
                to_money(parsed.get("appendedShippingFee").unwrap_or(&Value::Null));
            let merged_shipping = to_money(parsed.get("mergedShippingFee").unwrap_or(&Value::Null));
            let shipping_adjustment =
                compute_shipping_adjustment(source_shipping, appended_shipping, merged_shipping);
            let appended_amount = to_money(parsed.get("appendedAmount").unwrap_or(&Value::Null));
            normalized_history.push(json!({
                "sourceOrderId": source_order_id,
                "appendedOrderId": appended_order_id,
                "newOrderId": new_order_id,
                "sourceAmount": to_money(parsed.get("sourceAmount").unwrap_or(&Value::Null)),
                "appendedAmount": appended_amount,
                "mergedAmount": to_money(parsed.get("mergedAmount").unwrap_or(&Value::Null)),
                "sourceShippingFee": source_shipping,
                "appendedShippingFee": appended_shipping,
                "mergedShippingFee": merged_shipping,
                "shippingAdjustment": shipping_adjustment,
                "incrementalPayable": to_money(&json!((appended_amount + shipping_adjustment).max(0.0))),
                "shippingSaved": if field_present(&parsed, "shippingSaved") {
                    to_money(parsed.get("shippingSaved").unwrap())
                } else {
                    to_money(&json!((-shipping_adjustment).max(0.0)))
                },
                "mergedAt": str_or_null(parsed.get("mergedAt")),
            }));
        }
    }

    let latest = normalized_history.last().cloned();
    let latest_str = |key: &str| -> Value {
        latest
            .as_ref()
            .and_then(|h| h.get(key).cloned())
            .unwrap_or(Value::Null)
    };

    let merged_order_id =
        first_non_empty_id(parsed.get("mergedOrderId"), &latest_str("newOrderId"));
    let source_order_id =
        first_non_empty_id(parsed.get("sourceOrderId"), &latest_str("sourceOrderId"));
    let submitted_order_id = first_non_empty_id(
        parsed.get("submittedOrderId"),
        &latest_str("appendedOrderId"),
    );
    let source_shipping = to_money(&coalesce(
        parsed.get("sourceShippingFee"),
        &latest_str("sourceShippingFee"),
    ));
    let appended_shipping = to_money(&coalesce(
        parsed.get("appendedShippingFee"),
        &latest_str("appendedShippingFee"),
    ));
    let merged_shipping = to_money(&coalesce(
        parsed.get("mergedShippingFee"),
        &latest_str("mergedShippingFee"),
    ));
    let shipping_adjustment = if field_present(&parsed, "shippingAdjustment") {
        to_money(parsed.get("shippingAdjustment").unwrap())
    } else {
        compute_shipping_adjustment(source_shipping, appended_shipping, merged_shipping)
    };
    let source_amount = to_money(&coalesce(
        parsed.get("sourceAmount"),
        &latest_str("sourceAmount"),
    ));
    let appended_amount = to_money(&coalesce(
        parsed.get("appendedAmount"),
        &latest_str("appendedAmount"),
    ));
    let merged_amount = to_money(&coalesce(
        parsed.get("mergedAmount"),
        &latest_str("mergedAmount"),
    ));
    let incremental = if field_present(&parsed, "incrementalPayable") {
        to_money(parsed.get("incrementalPayable").unwrap())
    } else if latest
        .as_ref()
        .map(|h| field_present(h, "incrementalPayable"))
        .unwrap_or(false)
    {
        to_money(&latest_str("incrementalPayable"))
    } else {
        to_money(&json!((appended_amount + shipping_adjustment).max(0.0)))
    };

    if merged_order_id.is_empty()
        && normalized_parts.is_empty()
        && source_order_id.is_empty()
        && submitted_order_id.is_empty()
    {
        return Value::Null;
    }

    let shipping_saved = if field_present(&parsed, "shippingSaved") {
        to_money(parsed.get("shippingSaved").unwrap())
    } else if latest
        .as_ref()
        .map(|h| field_present(h, "shippingSaved"))
        .unwrap_or(false)
    {
        to_money(&latest_str("shippingSaved"))
    } else {
        to_money(&json!((-shipping_adjustment).max(0.0)))
    };

    let merge_count = {
        let from_parsed = to_status(parsed.get("mergeCount").unwrap_or(&Value::Null)).unwrap_or(0);
        from_parsed.max(normalized_history.len() as i64)
    };

    json!({
        "mergedOrderId": none_if_empty(&merged_order_id),
        "sourceOrderId": none_if_empty(&source_order_id),
        "submittedOrderId": none_if_empty(&submitted_order_id),
        "sourceAmount": source_amount,
        "appendedAmount": appended_amount,
        "mergedAmount": merged_amount,
        "sourceShippingFee": source_shipping,
        "appendedShippingFee": appended_shipping,
        "mergedShippingFee": merged_shipping,
        "shippingAdjustment": shipping_adjustment,
        "incrementalPayable": incremental,
        "shippingSaved": shipping_saved,
        "contactPolicy": parsed.get("contactPolicy").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("new_order"),
        "contactChanged": truthy(parsed.get("contactChanged").unwrap_or(&Value::Null)),
        "mergedAt": coalesce_str(parsed.get("mergedAt"), &latest_str("mergedAt")),
        "mergeCount": merge_count,
        "parts": normalized_parts,
        "history": normalized_history,
    })
}

fn field_present(v: &Value, key: &str) -> bool {
    matches!(v.get(key), Some(x) if !x.is_null())
}

fn str_or_null(v: Option<&Value>) -> Value {
    match v {
        Some(Value::String(s)) if !s.is_empty() => json!(s),
        Some(Value::Number(n)) => json!(n.to_string()),
        _ => Value::Null,
    }
}

fn first_non_empty_id(primary: Option<&Value>, fallback: &Value) -> String {
    let p = primary.map(clean_id).unwrap_or_default();
    if !p.is_empty() {
        return p;
    }
    clean_id(fallback)
}

fn coalesce(primary: Option<&Value>, fallback: &Value) -> Value {
    match primary {
        Some(v) if !v.is_null() => v.clone(),
        _ => fallback.clone(),
    }
}

fn coalesce_str(primary: Option<&Value>, fallback: &Value) -> Value {
    match primary {
        Some(Value::String(s)) if !s.is_empty() => json!(s),
        _ => match fallback {
            Value::String(s) if !s.is_empty() => json!(s),
            _ => Value::Null,
        },
    }
}

fn none_if_empty(s: &str) -> Value {
    if s.is_empty() {
        Value::Null
    } else {
        json!(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 商品价格 ----

    #[test]
    fn effective_price_picks_valid_discount() {
        assert_eq!(effective_product_price(&json!(100), &json!(80)), 80.0);
        // 折扣价 >= 原价 → 视为无效，取原价
        assert_eq!(effective_product_price(&json!(100), &json!(120)), 100.0);
        assert_eq!(effective_product_price(&json!(100), &json!(0)), 100.0);
        assert_eq!(effective_product_price(&json!(100), &Value::Null), 100.0);
    }

    #[test]
    fn discount_and_positive_price_rules() {
        assert_eq!(
            normalize_discount_price(&json!(80), &json!(100)),
            Some(80.0)
        );
        assert_eq!(normalize_discount_price(&json!(100), &json!(100)), None); // 等于原价
        assert_eq!(normalize_discount_price(&json!(0), &json!(100)), None);
        assert_eq!(to_positive_price_or_null(&json!(50)), Some(50.0));
        assert_eq!(to_positive_price_or_null(&json!(0)), None);
        assert_eq!(to_positive_price_or_null(&json!(-5)), None);
    }

    // ---- 订单计价 / 运费门槛 ----

    fn item(id: i64, price: f64, qty: i64, tag: &str, ship: f64) -> Value {
        json!({
            "id": id, "name": format!("p{id}"),
            "price": price, "quantity": qty,
            "shippingTag": tag, "shippingCost": ship,
        })
    }

    #[test]
    fn shipping_charged_below_threshold() {
        let items = vec![item(1, 50.0, 2, "A", 10.0)]; // 小计 100 < 150
        let r = calculate_pricing_from_priced_items(&items, 150.0);
        assert_eq!(r.products_total, 100.0);
        assert_eq!(r.shipping_fee, 10.0);
        assert_eq!(r.original_total, 110.0);
    }

    #[test]
    fn free_shipping_at_or_above_threshold() {
        let items = vec![item(1, 100.0, 2, "A", 10.0)]; // 小计 200 >= 150
        let r = calculate_pricing_from_priced_items(&items, 150.0);
        assert_eq!(r.products_total, 200.0);
        assert_eq!(r.shipping_fee, 0.0);
        assert_eq!(r.original_total, 200.0);
    }

    #[test]
    fn shipping_group_takes_max_cost() {
        // 同一 shippingTag 的运费取组内最高（两个不同商品 id，避免被合并）
        let items = vec![item(1, 10.0, 1, "A", 8.0), item(2, 10.0, 1, "A", 12.0)];
        let r = calculate_pricing_from_priced_items(&items, 1000.0);
        assert_eq!(r.products_total, 20.0);
        assert_eq!(r.shipping_fee, 12.0);
    }

    // ---- 优惠券 ----

    #[test]
    fn coupon_code_normalized() {
        assert_eq!(normalize_coupon_code("  abc "), "ABC");
        assert_eq!(normalize_coupon_code("XyZ"), "XYZ");
    }

    #[test]
    fn coupon_expiry() {
        assert!(!is_coupon_expired(&json!({}))); // 无 expiresAt
        assert!(!is_coupon_expired(&json!({ "expiresAt": "" })));
        assert!(is_coupon_expired(&json!({ "expiresAt": "2000-01-01" }))); // 过去
        assert!(is_coupon_expired(&json!({ "expiresAt": "not-a-date" }))); // 无法解析 → 过期
        assert!(!is_coupon_expired(&json!({ "expiresAt": "2999-12-31" }))); // 未来
    }

    #[test]
    fn evaluate_coupon_amount_and_percent() {
        // 不存在
        assert!(evaluate_coupon(None, 100.0).is_err());

        // 定额：减 20
        let c =
            json!({ "status": 1, "discountType": "amount", "discountValue": 20, "minSpend": 0 });
        let e = evaluate_coupon(Some(&c), 100.0).unwrap();
        assert_eq!(e.discount_amount, 20.0);
        assert_eq!(e.payable_amount, 80.0);

        // 未达门槛
        let c =
            json!({ "status": 1, "discountType": "amount", "discountValue": 20, "minSpend": 200 });
        assert!(evaluate_coupon(Some(&c), 100.0).is_err());

        // 百分比：10% of 200 = 20
        let c =
            json!({ "status": 1, "discountType": "percent", "discountValue": 10, "minSpend": 0 });
        let e = evaluate_coupon(Some(&c), 200.0).unwrap();
        assert_eq!(e.discount_amount, 20.0);
        assert_eq!(e.payable_amount, 180.0);

        // 百分比封顶 maxDiscount=15
        let c = json!({ "status": 1, "discountType": "percent", "discountValue": 10, "minSpend": 0, "maxDiscount": 15 });
        let e = evaluate_coupon(Some(&c), 200.0).unwrap();
        assert_eq!(e.discount_amount, 15.0);
        assert_eq!(e.payable_amount, 185.0);
    }

    #[test]
    fn evaluate_coupon_rejects_unusable_and_expired() {
        // status != COUPON_UNUSED(1)
        let c =
            json!({ "status": 2, "discountType": "amount", "discountValue": 10, "minSpend": 0 });
        assert!(evaluate_coupon(Some(&c), 100.0).is_err());
        // 已过期
        let c = json!({ "status": 1, "expiresAt": "2000-01-01", "discountType": "amount", "discountValue": 10, "minSpend": 0 });
        assert!(evaluate_coupon(Some(&c), 100.0).is_err());
    }

    #[test]
    fn fmt_num_integer_vs_fraction() {
        assert_eq!(fmt_num(100.0), "100");
        assert_eq!(fmt_num(99.5), "99.5");
    }
}
