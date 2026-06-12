//! AI 审核拦截 → 邮件通知管理员。
//!
//! 收件人：`status='active'` 且填了邮箱的用户中，**超管 ∪ 在该模块有角色的管理员**。
//! 超管默认会收到（前提是给自己的账号填了邮箱）；模块管理员由超管在 console 配置邮箱后也会收到。
//! 邮件未启用 / 未配置时静默跳过；任何失败仅记日志，**绝不影响主流程**。

use haruhi_core::Config;
use sqlx::SqlitePool;

use crate::state::AppState;

/// 异步触发：克隆所需状态后 `spawn`，立即返回，不阻塞请求处理。
pub fn ai_flagged(
    state: &AppState,
    app: &str,
    kind: &str,
    title: &str,
    item_id: &str,
    reason: &str,
) {
    let core = state.pools.core.clone();
    let cfg = state.cfg.clone();
    let app = app.to_string();
    let kind = kind.to_string();
    let title = title.to_string();
    let item_id = item_id.to_string();
    let reason = reason.to_string();
    tokio::spawn(async move {
        send_ai_flagged(&core, &cfg, &app, &kind, &title, &item_id, &reason).await;
    });
}

/// 实际查询收件人并逐个发信。可被「已在 spawn 内」的调用方（如 exam 异步审核）直接 `await`。
pub async fn send_ai_flagged(
    core: &SqlitePool,
    cfg: &Config,
    app: &str,
    kind: &str,
    title: &str,
    item_id: &str,
    reason: &str,
) {
    if !cfg.mail.enabled {
        return;
    }
    let mailer = match haruhi_mail::Mailer::from_config(cfg) {
        Some(m) => m,
        None => return,
    };

    let recipients: Vec<(String,)> = match sqlx::query_as(
        "SELECT DISTINCT email FROM users \
         WHERE status = 'active' AND email IS NOT NULL AND TRIM(email) <> '' \
           AND (is_super_admin = 1 OR id IN (SELECT user_id FROM user_app_roles WHERE app = ?))",
    )
    .bind(app)
    .fetch_all(core)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("[AI通知] 查询收件人失败: {e}");
            return;
        }
    };
    if recipients.is_empty() {
        tracing::info!("[AI通知] {app} 模块暂无可通知的管理员邮箱，跳过");
        return;
    }

    let label = app_label(app);
    let url = admin_url(app);
    let reason = if reason.trim().is_empty() {
        "未提供"
    } else {
        reason
    };
    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };

    let subject = format!("【凉宫春日应援团·{label}】AI 审核拦截了一项{kind}，待处理");
    let html = format!(
        "<div style=\"font-family:system-ui,'Segoe UI',Arial,sans-serif;max-width:560px;margin:0 auto;color:#1f2328\">\
         <h2 style=\"font-size:17px;margin:0 0 12px\">AI 审核拦截了一项{kind}</h2>\
         <table style=\"border-collapse:collapse;font-size:14px;line-height:1.7\">\
         <tr><td style=\"color:#6b7280;padding-right:14px\">模块</td><td>{label}（{app}）</td></tr>\
         <tr><td style=\"color:#6b7280;padding-right:14px\">类型</td><td>{kind}</td></tr>\
         <tr><td style=\"color:#6b7280;padding-right:14px\">标题/内容</td><td>{title}</td></tr>\
         <tr><td style=\"color:#6b7280;padding-right:14px\">ID</td><td>{item_id}</td></tr>\
         <tr><td style=\"color:#6b7280;padding-right:14px\">拦截原因</td><td>{reason}</td></tr>\
         </table>\
         <p style=\"margin:18px 0 0\"><a href=\"{url}\" style=\"background:#D97757;color:#fff;text-decoration:none;padding:9px 16px;border-radius:8px;display:inline-block\">去后台处理 →</a></p>\
         <p style=\"color:#9ca3af;font-size:12px;margin-top:18px\">本邮件由凉宫春日应援团统一后端在 AI 审核拦截内容时自动发送。你收到是因为你是超级管理员或该模块的管理员。</p>\
         </div>",
        kind = esc(kind),
        label = esc(&label),
        app = esc(app),
        title = esc(title),
        item_id = esc(item_id),
        reason = esc(reason),
        url = url,
    );
    let text = format!(
        "AI 审核拦截了一项{kind}\n模块：{label}（{app}）\n标题/内容：{title}\nID：{item_id}\n拦截原因：{reason}\n去后台处理：{url}\n"
    );

    for (email,) in recipients {
        match mailer.send(&email, &subject, &html, &text).await {
            Ok(()) => tracing::info!("[AI通知] 已通知 {email}（{app}/{kind} #{item_id}）"),
            Err(e) => tracing::error!("[AI通知] 发送给 {email} 失败: {e}"),
        }
    }
}

fn app_label(app: &str) -> String {
    match app {
        "art" => "画廊",
        "exam" => "考试",
        "news" => "新闻",
        "novel" => "书库",
        "shop" => "商城",
        other => other,
    }
    .to_string()
}

fn admin_url(app: &str) -> String {
    let base = "https://haruyuki.cn";
    match app {
        "art" => format!("{base}/art/admin"),
        "exam" => format!("{base}/exam/admin"),
        other => format!("{base}/{other}/"),
    }
}
