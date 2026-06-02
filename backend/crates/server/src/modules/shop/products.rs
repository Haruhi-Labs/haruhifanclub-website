//! shop 商品：公开列表 + 后台增改删 + 库存/预售进度调整 + 图片上传。

use std::collections::HashMap;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Map, Value};

use super::common::*;
use super::pricing::*;
use crate::state::AppState;

/// buildPaidProductQuantityMap：已支付订单的商品累计销量（productId -> qty）。
pub async fn build_paid_product_quantity_map(
    pool: &sqlx::SqlitePool,
) -> AppResult<HashMap<i64, i64>> {
    let placeholders = SALES_VALID_STATUSES
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT items FROM orders WHERE CAST(status AS INTEGER) IN ({placeholders})");
    let mut q = sqlx::query_scalar::<_, Option<String>>(&sql);
    for s in SALES_VALID_STATUSES {
        q = q.bind(s);
    }
    let rows: Vec<Option<String>> = q.fetch_all(pool).await?;
    let mut map: HashMap<i64, i64> = HashMap::new();
    for items_text in rows {
        let items = safe_parse(items_text.as_deref(), json!([]));
        if let Some(arr) = items.as_array() {
            for item in arr {
                let pid = num(item.get("id").unwrap_or(&Value::Null));
                let qty = num(item.get("quantity").unwrap_or(&Value::Null)) as i64;
                if pid.fract() != 0.0 || pid <= 0.0 || qty <= 0 {
                    continue;
                }
                *map.entry(pid as i64).or_insert(0) += qty;
            }
        }
    }
    Ok(map)
}

/// 商品行 -> 完整 JSON（含原始全部列，松类型读取）。
/// 用一个通用读取：SELECT * 后把每列以字符串/数字读出。
#[derive(sqlx::FromRow)]
struct ProductRow {
    id: i64,
    name: Option<String>,
    price: Option<String>,
    #[sqlx(rename = "discountPrice")]
    discount_price: Option<String>,
    category: Option<String>,
    #[sqlx(rename = "typeId")]
    type_id: Option<String>,
    stock: Option<String>,
    image: Option<String>,
    #[sqlx(rename = "imageOriginal")]
    image_original: Option<String>,
    #[sqlx(rename = "imageMobile")]
    image_mobile: Option<String>,
    desc: Option<String>,
    specs: Option<String>,
    #[sqlx(rename = "detailText")]
    detail_text: Option<String>,
    #[sqlx(rename = "detailImages")]
    detail_images: Option<String>,
    #[sqlx(rename = "shippingTag")]
    shipping_tag: Option<String>,
    #[sqlx(rename = "shippingCost")]
    shipping_cost: Option<String>,
    #[sqlx(rename = "presaleMode")]
    presale_mode: Option<String>,
    #[sqlx(rename = "presaleGoalTarget")]
    presale_goal_target: Option<String>,
    #[sqlx(rename = "presaleFixedDateType")]
    presale_fixed_date_type: Option<String>,
    #[sqlx(rename = "presaleFixedDateValue")]
    presale_fixed_date_value: Option<String>,
    #[sqlx(rename = "presalePaidOffset")]
    presale_paid_offset: Option<String>,
    #[sqlx(rename = "sortOrder")]
    sort_order: Option<String>,
}

fn opt_f(s: &Option<String>) -> f64 {
    s.as_deref()
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}
fn opt_i(s: &Option<String>) -> i64 {
    s.as_deref()
        .and_then(|x| {
            x.trim()
                .parse::<i64>()
                .ok()
                .or_else(|| x.trim().parse::<f64>().ok().map(|f| f as i64))
        })
        .unwrap_or(0)
}

/// 商品行转“原始对象”（用于 presale snapshot 与 {...p, 覆盖}）。
fn product_row_to_raw(p: &ProductRow) -> Value {
    json!({
        "id": p.id,
        "name": p.name,
        "price": opt_f(&p.price),
        "discountPrice": p.discount_price.as_deref().filter(|s| !s.is_empty()).map(|s| s.parse::<f64>().unwrap_or(0.0)),
        "category": p.category,
        "typeId": p.type_id,
        "stock": opt_i(&p.stock),
        "image": p.image,
        "imageOriginal": p.image_original,
        "imageMobile": p.image_mobile,
        "desc": p.desc,
        "specs": p.specs,
        "detailText": p.detail_text,
        "detailImages": p.detail_images,
        "shippingTag": p.shipping_tag,
        "shippingCost": opt_f(&p.shipping_cost),
        "presaleMode": p.presale_mode,
        "presaleGoalTarget": opt_i(&p.presale_goal_target),
        "presaleFixedDateType": p.presale_fixed_date_type,
        "presaleFixedDateValue": p.presale_fixed_date_value,
        "presalePaidOffset": opt_i(&p.presale_paid_offset),
        "sortOrder": opt_i(&p.sort_order),
    })
}

/// 商品列查询：数值列统一 CAST AS TEXT（松类型容错，避免 sqlx INTEGER/TEXT 解码 500）。
const PRODUCT_SELECT: &str = "SELECT id, name, CAST(price AS TEXT) AS price, CAST(discountPrice AS TEXT) AS discountPrice, \
    category, typeId, CAST(stock AS TEXT) AS stock, image, imageOriginal, imageMobile, desc, specs, detailText, detailImages, \
    shippingTag, CAST(shippingCost AS TEXT) AS shippingCost, presaleMode, CAST(presaleGoalTarget AS TEXT) AS presaleGoalTarget, \
    presaleFixedDateType, presaleFixedDateValue, CAST(presalePaidOffset AS TEXT) AS presalePaidOffset, CAST(sortOrder AS TEXT) AS sortOrder \
    FROM products";

// GET /products（公开）
pub async fn list_products(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<ProductRow> = sqlx::query_as(&format!(
        "{PRODUCT_SELECT} ORDER BY CAST(sortOrder AS INTEGER) ASC, id ASC"
    ))
    .fetch_all(&state.pools.shop)
    .await?;
    let paid_map = build_paid_product_quantity_map(&state.pools.shop).await?;

    let products: Vec<Value> = rows
        .iter()
        .map(|p| {
            let raw = product_row_to_raw(p);
            let presale = product_presale_snapshot(&raw);
            let paid_count_base = *paid_map.get(&p.id).unwrap_or(&0);
            let paid_offset = opt_i(&p.presale_paid_offset);
            let paid_count = (paid_count_base + paid_offset).max(0);

            // {...p, 覆盖字段}
            let mut obj: Map<String, Value> = raw.as_object().cloned().unwrap_or_default();
            obj.insert("price".into(), json!(opt_f(&p.price)));
            obj.insert(
                "discountPrice".into(),
                match normalize_discount_price(
                    &raw.get("discountPrice").cloned().unwrap_or(Value::Null),
                    &json!(opt_f(&p.price)),
                ) {
                    Some(d) => json!(d),
                    None => Value::Null,
                },
            );
            obj.insert(
                "imageOriginal".into(),
                json!(p.image_original.clone().unwrap_or_default()),
            );
            obj.insert("specs".into(), safe_parse(p.specs.as_deref(), json!([])));
            obj.insert(
                "detailImages".into(),
                safe_parse(p.detail_images.as_deref(), json!([])),
            );
            obj.insert(
                "shippingTag".into(),
                json!(p
                    .shipping_tag
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "default".into())),
            );
            obj.insert("shippingCost".into(), json!(opt_f(&p.shipping_cost)));
            obj.insert("presaleMode".into(), json!(presale.mode));
            obj.insert("presaleGoalTarget".into(), json!(presale.goal_target));
            obj.insert(
                "presaleFixedDateType".into(),
                json!(presale.fixed_date_type),
            );
            obj.insert(
                "presaleFixedDateValue".into(),
                json!(presale.fixed_date_value),
            );
            obj.insert("presalePaidCountBase".into(), json!(paid_count_base));
            obj.insert("presalePaidOffset".into(), json!(paid_offset));
            obj.insert("presalePaidCount".into(), json!(paid_count));
            Value::Object(obj)
        })
        .collect();

    Ok(Json(Value::Array(products)))
}

// POST /upload（Write）：图片上传，转/校验 WebP，存 uploads/shop/
pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_name = String::from("file");
    let mut content_type = String::new();
    let mut purpose = String::new();
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| haruhi_core::AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        match field.name() {
            Some("file") => {
                if let Some(n) = field.file_name() {
                    original_name = n.to_string();
                }
                content_type = field.content_type().unwrap_or("").to_lowercase();
                let bytes = field.bytes().await.map_err(|e| {
                    haruhi_core::AppError::bad_request(format!("读取文件失败: {e}"))
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            Some("purpose") => {
                purpose = field.text().await.unwrap_or_default().trim().to_lowercase();
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let file_bytes = match file_bytes {
        Some(b) => b,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No file uploaded" })),
            )
                .into_response())
        }
    };

    let purpose = if purpose == "qr" {
        "qr"
    } else if purpose == "original" {
        "original"
    } else {
        "general"
    };

    let is_image = content_type.starts_with("image/")
        || matches!(
            ext_of_lower(&original_name).as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
        );
    if !is_image {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "仅支持图片文件" })),
        )
            .into_response());
    }

    let shop_dir = state.cfg.uploads_subdir("shop");
    let uuid = uuid::Uuid::new_v4().to_string();

    // general 类用途：仅接受 WebP（旧实现强制 image/webp 且校验真实 WebP 头）。
    if purpose == "general" {
        let is_webp = content_type == "image/webp" || is_real_webp(&file_bytes);
        if !is_webp {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "非二维码图片仅支持 WebP 上传" })),
            )
                .into_response());
        }
        if !is_real_webp(&file_bytes) {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "WebP 文件内容无效，请更换浏览器后重试" })),
            )
                .into_response());
        }
        let fname = format!("{uuid}.webp");
        haruhi_media::save_file(&shop_dir, &fname, &file_bytes).await?;
        return Ok(Json(json!({ "url": format!("{UPLOAD_URL_PREFIX}/{fname}") })).into_response());
    }

    // qr / original：原样落盘（保留扩展名）。
    let ext = ext_of_lower(&original_name);
    let fname = if ext.is_empty() {
        uuid.to_string()
    } else {
        format!("{uuid}.{ext}")
    };
    haruhi_media::save_file(&shop_dir, &fname, &file_bytes).await?;
    Ok(Json(json!({ "url": format!("{UPLOAD_URL_PREFIX}/{fname}") })).into_response())
}

fn ext_of_lower(name: &str) -> String {
    haruhi_media::ext_of(name, "").to_lowercase()
}

/// 校验真实 WebP 文件头（RIFF....WEBP）。
fn is_real_webp(buf: &[u8]) -> bool {
    buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP"
}

// POST /products（Write）：新增商品
pub async fn create_product(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let clean_price = num(body.get("price").unwrap_or(&Value::Null));
    let stock_v = body.get("stock").cloned().unwrap_or(Value::Null);
    let clean_stock = num(&stock_v);
    let clean_shipping_cost = num(body.get("shippingCost").unwrap_or(&json!(0)));
    let discount_in = body.get("discountPrice").cloned().unwrap_or(Value::Null);
    let clean_discount = normalize_discount_price(&discount_in, &json!(clean_price));
    let presale = match normalize_product_presale_input(&body) {
        Ok(p) => p,
        Err(e) => return Ok(bad(&e)),
    };

    if !clean_price.is_finite() || clean_price < 0.0 {
        return Ok(bad("商品价格无效"));
    }
    if clean_stock.fract() != 0.0 || clean_stock < 0.0 {
        return Ok(bad("库存无效"));
    }
    if !clean_shipping_cost.is_finite() || clean_shipping_cost < 0.0 {
        return Ok(bad("运费无效"));
    }
    if !discount_in.is_null() && !is_empty_str(&discount_in) && clean_discount.is_none() {
        return Ok(bad("折扣价必须大于0且小于原价"));
    }

    let res = sqlx::query(
        "INSERT INTO products (name, price, discountPrice, category, typeId, stock, image, imageMobile, imageOriginal, desc, specs, detailText, detailImages, shippingTag, shippingCost, presaleMode, presaleGoalTarget, presaleFixedDateType, presaleFixedDateValue, presalePaidOffset) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(str_field(&body, "name"))
    .bind(clean_price)
    .bind(clean_discount)
    .bind(str_field(&body, "category"))
    .bind(str_field(&body, "typeId"))
    .bind(clean_stock as i64)
    .bind(body.get("image").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("imageMobile").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("imageOriginal").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("desc").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(json_str(body.get("specs"), "[]"))
    .bind(body.get("detailText").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(json_str(body.get("detailImages"), "[]"))
    .bind(body.get("shippingTag").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("default"))
    .bind(clean_shipping_cost)
    .bind(&presale.mode)
    .bind(presale.goal_target)
    .bind(&presale.fixed_date_type)
    .bind(&presale.fixed_date_value)
    .execute(&state.pools.shop)
    .await?;

    // 返回 { id, ...req.body }
    let mut obj = body.as_object().cloned().unwrap_or_default();
    obj.insert("id".into(), json!(res.last_insert_rowid()));
    Ok(Json(Value::Object(obj)).into_response())
}

// PUT /products/reorder（Write）：批量排序
pub async fn reorder_products(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;
    let order = match body.get("order").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return Ok(bad("参数错误")),
    };
    let mut tx = state.pools.shop.begin().await?;
    for item in order {
        let so = num(item.get("sortOrder").unwrap_or(&Value::Null)) as i64;
        let id = num(item.get("id").unwrap_or(&Value::Null)) as i64;
        sqlx::query("UPDATE products SET sortOrder = ? WHERE id = ?")
            .bind(so)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({ "message": "OK" })).into_response())
}

// PUT /products/:id（Write）：编辑商品（stock 可不传则保留）
pub async fn update_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let product_id = parse_int_radix10(&id).unwrap_or(0);
    let clean_price = num(body.get("price").unwrap_or(&Value::Null));
    let stock_in = body.get("stock").cloned().unwrap_or(Value::Null);
    let has_stock_input = !(stock_in.is_null() || is_empty_str(&stock_in));
    let clean_stock = if has_stock_input {
        Some(num(&stock_in))
    } else {
        None
    };
    let clean_shipping_cost = num(body.get("shippingCost").unwrap_or(&json!(0)));
    let discount_in = body.get("discountPrice").cloned().unwrap_or(Value::Null);
    let clean_discount = normalize_discount_price(&discount_in, &json!(clean_price));
    let presale = match normalize_product_presale_input(&body) {
        Ok(p) => p,
        Err(e) => return Ok(bad(&e)),
    };

    if !clean_price.is_finite() || clean_price < 0.0 {
        return Ok(bad("商品价格无效"));
    }
    if product_id <= 0 {
        return Ok(bad("商品ID无效"));
    }
    if has_stock_input {
        let cs = clean_stock.unwrap();
        if cs.fract() != 0.0 || cs < 0.0 {
            return Ok(bad("库存无效"));
        }
    }
    if !clean_shipping_cost.is_finite() || clean_shipping_cost < 0.0 {
        return Ok(bad("运费无效"));
    }
    if !discount_in.is_null() && !is_empty_str(&discount_in) && clean_discount.is_none() {
        return Ok(bad("折扣价必须大于0且小于原价"));
    }

    let existing: Option<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, CAST(stock AS TEXT) FROM products WHERE id = ?")
            .bind(product_id)
            .fetch_optional(&state.pools.shop)
            .await?;
    let existing = match existing {
        Some(e) => e,
        None => return Ok(not_found("商品不存在")),
    };
    let final_stock = match clean_stock {
        Some(s) => s as i64,
        None => existing
            .1
            .as_deref()
            .and_then(|x| x.trim().parse::<f64>().ok())
            .unwrap_or(0.0) as i64,
    };

    let res = sqlx::query(
        "UPDATE products SET name=?, price=?, discountPrice=?, category=?, typeId=?, stock=?, image=?, imageMobile=?, imageOriginal=?, desc=?, specs=?, detailText=?, detailImages=?, shippingTag=?, shippingCost=?, presaleMode=?, presaleGoalTarget=?, presaleFixedDateType=?, presaleFixedDateValue=? WHERE id=?",
    )
    .bind(str_field(&body, "name"))
    .bind(clean_price)
    .bind(clean_discount)
    .bind(str_field(&body, "category"))
    .bind(str_field(&body, "typeId"))
    .bind(final_stock)
    .bind(body.get("image").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("imageMobile").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("imageOriginal").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(body.get("desc").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(json_str(body.get("specs"), "[]"))
    .bind(body.get("detailText").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(json_str(body.get("detailImages"), "[]"))
    .bind(body.get("shippingTag").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("default"))
    .bind(clean_shipping_cost)
    .bind(&presale.mode)
    .bind(presale.goal_target)
    .bind(&presale.fixed_date_type)
    .bind(&presale.fixed_date_value)
    .bind(product_id)
    .execute(&state.pools.shop)
    .await?;

    Ok(Json(json!({ "message": "Updated", "changes": res.rows_affected() })).into_response())
}

// PUT /admin/products/:id/adjust（Write）：库存/预售进度增量调整
pub async fn adjust_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Write).await?;

    let product_id = parse_int_radix10(&id).unwrap_or(0);
    let stock_delta = normalize_integer_delta(body.get("stockDelta"));
    let presale_delta = normalize_integer_delta(body.get("presalePaidDelta"));

    if product_id <= 0 {
        return Ok(bad("商品ID无效"));
    }
    let (stock_delta, presale_delta) = match (stock_delta, presale_delta) {
        (Some(a), Some(b)) => (a, b),
        _ => return Ok(bad("调整数量必须为整数")),
    };
    if stock_delta == 0 && presale_delta == 0 {
        return Ok(bad("请至少调整一项数量"));
    }

    let product: Option<(i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, CAST(stock AS TEXT), CAST(presalePaidOffset AS TEXT) FROM products WHERE id = ?",
    )
    .bind(product_id)
    .fetch_optional(&state.pools.shop)
    .await?;
    let product = match product {
        Some(p) => p,
        None => return Ok(not_found("商品不存在")),
    };

    let current_stock = product
        .1
        .as_deref()
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(0.0) as i64;
    let current_offset = product
        .2
        .as_deref()
        .and_then(|x| x.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let next_stock = current_stock + stock_delta;
    if next_stock < 0 {
        return Ok(bad(&format!("库存不足，当前库存仅 {current_stock}")));
    }

    let paid_map = build_paid_product_quantity_map(&state.pools.shop).await?;
    let paid_count_base = *paid_map.get(&product_id).unwrap_or(&0);
    let next_offset = current_offset + presale_delta;
    let next_presale_paid = paid_count_base + next_offset;
    if next_presale_paid < 0 {
        return Ok(bad(&format!(
            "预售进度不足，当前仅 {}",
            (paid_count_base + current_offset).max(0)
        )));
    }

    sqlx::query("UPDATE products SET stock = ?, presalePaidOffset = ? WHERE id = ?")
        .bind(next_stock)
        .bind(next_offset)
        .bind(product_id)
        .execute(&state.pools.shop)
        .await?;

    Ok(Json(json!({
        "success": true,
        "productId": product_id,
        "stock": next_stock,
        "presalePaidCountBase": paid_count_base,
        "presalePaidOffset": next_offset,
        "presalePaidCount": next_presale_paid,
    }))
    .into_response())
}

// DELETE /products/:id（Manage）
pub async fn delete_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "shop", Action::Manage).await?;
    let res = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.shop)
        .await?;
    Ok(Json(json!({ "message": "Deleted", "changes": res.rows_affected() })).into_response())
}

// ---------- 工具 ----------

/// normalizeIntegerDeltaInput：空→0；非整数→None；否则数值。
fn normalize_integer_delta(v: Option<&Value>) -> Option<i64> {
    match v {
        None | Some(Value::Null) => Some(0),
        Some(Value::String(s)) if s.is_empty() => Some(0),
        Some(val) => {
            let n = num(val);
            if n.fract() != 0.0 {
                None
            } else {
                Some(n as i64)
            }
        }
    }
}

fn is_empty_str(v: &Value) -> bool {
    matches!(v, Value::String(s) if s.is_empty())
}

fn json_str(v: Option<&Value>, default: &str) -> String {
    match v {
        Some(val) if !val.is_null() => {
            serde_json::to_string(val).unwrap_or_else(|_| default.to_string())
        }
        _ => default.to_string(),
    }
}

pub fn bad(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}
pub fn not_found(msg: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
}
pub fn conflict(msg: &str) -> Response {
    (StatusCode::CONFLICT, Json(json!({ "error": msg }))).into_response()
}
pub fn forbidden(msg: &str) -> Response {
    (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
}
