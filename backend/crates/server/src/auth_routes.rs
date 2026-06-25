//! 统一用户级账号系统：注册/登录/登出/邮箱验证/找回密码/改密/资料/会话管理，
//! 全部在 `/api/auth/*`。会话走服务端 sessions 表 + httpOnly cookie（可吊销），
//! 同时仍返回一枚兼容 JWT，使尚未切到 cookie 的旧前端不中断。

use axum::extract::{Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use haruhi_auth::{
    clear_cookie, consume_user_token, cookie_value, create_session, csrf_set_cookie, hash_password,
    issue_token, issue_user_token, revoke_session_by_cookie, revoke_user_sessions, session_id_of,
    session_set_cookie, verify_password, AuthUser, CSRF_COOKIE, SESSION_COOKIE,
};
use haruhi_core::{AppError, AppResult, Config};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ratelimit::client_ip;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/change-password", post(change_password))
        .route("/auth/profile", patch(update_profile))
        .route("/auth/sessions", get(list_sessions))
        .route("/auth/sessions/{id}", delete(revoke_one_session))
}

// ---------- 跨模块共享：终端用户身份（供各模块 UGC 落库署名）----------

/// 终端用户身份键：把账号 id 映射成各模块沿用的 uid 命名空间，
/// 让 art 的积分/创作者/点赞等既有体系无缝复用（"u{id}"）。
pub fn member_uid(user_id: i64) -> String {
    format!("u{user_id}")
}

/// 解析发布署名昵称（落库快照），并确认账号存在且未注销。
/// 注：为降低门槛，已取消「发布前须验证邮箱」的限制——登录即可发布。
/// 函数名保留以兼容各模块调用点。
pub async fn require_verified_member(
    core: &sqlx::SqlitePool,
    user: &AuthUser,
) -> AppResult<String> {
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT nickname, username FROM users WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(user.id)
    .fetch_optional(core)
    .await?;
    let (nickname, username) = row.ok_or(AppError::Unauthorized)?;
    Ok(nickname
        .filter(|s| !s.trim().is_empty())
        .or(username)
        .unwrap_or_else(|| "用户".to_string()))
}

// ---------- 公共助手 ----------

const PASSWORD_MIN: usize = 8;
const VERIFY_TTL: i64 = 86_400; // 邮箱验证链接 24h
const RESET_TTL: i64 = 3_600; // 重置密码链接 1h

/// 极简邮箱格式校验：有 @、@ 后有点、无空格。够挡明显错误，真实性由验证邮件保证。
fn valid_email(s: &str) -> bool {
    let s = s.trim();
    !s.contains(char::is_whitespace)
        && s.split_once('@')
            .map(|(local, domain)| !local.is_empty() && domain.contains('.') && domain.len() >= 3)
            .unwrap_or(false)
}

/// 构造登录态 cookie（会话 httpOnly + csrf 可读）。
fn auth_cookies(cfg: &Config, raw: &str, csrf: &str) -> HeaderMap {
    let mut h = HeaderMap::new();
    let ttl = cfg.session_ttl_seconds;
    h.append(
        SET_COOKIE,
        session_set_cookie(raw, ttl, cfg.cookie_secure)
            .parse()
            .expect("会话 cookie 头应为合法 ASCII"),
    );
    h.append(
        SET_COOKIE,
        csrf_set_cookie(csrf, ttl, cfg.cookie_secure)
            .parse()
            .expect("csrf cookie 头应为合法 ASCII"),
    );
    h
}

/// 清除登录态 cookie。
fn clearing_cookies(cfg: &Config) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.append(
        SET_COOKIE,
        clear_cookie(SESSION_COOKIE, cfg.cookie_secure)
            .parse()
            .unwrap(),
    );
    h.append(
        SET_COOKIE,
        clear_cookie(CSRF_COOKIE, cfg.cookie_secure)
            .parse()
            .unwrap(),
    );
    h
}

fn verify_link(cfg: &Config, raw: &str) -> String {
    format!(
        "{}/verify-email?token={raw}",
        cfg.account_web_base.trim_end_matches('/')
    )
}

fn reset_link(cfg: &Config, raw: &str) -> String {
    format!(
        "{}/reset-password?token={raw}",
        cfg.account_web_base.trim_end_matches('/')
    )
}

/// 发账号类邮件；邮件未启用时把链接打日志（开发期端到端可测）。
async fn deliver(state: &AppState, to: &str, subject: &str, html: &str, text: &str, link: &str) {
    match &state.mailer {
        Some(m) => {
            if let Err(e) = m.send(to, subject, html, text).await {
                tracing::error!("发送账号邮件失败({to}): {e}");
            }
        }
        None => tracing::info!("[邮件未启用] 发往 {to}「{subject}」链接: {link}"),
    }
}

/// 创建会话并附带 cookie + 兼容 JWT，组装登录成功响应。
pub(crate) async fn login_response(
    state: &AppState,
    headers: &HeaderMap,
    user_id: i64,
    is_super: bool,
) -> AppResult<(HeaderMap, Json<Value>)> {
    let ua = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let ip = client_ip(headers);
    let (raw, csrf) = create_session(
        &state.pools.core,
        user_id,
        state.cfg.session_ttl_seconds,
        ua,
        Some(&ip),
    )
    .await?;
    // 兼容 token：让尚未切 cookie 的旧前端仍能工作（cookie 才是主路径）
    let token = issue_token(
        &state.cfg.jwt_secret,
        user_id,
        is_super,
        state.cfg.jwt_ttl_seconds,
    )?;
    sqlx::query("UPDATE users SET last_login_at = datetime('now') WHERE id = ?")
        .bind(user_id)
        .execute(&state.pools.core)
        .await?;
    let profile = load_profile(state, user_id).await?;
    Ok((
        auth_cookies(&state.cfg, &raw, &csrf),
        Json(json!({ "token": token, "user": profile })),
    ))
}

// ---------- 注册 / 登录 / 登出 ----------

#[derive(Deserialize)]
struct RegisterReq {
    email: String,
    password: String,
    #[serde(default)]
    nickname: Option<String>,
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterReq>,
) -> AppResult<(HeaderMap, Json<Value>)> {
    // 防刷邮件 / 防枚举
    let ip = client_ip(&headers);
    if let Err(remaining) = state.account_limiter.check_and_record(&ip) {
        return Err(AppError::TooManyRequests(format!(
            "操作过于频繁，请 {remaining} 秒后再试"
        )));
    }

    let email = req.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(AppError::bad_request("邮箱格式不正确"));
    }
    if req.password.len() < PASSWORD_MIN {
        return Err(AppError::bad_request(format!("密码至少 {PASSWORD_MIN} 位")));
    }
    let nickname = req
        .nickname
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| email.split('@').next().unwrap_or("用户").to_string());

    // 邮箱（或同名 username）已占用？终端用户 username = email
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE email = ? OR username = ?")
            .bind(&email)
            .bind(&email)
            .fetch_optional(&state.pools.core)
            .await?;
    if exists.is_some() {
        return Err(AppError::conflict("该邮箱已注册"));
    }

    let hash = hash_password(&req.password)?;
    // 为降低门槛、规避验证邮件被误判为垃圾邮件：注册即视为已验证，不再发验证邮件。
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, display_name, nickname, email, \
         is_super_admin, status, email_verified) \
         VALUES (?, ?, ?, ?, ?, 0, 'active', 1) RETURNING id",
    )
    .bind(&email)
    .bind(&hash)
    .bind(&nickname)
    .bind(&nickname)
    .bind(&email)
    .fetch_one(&state.pools.core)
    .await?;

    audit(&state, Some(id), "auth", "register", &email).await;
    login_response(&state, &headers, id, false).await
}

#[derive(Deserialize)]
struct LoginReq {
    /// 邮箱或用户名（终端用户用邮箱；老管理员用用户名）
    #[serde(alias = "username", alias = "email")]
    account: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginReq>,
) -> AppResult<(HeaderMap, Json<Value>)> {
    let ip = client_ip(&headers);
    if let Err(remaining) = state.login_limiter.check_and_record(&ip) {
        return Err(AppError::TooManyRequests(format!(
            "登录尝试过于频繁，请 {remaining} 秒后再试"
        )));
    }

    let account = req.account.trim();
    let account_lc = account.to_lowercase();
    let row: Option<(i64, String, bool, String, Option<String>)> = sqlx::query_as(
        "SELECT id, password_hash, is_super_admin, status, deleted_at FROM users \
         WHERE (username = ? OR email = ?) LIMIT 1",
    )
    .bind(account)
    .bind(&account_lc)
    .fetch_optional(&state.pools.core)
    .await?;

    let (id, hash, is_super, status, deleted_at) = row.ok_or(AppError::Unauthorized)?;
    if status != "active" || deleted_at.is_some() {
        return Err(AppError::Forbidden);
    }
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized);
    }
    state.login_limiter.reset(&ip);

    // 两步验证：若该账号已启用 TOTP，则先不发会话，返回短期待验令牌，前端跳二次验证。
    let totp_enabled: Option<bool> =
        sqlx::query_scalar("SELECT enabled FROM user_totp WHERE user_id = ?")
            .bind(id)
            .fetch_optional(&state.pools.core)
            .await?;
    if totp_enabled.unwrap_or(false) {
        let pending = issue_user_token(&state.pools.core, id, "2fa_pending", 300).await?;
        audit(&state, Some(id), "auth", "login_2fa_required", account).await;
        return Ok((
            HeaderMap::new(),
            Json(json!({ "twoFactorRequired": true, "pendingToken": pending })),
        ));
    }

    audit(&state, Some(id), "auth", "login", account).await;
    login_response(&state, &headers, id, is_super).await
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, Json<Value>)> {
    if let Some(raw) = cookie_value(&headers, SESSION_COOKIE) {
        revoke_session_by_cookie(&state.pools.core, &raw).await?;
    }
    Ok((clearing_cookies(&state.cfg), Json(json!({ "ok": true }))))
}

async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let profile = load_profile(&state, user.id).await?;
    Ok(Json(profile))
}

// ---------- 邮箱验证 ----------

#[derive(Deserialize)]
struct TokenReq {
    token: String,
}

async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<TokenReq>,
) -> AppResult<Json<Value>> {
    let uid = consume_user_token(&state.pools.core, req.token.trim(), "verify_email").await?;
    sqlx::query("UPDATE users SET email_verified = 1 WHERE id = ?")
        .bind(uid)
        .execute(&state.pools.core)
        .await?;
    audit(&state, Some(uid), "auth", "verify_email", "").await;
    Ok(Json(json!({ "ok": true })))
}

async fn resend_verification(
    State(state): State<AppState>,
    headers: HeaderMap,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let ip = client_ip(&headers);
    if let Err(remaining) = state.account_limiter.check_and_record(&ip) {
        return Err(AppError::TooManyRequests(format!(
            "操作过于频繁，请 {remaining} 秒后再试"
        )));
    }
    let row: Option<(Option<String>, Option<String>, bool)> =
        sqlx::query_as("SELECT email, nickname, email_verified FROM users WHERE id = ?")
            .bind(user.id)
            .fetch_optional(&state.pools.core)
            .await?;
    let Some((email, nickname, verified)) = row else {
        return Err(AppError::Unauthorized);
    };
    if verified {
        return Ok(Json(json!({ "ok": true, "alreadyVerified": true })));
    }
    let Some(email) = email else {
        return Err(AppError::bad_request("账号未绑定邮箱"));
    };
    let nickname = nickname.unwrap_or_else(|| "用户".to_string());
    let token = issue_user_token(&state.pools.core, user.id, "verify_email", VERIFY_TTL).await?;
    let link = verify_link(&state.cfg, &token);
    deliver(
        &state,
        &email,
        "请验证你的春日应援团账号邮箱",
        &format!("<p>你好 {nickname}，请点击链接验证邮箱（24 小时内有效）：</p><p><a href=\"{link}\">{link}</a></p>"),
        &format!("你好 {nickname}，请打开链接验证邮箱（24 小时内有效）：\n{link}"),
        &link,
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

// ---------- 找回 / 重置 / 修改密码 ----------

#[derive(Deserialize)]
struct ForgotReq {
    email: String,
}

async fn forgot_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ForgotReq>,
) -> AppResult<Json<Value>> {
    let ip = client_ip(&headers);
    // 失败也走限流，避免被当枚举/发信探针
    let _ = state.account_limiter.check_and_record(&ip);

    let email = req.email.trim().to_lowercase();
    if valid_email(&email) {
        let row: Option<(i64, Option<String>)> = sqlx::query_as(
            "SELECT id, nickname FROM users WHERE email = ? AND status = 'active' AND deleted_at IS NULL",
        )
        .bind(&email)
        .fetch_optional(&state.pools.core)
        .await?;
        if let Some((uid, nickname)) = row {
            let nickname = nickname.unwrap_or_else(|| "用户".to_string());
            let token =
                issue_user_token(&state.pools.core, uid, "reset_password", RESET_TTL).await?;
            let link = reset_link(&state.cfg, &token);
            deliver(
                &state,
                &email,
                "重置你的春日应援团账号密码",
                &format!("<p>你好 {nickname}，点击链接重置密码（1 小时内有效）。若非本人操作请忽略：</p><p><a href=\"{link}\">{link}</a></p>"),
                &format!("你好 {nickname}，打开链接重置密码（1 小时内有效），若非本人操作请忽略：\n{link}"),
                &link,
            )
            .await;
            audit(&state, Some(uid), "auth", "forgot_password", &email).await;
        }
    }
    // 一律 200：不泄露邮箱是否注册（防枚举）
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ResetReq {
    token: String,
    password: String,
}

async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetReq>,
) -> AppResult<Json<Value>> {
    if req.password.len() < PASSWORD_MIN {
        return Err(AppError::bad_request(format!("密码至少 {PASSWORD_MIN} 位")));
    }
    let uid = consume_user_token(&state.pools.core, req.token.trim(), "reset_password").await?;
    let hash = hash_password(&req.password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&hash)
        .bind(uid)
        .execute(&state.pools.core)
        .await?;
    // 改密后踢掉所有旧会话（含可能被盗用的）
    revoke_user_sessions(&state.pools.core, uid, None).await?;
    audit(&state, Some(uid), "auth", "reset_password", "").await;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ChangePwReq {
    #[serde(rename = "oldPassword", alias = "old_password")]
    old_password: String,
    #[serde(rename = "newPassword", alias = "new_password")]
    new_password: String,
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    user: AuthUser,
    Json(req): Json<ChangePwReq>,
) -> AppResult<Json<Value>> {
    if req.new_password.len() < PASSWORD_MIN {
        return Err(AppError::bad_request(format!(
            "新密码至少 {PASSWORD_MIN} 位"
        )));
    }
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(&state.pools.core)
        .await?;
    if !verify_password(&req.old_password, &hash) {
        return Err(AppError::bad_request("原密码不正确"));
    }
    let new_hash = hash_password(&req.new_password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    // 踢掉除当前会话外的所有会话
    let current = cookie_value(&headers, SESSION_COOKIE).map(|raw| session_id_of(&raw));
    revoke_user_sessions(&state.pools.core, user.id, current.as_deref()).await?;
    audit(&state, Some(user.id), "auth", "change_password", "").await;
    Ok(Json(json!({ "ok": true })))
}

// ---------- 资料 ----------

#[derive(Deserialize)]
struct ProfileReq {
    #[serde(default)]
    nickname: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
    #[serde(default)]
    bio: Option<String>,
}

async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ProfileReq>,
) -> AppResult<Json<Value>> {
    if let Some(nick) = req.nickname.as_deref().map(str::trim) {
        if nick.is_empty() || nick.chars().count() > 32 {
            return Err(AppError::bad_request("昵称需为 1–32 字"));
        }
        sqlx::query("UPDATE users SET nickname = ? WHERE id = ?")
            .bind(nick)
            .bind(user.id)
            .execute(&state.pools.core)
            .await?;
    }
    if let Some(avatar) = req.avatar.as_deref() {
        sqlx::query("UPDATE users SET avatar = ? WHERE id = ?")
            .bind(avatar.trim())
            .bind(user.id)
            .execute(&state.pools.core)
            .await?;
    }
    if let Some(bio) = req.bio.as_deref() {
        if bio.chars().count() > 280 {
            return Err(AppError::bad_request("简介不超过 280 字"));
        }
        sqlx::query("UPDATE users SET bio = ? WHERE id = ?")
            .bind(bio.trim())
            .bind(user.id)
            .execute(&state.pools.core)
            .await?;
    }
    let profile = load_profile(&state, user.id).await?;
    Ok(Json(profile))
}

// ---------- 会话（设备）管理 ----------

async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let current = cookie_value(&headers, SESSION_COOKIE).map(|raw| session_id_of(&raw));
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, created_at, last_seen_at, user_agent, ip, expires_at FROM sessions \
             WHERE user_id = ? ORDER BY last_seen_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.pools.core)
    .await?;
    let list: Vec<Value> = rows
        .into_iter()
        .map(|(id, created, seen, ua, ip, exp)| {
            json!({
                "id": id,
                "current": current.as_deref() == Some(id.as_str()),
                "createdAt": created,
                "lastSeenAt": seen,
                "userAgent": ua,
                "ip": ip,
                "expiresAt": exp,
            })
        })
        .collect();
    Ok(Json(json!({ "sessions": list })))
}

async fn revoke_one_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    // 限定只能删自己的会话
    sqlx::query("DELETE FROM sessions WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

// ---------- profile + 审计 ----------

/// 组装用户档案 + 各 app 权限矩阵 + 终端用户资料字段。
async fn load_profile(state: &AppState, user_id: i64) -> AppResult<Value> {
    let urow: Option<(
        String,
        Option<String>,
        bool,
        Option<String>,
        bool,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT username, display_name, is_super_admin, email, email_verified, nickname, avatar, bio \
         FROM users WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.pools.core)
    .await?;
    let (username, display_name, is_super, email, email_verified, nickname, avatar, bio) =
        urow.ok_or(AppError::Unauthorized)?;

    let roles: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT ua.app, r.key, r.name, r.level FROM user_app_roles ua \
         JOIN roles r ON r.id = ua.role_id WHERE ua.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&state.pools.core)
    .await?;

    let apps: Value = Value::Object(
        roles
            .into_iter()
            .map(|(app, key, name, level)| {
                (
                    app,
                    json!({ "role": key, "roleName": name, "level": level }),
                )
            })
            .collect(),
    );

    // 账号安全态：是否启用两步验证、是否已有通行密钥（供设置页与守卫判断）。
    let totp_enabled: Option<bool> =
        sqlx::query_scalar("SELECT enabled FROM user_totp WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&state.pools.core)
            .await?;
    let passkey_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_passkeys WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&state.pools.core)
            .await?;

    Ok(json!({
        "id": user_id,
        "username": username,
        "displayName": display_name,
        "nickname": nickname,
        "avatar": avatar,
        "bio": bio,
        "email": email,
        "emailVerified": email_verified,
        "isSuperAdmin": is_super,
        "apps": apps,
        "twoFactorEnabled": totp_enabled.unwrap_or(false),
        "hasPasskey": passkey_count > 0,
    }))
}

/// 写一条审计日志（失败不影响主流程）。
async fn audit(state: &AppState, user_id: Option<i64>, app: &str, action: &str, target: &str) {
    let _ = sqlx::query("INSERT INTO audit_log (user_id, app, action, target) VALUES (?, ?, ?, ?)")
        .bind(user_id)
        .bind(app)
        .bind(action)
        .bind(target)
        .execute(&state.pools.core)
        .await;
}
