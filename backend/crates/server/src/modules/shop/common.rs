//! shop 通用工具：JSON 解析、金额处理、参数解析、订单/商品序列化、松类型容错读取。
//! 忠实移植旧 server.cjs 顶部的 safeParse / roundMoney / mapOrderRow / 各 sanitize 函数。

use serde_json::{json, Map, Value};

// ============================================================
// 常量（对齐旧 server.cjs）
// ============================================================

/// 已支付到已完成阶段（计入销售额）。
pub const SALES_VALID_STATUSES: [i64; 4] = [2, 3, 4, 5];
/// 合法订单状态值。
pub const ORDER_STATUS_VALUES: [i64; 6] = [0, 1, 2, 3, 4, 5];

/// 状态机：oldStatus -> 允许的 newStatus 列表。
pub fn status_transitions(old: i64) -> &'static [i64] {
    match old {
        0 => &[],
        1 => &[0, 2, 5], // 待付款 -> 取消 / 待发货 / 待确认
        2 => &[0, 3],    // 待发货 -> 取消 / 已发货
        3 => &[4],       // 已发货 -> 已完成
        4 => &[],        // 已完成终态
        5 => &[0, 2],    // 待确认 -> 取消 / 待发货
        _ => &[],
    }
}

/// 可取消状态（取消时回补库存）。
pub const CANCELLABLE_STATUSES: [i64; 3] = [1, 2, 5];
/// 自助修改联系信息允许的源状态。
pub fn is_self_service_contact_editable(status: i64) -> bool {
    matches!(status, 1 | 2 | 5)
}
/// 自助合并允许的源状态。
pub fn is_self_service_mergeable_source(status: i64) -> bool {
    matches!(status, 1 | 2 | 5)
}

// 优惠券状态
pub const COUPON_DISABLED: i64 = 0;
pub const COUPON_UNUSED: i64 = 1;
pub const COUPON_USED: i64 = 2;
pub const COUPON_DISCOUNT_TYPES: [&str; 2] = ["amount", "percent"];

// 预售模式
pub const PRESALE_NONE: &str = "none";
pub const PRESALE_GOAL: &str = "goal";
pub const PRESALE_FIXED: &str = "fixed";

// 漏斗步骤（转化率统计）
pub const FUNNEL_STEPS: [(&str, &str); 6] = [
    ("home_view", "首页访问"),
    ("product_view", "商品详情访问"),
    ("add_to_cart", "加入购物车"),
    ("checkout_view", "进入结算页"),
    ("order_submitted", "提交订单"),
    ("payment_submitted", "提交支付"),
];

/// uploads URL 前缀（库内与返回统一为 /uploads/shop）。
pub const UPLOAD_URL_PREFIX: &str = "/uploads/shop";
/// 旧 site-config 校验里用到的“合法上传 URL”前缀（与新约定对齐）。
pub const LEGACY_API_UPLOADS_PREFIX: &str = "/api/uploads/";

// ============================================================
// JSON / 数值工具
// ============================================================

/// 对齐 safeParse：null/空串返回 fallback，解析失败返回 fallback。
pub fn safe_parse(s: Option<&str>, fallback: Value) -> Value {
    match s {
        Some(text) if !text.is_empty() => serde_json::from_str::<Value>(text).unwrap_or(fallback),
        _ => fallback,
    }
}

/// 对齐 roundMoney：Number((x||0).toFixed(2))，保留两位小数。
pub fn round_money(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    // toFixed(2) 用 round-half-away-from-zero 近似；JS toFixed 行为接近此。
    (value * 100.0).round() / 100.0
}

/// 把 serde_json::Value 当作数字读取（容错：字符串数字、null）。对齐 JS Number(x)||0。
pub fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// 取对象字段为字符串（trim）。
pub fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// JS truthy 判断（用于 isPresale 等布尔字段：true/非零数字/非空串）。
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        _ => true,
    }
}

/// sanitizeConfigText：trim + 截断到 maxLength（按字符）。
pub fn sanitize_config_text(value: &str, max_length: usize) -> String {
    value.trim().chars().take(max_length).collect()
}

/// 模拟 JS parseInt(x,10) || default（0 视为 falsy → default）。
pub fn parse_int_or(s: Option<&str>, default: i64) -> i64 {
    let n = s.and_then(parse_int_radix10);
    match n {
        Some(v) if v != 0 => v,
        _ => default,
    }
}

/// 模拟 JS Number.parseInt(x,10)：取十进制前缀，无效返回 None。
pub fn parse_int_radix10(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut i = 0;
    let mut sign = 1_i64;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    t[start..i].parse::<i64>().ok().map(|n| sign * n)
}

// ============================================================
// 松类型容错的 SELECT 列表（数值列统一 CAST AS TEXT，按 Option<String> 读，避免 sqlx 解码 500）
// ============================================================

/// orders 表全列（数值列 CAST AS TEXT）。配合 OrderRow。
pub const ORDER_SELECT: &str = "SELECT id, CAST(total AS TEXT) AS total, CAST(originalTotal AS TEXT) AS originalTotal, \
    CAST(discountAmount AS TEXT) AS discountAmount, couponCode, mergeMeta, items, contactName, contactPhone, contactEmail, \
    province, city, district, addressDetail, trackingCompany, trackingNo, CAST(status AS TEXT) AS status, created_at, \
    CAST(exported AS TEXT) AS exported, CAST(hasPresaleItems AS TEXT) AS hasPresaleItems, CAST(hasSpotItems AS TEXT) AS hasSpotItems, \
    CAST(spotExported AS TEXT) AS spotExported, presaleExportedProducts FROM orders";

/// sub_orders 表全列（shipped CAST AS TEXT）。配合 SubOrderRow。
pub const SUB_ORDER_SELECT: &str = "SELECT id, subKey, label, items, trackingCompany, trackingNo, \
    CAST(shipped AS TEXT) AS shipped, shipped_at FROM sub_orders";

/// coupons 表全列（数值列 CAST AS TEXT）。配合 coupons::CouponRow。
pub const COUPON_SELECT: &str = "SELECT id, code, name, batchNo, CAST(minSpend AS TEXT) AS minSpend, discountType, \
    CAST(discountValue AS TEXT) AS discountValue, CAST(maxDiscount AS TEXT) AS maxDiscount, CAST(status AS TEXT) AS status, \
    expiresAt, usedOrderId, used_at, created_at FROM coupons";

/// contact_messages 表全列（status CAST AS TEXT）。配合 misc::ContactRow。
pub const CONTACT_SELECT: &str = "SELECT id, name, contact, orderId, content, CAST(status AS TEXT) AS status, \
    handled_at, created_at FROM contact_messages";

// ============================================================
// 松类型容错的订单行（旧 shop.db 数值列可能存成 TEXT）
// ============================================================

/// 订单行：SELECT * FROM orders。数值列读 String 容错，避免解码 500。
#[derive(sqlx::FromRow, Clone)]
pub struct OrderRow {
    pub id: String,
    pub total: Option<String>,
    #[sqlx(rename = "originalTotal")]
    pub original_total: Option<String>,
    #[sqlx(rename = "discountAmount")]
    pub discount_amount: Option<String>,
    #[sqlx(rename = "couponCode")]
    pub coupon_code: Option<String>,
    #[sqlx(rename = "mergeMeta")]
    pub merge_meta: Option<String>,
    pub items: Option<String>,
    #[sqlx(rename = "contactName")]
    pub contact_name: Option<String>,
    #[sqlx(rename = "contactPhone")]
    pub contact_phone: Option<String>,
    #[sqlx(rename = "contactEmail")]
    pub contact_email: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    #[sqlx(rename = "addressDetail")]
    pub address_detail: Option<String>,
    #[sqlx(rename = "trackingCompany")]
    pub tracking_company: Option<String>,
    #[sqlx(rename = "trackingNo")]
    pub tracking_no: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
    pub exported: Option<String>,
    #[sqlx(rename = "hasPresaleItems")]
    pub has_presale_items: Option<String>,
    #[sqlx(rename = "hasSpotItems")]
    pub has_spot_items: Option<String>,
    #[sqlx(rename = "spotExported")]
    pub spot_exported: Option<String>,
    #[sqlx(rename = "presaleExportedProducts")]
    pub presale_exported_products: Option<String>,
}

/// 子订单行：SELECT * FROM sub_orders。
#[derive(sqlx::FromRow, Clone)]
pub struct SubOrderRow {
    pub id: i64,
    #[sqlx(rename = "subKey")]
    pub sub_key: String,
    pub label: String,
    pub items: Option<String>,
    #[sqlx(rename = "trackingCompany")]
    pub tracking_company: Option<String>,
    #[sqlx(rename = "trackingNo")]
    pub tracking_no: Option<String>,
    pub shipped: Option<String>,
    pub shipped_at: Option<String>,
}

fn opt_str_num(s: &Option<String>) -> f64 {
    s.as_deref()
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn opt_str_int(s: &Option<String>) -> i64 {
    s.as_deref()
        .and_then(|x| {
            // 容错：可能是 "2" 或 "2.0"
            x.trim()
                .parse::<i64>()
                .ok()
                .or_else(|| x.trim().parse::<f64>().ok().map(|f| f as i64))
        })
        .unwrap_or(0)
}

fn opt_str_bool_flag(s: &Option<String>) -> i64 {
    if opt_str_int(s) != 0 {
        1
    } else {
        0
    }
}

/// 子订单 -> JSON（对齐 mapOrderRow 内 subOrders.map）。
pub fn sub_order_to_json(s: &SubOrderRow) -> Value {
    json!({
        "id": s.id,
        "subKey": s.sub_key,
        "label": s.label,
        "items": safe_parse(s.items.as_deref(), json!([])),
        "trackingCompany": s.tracking_company.clone().unwrap_or_default(),
        "trackingNo": s.tracking_no.clone().unwrap_or_default(),
        "shipped": opt_str_bool_flag(&s.shipped),
        "shipped_at": s.shipped_at,
    })
}

/// 对齐 mapOrderRow：把 orders 行 + 可选子订单转为前端 JSON。
/// 当 `items_override` 提供时（如 spot-export-data），用其替换 items。
pub fn map_order_row(
    row: &OrderRow,
    sub_orders: Option<&[SubOrderRow]>,
    items_override: Option<Value>,
) -> Value {
    let merge_meta = super::pricing::normalize_order_merge_meta(
        row.merge_meta.as_deref().map(|s| Value::String(s.to_string())),
    );
    let items = items_override.unwrap_or_else(|| safe_parse(row.items.as_deref(), json!([])));
    let items_arr: Vec<Value> = items.as_array().cloned().unwrap_or_default();

    let has_presale_flag = opt_str_int(&row.has_presale_items) != 0;
    let any_presale = items_arr
        .iter()
        .any(|i| truthy(i.get("isPresale").unwrap_or(&Value::Null)));
    let has_presale_items = if has_presale_flag || any_presale { 1 } else { 0 };
    let has_spot_items = items_arr
        .iter()
        .any(|i| !truthy(i.get("isPresale").unwrap_or(&Value::Null)));
    let order_type = if has_presale_items == 1 && has_spot_items {
        "mixed"
    } else if has_presale_items == 1 {
        "presale"
    } else {
        "spot"
    };
    let presale_exported_products =
        safe_parse(row.presale_exported_products.as_deref(), json!([]));

    let sub_orders_json: Vec<Value> = sub_orders
        .map(|subs| subs.iter().map(sub_order_to_json).collect())
        .unwrap_or_default();

    // {...row, ...覆盖字段}：忠实保留 row 的全部原始字段名（驼峰）。
    let mut obj = Map::new();
    obj.insert("id".into(), json!(row.id));
    obj.insert("total".into(), json!(opt_str_num(&row.total)));
    obj.insert("originalTotal".into(), json!(opt_str_num(&row.original_total)));
    obj.insert("discountAmount".into(), json!(opt_str_num(&row.discount_amount)));
    obj.insert("couponCode".into(), json!(row.coupon_code));
    obj.insert("mergeMeta".into(), merge_meta.clone());
    obj.insert("items".into(), items.clone());
    obj.insert("contactName".into(), json!(row.contact_name));
    obj.insert("contactPhone".into(), json!(row.contact_phone));
    obj.insert("contactEmail".into(), json!(row.contact_email));
    obj.insert("province".into(), json!(row.province));
    obj.insert("city".into(), json!(row.city));
    obj.insert("district".into(), json!(row.district));
    obj.insert("addressDetail".into(), json!(row.address_detail));
    obj.insert("trackingCompany".into(), json!(row.tracking_company));
    obj.insert("trackingNo".into(), json!(row.tracking_no));
    obj.insert("status".into(), json!(opt_str_int(&row.status)));
    obj.insert("created_at".into(), json!(row.created_at));
    obj.insert("exported".into(), json!(opt_str_bool_flag(&row.exported)));
    obj.insert("spotExported".into(), json!(opt_str_bool_flag(&row.spot_exported)));
    obj.insert("hasPresaleItems".into(), json!(has_presale_items));
    obj.insert("hasSpotItems".into(), json!(opt_str_bool_flag(&row.has_spot_items)));
    obj.insert("orderType".into(), json!(order_type));
    obj.insert("presaleExportedProducts".into(), presale_exported_products);
    obj.insert("subOrders".into(), json!(sub_orders_json));
    obj.insert(
        "contact".into(),
        json!({
            "name": row.contact_name,
            "phone": row.contact_phone,
            "email": row.contact_email,
            "province": row.province,
            "city": row.city,
            "district": row.district,
            "addressDetail": row.address_detail,
        }),
    );
    Value::Object(obj)
}

/// extractPhoneLast4：取手机号纯数字后四位。
pub fn extract_phone_last4(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return String::new();
    }
    digits[digits.len() - 4..].to_string()
}
