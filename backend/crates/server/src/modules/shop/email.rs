//! shop 邮件：模板渲染（忠实搬 email.cjs 的 buildOrderEmail）+ email_jobs 入队 + 后台 worker。
//!
//! 与旧实现差异：
//! - 旧 email.cjs 内置 nodemailer/Resend 双驱动 + OAuth2；这里发送统一交给 haruhi_mail::Mailer
//!   （Resend/SMTP 双驱动已封装），worker 只负责轮询队列、退避重试、状态机。
//! - 旧 enqueue 时若 mail 不可用会把 job 直接写成 failed；这里入队恒为 pending，
//!   由 worker 在 Mailer 为 None 时空转（job 保持 pending 等配置就绪），其余语义一致。

use std::sync::Arc;

use haruhi_core::Config;
use serde_json::Value;
use sqlx::SqlitePool;

/// 支持的订单邮件事件。
pub const EMAIL_EVENTS: [&str; 5] = [
    "order_created",
    "order_confirmed",
    "order_shipped",
    "order_completed",
    "order_cancelled",
];

const DEFAULT_BACKOFF_MINUTES: [i64; 5] = [1, 5, 15, 60, 180];

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// formatMoney：¥ + 两位小数千分位。
fn format_money(value: f64) -> String {
    let v = if value.is_finite() { value } else { 0.0 };
    // 千分位
    let neg = v < 0.0;
    let abs = v.abs();
    let cents = (abs * 100.0).round() as i64;
    let int_part = cents / 100;
    let frac = cents % 100;
    let int_str = {
        let s = int_part.to_string();
        let bytes = s.as_bytes();
        let mut out = String::new();
        let len = bytes.len();
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                out.push(',');
            }
            out.push(*b as char);
        }
        out
    };
    format!("¥{}{}.{:02}", if neg { "-" } else { "" }, int_str, frac)
}

/// formatDateTime：尽量贴近 toLocaleString('zh-CN', hour12:false)。无值返回 "-"。
fn format_datetime(value: Option<&str>) -> String {
    let s = match value {
        Some(s) if !s.is_empty() => s,
        _ => return "-".to_string(),
    };
    // 库里多为 "YYYY-MM-DD HH:MM:SS"（UTC）。直接呈现该字符串（与旧 toLocaleString 不完全一致，
    // 但作为通知文案足够；时区换算非业务关键）。
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return ndt.format("%Y/%m/%d %H:%M:%S").to_string();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return dt.format("%Y/%m/%d %H:%M:%S").to_string();
    }
    s.to_string()
}

fn status_text_by_event(event_key: &str, status: i64) -> String {
    match event_key {
        "order_created" => "待付款",
        "order_confirmed" => "待发货",
        "order_shipped" => "已发货",
        "order_completed" => "已完成",
        "order_cancelled" => "已取消",
        _ => match status {
            0 => "已取消",
            1 => "待付款",
            2 => "待发货",
            3 => "已发货",
            4 => "已完成",
            5 => "待确认",
            _ => "状态更新",
        },
    }
    .to_string()
}

fn title_text_by_event(event_key: &str) -> &'static str {
    match event_key {
        "order_created" => "订单已创建",
        "order_confirmed" => "订单已确认",
        "order_shipped" => "订单已发货",
        "order_completed" => "订单已完成",
        "order_cancelled" => "订单已取消",
        _ => "订单状态更新",
    }
}

fn description_text_by_event(event_key: &str) -> &'static str {
    match event_key {
        "order_created" => "我们已收到您的订单，请在24小时内完成支付。",
        "order_confirmed" => {
            "您的支付已确认，订单已进入待发货状态，我们将尽快安排发货。如需修改地址，可在“订单查询”页面操作。"
        }
        "order_shipped" => "您的订单已发货，请留意物流动态。",
        "order_completed" => "订单已完成，感谢您的支持。",
        "order_cancelled" => "因未验证到正确的支付状态，订单已取消，如有疑问可联系管理员。",
        _ => "您的订单状态已更新。",
    }
}

/// 解析 PUBLIC_SITE_URL + basePath 得到“订单查询入口” URL（对齐 resolveOrderQueryUrl）。
fn resolve_order_query_url(public_site_url: &str) -> String {
    let site = public_site_url.trim().trim_end_matches('/');
    if site.is_empty() {
        return String::new();
    }
    // 旧默认 basePath = /shop/，前缀为 /shop。
    format!("{site}/shop/query")
}

/// buildOrderEmail：返回 (subject, html, text)。`order` 为 mapOrderRow 后的 JSON。
pub fn build_order_email(event_key: &str, order: &Value, order_query_url: &str) -> (String, String, String) {
    let status = order.get("status").and_then(|v| v.as_i64()).unwrap_or(0);
    let status_text = status_text_by_event(event_key, status);
    let title = title_text_by_event(event_key);
    let description = description_text_by_event(event_key);
    let order_id = order.get("id").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let created_at_text = format_datetime(order.get("created_at").and_then(|v| v.as_str()));

    let tracking_company = order.get("trackingCompany").and_then(|v| v.as_str()).unwrap_or("");
    let tracking_no = order.get("trackingNo").and_then(|v| v.as_str()).unwrap_or("");
    let tracking_text = if !tracking_company.is_empty() && !tracking_no.is_empty() {
        format!("{tracking_company} / {tracking_no}")
    } else {
        "-".to_string()
    };

    let contact = order.get("contact").cloned().unwrap_or(Value::Null);
    let cget = |k: &str| contact.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let address_text = {
        let a = format!("{}{}{}{}", cget("province"), cget("city"), cget("district"), cget("addressDetail"));
        if a.is_empty() {
            "-".to_string()
        } else {
            a
        }
    };

    let empty = vec![];
    let items = order.get("items").and_then(|v| v.as_array()).unwrap_or(&empty);
    let items_html: String = items
        .iter()
        .map(|it| {
            let name = it.get("name").and_then(|v| v.as_str()).unwrap_or("商品");
            let qty = it.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
            format!("<li>{} × {}</li>", escape_html(name), qty)
        })
        .collect();
    let items_text: String = items
        .iter()
        .map(|it| {
            let name = it.get("name").and_then(|v| v.as_str()).unwrap_or("商品");
            let qty = it.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
            format!("{name} x{qty}")
        })
        .collect::<Vec<_>>()
        .join("；");

    let subject = format!("[春日商城] {title}");
    let total = order.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let contact_name = cget("name");
    let contact_phone = cget("phone");

    let query_hint_html = if !order_query_url.is_empty() {
        format!(
            "<p>订单查询入口：<a href=\"{0}\" target=\"_blank\" rel=\"noreferrer\">{0}</a></p>",
            escape_html(order_query_url)
        )
    } else {
        String::new()
    };
    let query_hint_text = if !order_query_url.is_empty() {
        format!("订单查询入口：{order_query_url}\n")
    } else {
        String::new()
    };

    let items_html_final = if items_html.is_empty() {
        "<li>-</li>".to_string()
    } else {
        items_html
    };

    let html = format!(
        r#"<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,'PingFang SC','Microsoft YaHei',sans-serif;max-width:640px;margin:0 auto;padding:20px;color:#111827;">
  <h2 style="margin:0 0 10px;color:#1d4ed8;">{title_e}</h2>
  <p style="margin:0 0 16px;color:#374151;">{desc_e}</p>

  <div style="background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;padding:14px 16px;">
    <p style="margin:0 0 8px;"><strong>订单号：</strong>{order_id_e}</p>
    <p style="margin:0 0 8px;"><strong>当前状态：</strong>{status_e}</p>
    <p style="margin:0 0 8px;"><strong>下单时间：</strong>{created_e}</p>
    <p style="margin:0 0 8px;"><strong>订单金额：</strong>{money_e}</p>
    <p style="margin:0 0 8px;"><strong>收货人：</strong>{name_e}</p>
    <p style="margin:0 0 8px;"><strong>联系方式：</strong>{phone_e}</p>
    <p style="margin:0 0 8px;"><strong>收货地址：</strong>{addr_e}</p>
    <p style="margin:0;"><strong>物流信息：</strong>{tracking_e}</p>
  </div>

  <div style="margin-top:14px;">
    <p style="margin:0 0 8px;"><strong>商品明细</strong></p>
    <ul style="margin:0;padding-left:18px;">{items_html}</ul>
  </div>

  <div style="margin-top:14px;color:#374151;">
    {query_hint_html}
    <p style="margin:0;">如有问题请通过“联系我们”页面留言，我们会尽快处理。</p>
  </div>

  <p style="margin-top:18px;color:#6b7280;font-size:12px;">本邮件由系统自动发送，请勿直接回复。</p>
</div>"#,
        title_e = escape_html(title),
        desc_e = escape_html(description),
        order_id_e = escape_html(&order_id),
        status_e = escape_html(&status_text),
        created_e = escape_html(&created_at_text),
        money_e = escape_html(&format_money(total)),
        name_e = escape_html(if contact_name.is_empty() { "-" } else { contact_name }),
        phone_e = escape_html(if contact_phone.is_empty() { "-" } else { contact_phone }),
        addr_e = escape_html(&address_text),
        tracking_e = escape_html(&tracking_text),
        items_html = items_html_final,
        query_hint_html = query_hint_html,
    );

    let mut text_lines = vec![
        title.to_string(),
        description.to_string(),
        format!("订单号：{order_id}"),
        format!("当前状态：{status_text}"),
        format!("下单时间：{created_at_text}"),
        format!("订单金额：{}", format_money(total)),
        format!("收货人：{}", if contact_name.is_empty() { "-" } else { contact_name }),
        format!("联系方式：{}", if contact_phone.is_empty() { "-" } else { contact_phone }),
        format!("收货地址：{address_text}"),
        format!("物流信息：{tracking_text}"),
        format!("商品明细：{}", if items_text.is_empty() { "-".to_string() } else { items_text }),
    ];
    let qh = query_hint_text.trim().to_string();
    if !qh.is_empty() {
        text_lines.push(qh);
    }
    text_lines.push("如有问题请通过“联系我们”页面留言。".to_string());
    text_lines.push("本邮件由系统自动发送，请勿直接回复。".to_string());
    let text = text_lines.into_iter().filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n");

    (subject.trim().to_string(), html.trim().to_string(), text)
}

fn is_valid_email(value: &str) -> bool {
    // /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    let v = value.trim();
    let parts: Vec<&str> = v.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || local.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if domain.is_empty() || domain.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    // domain 需含 '.' 且点前后非空
    match domain.rsplit_once('.') {
        Some((a, b)) => !a.is_empty() && !b.is_empty(),
        None => false,
    }
}

/// 入队一条订单邮件 job（对齐 enqueueOrderEmail）。
/// `pool` 可为事务外连接池（本实现统一在 commit 后调用）。
pub async fn enqueue_order_email(
    cfg: &Config,
    pool: &SqlitePool,
    event_key: &str,
    order: &Value,
) {
    if let Err(e) = enqueue_order_email_inner(cfg, pool, event_key, order).await {
        tracing::error!("[Mail] enqueue 失败 event={event_key}: {e}");
    }
}

async fn enqueue_order_email_inner(
    cfg: &Config,
    pool: &SqlitePool,
    event_key: &str,
    order: &Value,
) -> anyhow::Result<()> {
    let order_id = order.get("id").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let to_email = order
        .get("contact")
        .and_then(|c| c.get("email"))
        .and_then(|v| v.as_str())
        .or_else(|| order.get("contactEmail").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if order_id.is_empty() {
        return Ok(());
    }

    let order_query_url = resolve_order_query_url(&cfg.public_site_url);
    let max_attempts = max_attempts(cfg);

    if !EMAIL_EVENTS.contains(&event_key) {
        insert_job(
            pool,
            &order_id,
            if to_email.is_empty() { "-" } else { &to_email },
            event_key,
            "[春日商城] 通知任务创建失败",
            "<p>邮件事件类型不支持。</p>",
            "邮件事件类型不支持。",
            "failed",
            max_attempts,
            Some(&format!("unsupported eventKey: {event_key}")),
        )
        .await?;
        tracing::warn!("[Mail] unsupported eventKey: {event_key}");
        return Ok(());
    }

    let (subject, html, text) = build_order_email(event_key, order, &order_query_url);

    if !is_valid_email(&to_email) {
        insert_job(
            pool,
            &order_id,
            if to_email.is_empty() { "-" } else { &to_email },
            event_key,
            &subject,
            &html,
            &text,
            "failed",
            max_attempts,
            Some("invalid recipient email"),
        )
        .await?;
        tracing::warn!("[Mail] invalid recipient email for order {order_id}: {}", if to_email.is_empty() { "-" } else { &to_email });
        return Ok(());
    }

    insert_job(
        pool, &order_id, &to_email, event_key, &subject, &html, &text, "pending", max_attempts, None,
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_job(
    pool: &SqlitePool,
    order_id: &str,
    to_email: &str,
    event_key: &str,
    subject: &str,
    html: &str,
    text: &str,
    status: &str,
    max_attempts: i64,
    last_error: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO email_jobs (orderId, toEmail, eventKey, subject, html, text, status, attempts, maxAttempts, nextRunAt, lastError) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, CURRENT_TIMESTAMP, ?)",
    )
    .bind(order_id)
    .bind(to_email)
    .bind(event_key)
    .bind(subject)
    .bind(html)
    .bind(text)
    .bind(status)
    .bind(max_attempts)
    .bind(last_error)
    .execute(pool)
    .await?;
    Ok(())
}

fn max_attempts(_cfg: &Config) -> i64 {
    std::env::var("MAIL_MAX_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(5)
}

fn retry_backoffs() -> Vec<i64> {
    let raw = std::env::var("MAIL_RETRY_BACKOFFS_MINUTES").unwrap_or_default();
    let list: Vec<i64> = raw
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .filter(|n| n.is_finite() && *n > 0.0)
        .map(|n| n.floor() as i64)
        .collect();
    if list.is_empty() {
        DEFAULT_BACKOFF_MINUTES.to_vec()
    } else {
        list
    }
}

fn worker_interval_ms() -> u64 {
    std::env::var("MAIL_WORKER_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(10_000)
}

fn worker_batch_size() -> i64 {
    let n = std::env::var("MAIL_WORKER_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(20);
    n.min(200)
}

/// 入队子订单发货邮件（对齐 enqueueSubOrderEmailSafely）：用子订单 items + 物流覆盖订单。
pub async fn enqueue_sub_order_email(
    cfg: &Config,
    pool: &SqlitePool,
    order_json: &Value,
    sub_items: Value,
    tracking_company: &str,
    tracking_no: &str,
) {
    let mut order = order_json.clone();
    if let Some(obj) = order.as_object_mut() {
        obj.insert("items".into(), sub_items);
        obj.insert("trackingCompany".into(), Value::String(tracking_company.to_string()));
        obj.insert("trackingNo".into(), Value::String(tracking_no.to_string()));
    }
    enqueue_order_email(cfg, pool, "order_shipped", &order).await;
}

// ============================================================
// 后台 worker
// ============================================================

#[derive(sqlx::FromRow)]
struct JobRow {
    id: i64,
    #[sqlx(rename = "toEmail")]
    to_email: String,
    #[sqlx(rename = "eventKey")]
    event_key: String,
    subject: String,
    html: String,
    text: String,
    attempts: Option<i64>,
    #[sqlx(rename = "maxAttempts")]
    max_attempts: Option<i64>,
    #[sqlx(rename = "orderId")]
    order_id: String,
}

/// 启动后台邮件 worker：按 interval 轮询 pending 且到期的 job，发送/退避/失败标记。
/// Mailer 为 None（邮件未启用）时不启动 worker（job 留在队列等配置就绪）。
pub fn spawn_email_worker(cfg: Arc<Config>, shop_pool: SqlitePool) {
    let mailer = haruhi_mail::Mailer::from_config(&cfg);
    let Some(mailer) = mailer else {
        tracing::info!("[Mail] 邮件未启用（Mailer=None），不启动 email worker");
        return;
    };

    let interval_ms = worker_interval_ms();
    let batch = worker_batch_size();
    let backoffs = retry_backoffs();
    tracing::info!("[Mail] worker started, interval={interval_ms}ms, batch={batch}");

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
        loop {
            ticker.tick().await;
            if let Err(e) = process_email_jobs(&mailer, &shop_pool, batch, &backoffs).await {
                tracing::error!("[Mail] worker error: {e}");
            }
        }
    });
}

async fn process_email_jobs(
    mailer: &haruhi_mail::Mailer,
    pool: &SqlitePool,
    batch: i64,
    backoffs: &[i64],
) -> anyhow::Result<()> {
    let jobs: Vec<JobRow> = sqlx::query_as(
        "SELECT id, toEmail, eventKey, subject, html, text, CAST(attempts AS INTEGER) AS attempts, \
         CAST(maxAttempts AS INTEGER) AS maxAttempts, orderId \
         FROM email_jobs \
         WHERE status = 'pending' AND DATETIME(nextRunAt) <= DATETIME('now') \
         ORDER BY id ASC LIMIT ?",
    )
    .bind(batch)
    .fetch_all(pool)
    .await?;

    for job in jobs {
        // 抢占 claim：pending -> processing（仅一个 worker 拿到）
        let claim = sqlx::query(
            "UPDATE email_jobs SET status='processing', updated_at=CURRENT_TIMESTAMP WHERE id=? AND status='pending'",
        )
        .bind(job.id)
        .execute(pool)
        .await?;
        if claim.rows_affected() != 1 {
            continue;
        }
        process_one_job(mailer, pool, &job, backoffs).await;
    }
    Ok(())
}

async fn process_one_job(mailer: &haruhi_mail::Mailer, pool: &SqlitePool, job: &JobRow, backoffs: &[i64]) {
    let attempts = job.attempts.unwrap_or(0) + 1;
    let max = {
        let m = job.max_attempts.unwrap_or(5);
        if m > 0 {
            m
        } else {
            5
        }
    };

    match mailer.send(&job.to_email, &job.subject, &job.html, &job.text).await {
        Ok(()) => {
            let _ = sqlx::query(
                "UPDATE email_jobs SET status='sent', attempts=attempts+1, sentAt=CURRENT_TIMESTAMP, \
                 lastError=NULL, updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(job.id)
            .execute(pool)
            .await;
            tracing::info!("[Mail] sent job#{} event={} order={}", job.id, job.event_key, job.order_id);
        }
        Err(err) => {
            let message = err.to_string();
            if attempts >= max {
                let _ = sqlx::query(
                    "UPDATE email_jobs SET status='failed', attempts=?, lastError=?, updated_at=CURRENT_TIMESTAMP WHERE id=?",
                )
                .bind(attempts)
                .bind(truncate(&message, 800))
                .bind(job.id)
                .execute(pool)
                .await;
                tracing::error!("[Mail] failed job#{} event={} order={}: {message}", job.id, job.event_key, job.order_id);
            } else {
                let idx = ((attempts - 1).max(0) as usize).min(backoffs.len().saturating_sub(1));
                let minutes = backoffs.get(idx).copied().unwrap_or(1);
                let next_run = chrono::Utc::now() + chrono::Duration::minutes(minutes);
                let next_run_str = next_run.format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = sqlx::query(
                    "UPDATE email_jobs SET status='pending', attempts=?, nextRunAt=?, lastError=?, updated_at=CURRENT_TIMESTAMP WHERE id=?",
                )
                .bind(attempts)
                .bind(&next_run_str)
                .bind(truncate(&message, 800))
                .bind(job.id)
                .execute(pool)
                .await;
                tracing::warn!("[Mail] retry job#{} attempts={attempts}/{max} next={next_run_str}: {message}", job.id);
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// 取订单 JSON（mapOrderRow 形态）+ 子订单，给邮件用。返回 None 表示订单不存在。
pub async fn load_order_for_email(pool: &SqlitePool, order_id: &str) -> Option<Value> {
    let row: Option<super::common::OrderRow> =
        sqlx::query_as(&format!("{} WHERE id = ?", super::common::ORDER_SELECT)).bind(order_id).fetch_optional(pool).await.ok()?;
    row.map(|r| super::common::map_order_row(&r, None, None))
}
