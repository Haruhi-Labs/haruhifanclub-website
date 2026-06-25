//! 两步验证（2FA / TOTP）：设置 / 启用 / 停用 / 登录二次验证。
//! 路由挂在 `/api/auth/2fa/*`。
//!
//! - 密钥（base32）存 `user_totp.secret`，`enabled` 标志启用态；
//! - 备用恢复码以 sha256 存 `user_backup_codes`，一次性；
//! - 登录时若已启用，password 校验后由 `auth_routes::login` 返回 `pendingToken`，
//!   再调本模块 `/auth/2fa/login` 完成二次验证后建会话。
//!   令牌「先校验后作废」：验码失败不消费，可重试。

use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use haruhi_auth::{hash_token, verify_password, AuthUser};
use haruhi_core::{AppError, AppResult};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::auth_routes::login_response;
use crate::state::AppState;

const ISSUER: &str = "凉宫春日应援团";
const BACKUP_CODE_COUNT: usize = 10;
// 备用码字母表：去掉易混的 0/O/1/I/L
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/2fa/setup", post(setup))
        .route("/auth/2fa/enable", post(enable))
        .route("/auth/2fa/disable", post(disable))
        .route("/auth/2fa/login", post(login_2fa))
}

// ---------- 助手 ----------

fn sha256_b64(s: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(s.as_bytes()))
}

fn gen_backup_code() -> String {
    let mut buf = [0u8; 10];
    OsRng.fill_bytes(&mut buf);
    let chars: String = buf
        .iter()
        .map(|b| CODE_ALPHABET[(*b as usize) % CODE_ALPHABET.len()] as char)
        .collect();
    format!("{}-{}", &chars[..5], &chars[5..])
}

fn build_totp(secret_b32: &str, account: &str) -> AppResult<TOTP> {
    let bytes = Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|_| AppError::internal("TOTP 密钥解析失败"))?;
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes,
        Some(ISSUER.to_string()),
        account.to_string(),
    )
    .map_err(|e| AppError::internal(format!("TOTP 构造失败: {e}")))
}

async fn account_label(core: &sqlx::SqlitePool, user_id: i64) -> String {
    let email: Option<String> = sqlx::query_scalar("SELECT email FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(core)
        .await
        .ok()
        .flatten()
        .flatten();
    email.unwrap_or_else(|| format!("u{user_id}"))
}

// ---------- 设置 / 启用 / 停用（需登录） ----------

async fn setup(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let enabled: Option<bool> =
        sqlx::query_scalar("SELECT enabled FROM user_totp WHERE user_id = ?")
            .bind(user.id)
            .fetch_optional(&state.pools.core)
            .await?;
    if enabled == Some(true) {
        return Err(AppError::conflict("两步验证已启用"));
    }

    let account = account_label(&state.pools.core, user.id).await;
    let secret = Secret::generate_secret();
    let secret_b32 = match secret.to_encoded() {
        Secret::Encoded(s) => s,
        Secret::Raw(_) => return Err(AppError::internal("TOTP 密钥编码失败")),
    };
    let totp = build_totp(&secret_b32, &account)?;
    let uri = totp.get_url();

    // 暂存密钥（未启用），覆盖任何旧的未完成设置。
    sqlx::query(
        "INSERT INTO user_totp (user_id, secret, enabled) VALUES (?, ?, 0) \
         ON CONFLICT(user_id) DO UPDATE SET secret = excluded.secret, enabled = 0, confirmed_at = NULL",
    )
    .bind(user.id)
    .bind(&secret_b32)
    .execute(&state.pools.core)
    .await?;

    // 重新生成一组一次性备用码。
    sqlx::query("DELETE FROM user_backup_codes WHERE user_id = ?")
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    let mut codes = Vec::with_capacity(BACKUP_CODE_COUNT);
    for _ in 0..BACKUP_CODE_COUNT {
        let c = gen_backup_code();
        sqlx::query("INSERT INTO user_backup_codes (user_id, code_hash) VALUES (?, ?)")
            .bind(user.id)
            .bind(sha256_b64(&c))
            .execute(&state.pools.core)
            .await?;
        codes.push(c);
    }

    Ok(Json(
        json!({ "otpauthUri": uri, "secret": secret_b32, "backupCodes": codes }),
    ))
}

#[derive(Deserialize)]
struct CodeReq {
    code: String,
}

async fn enable(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CodeReq>,
) -> AppResult<Json<Value>> {
    let row: Option<(String, bool)> =
        sqlx::query_as("SELECT secret, enabled FROM user_totp WHERE user_id = ?")
            .bind(user.id)
            .fetch_optional(&state.pools.core)
            .await?;
    let (secret, enabled) = row.ok_or_else(|| AppError::bad_request("请先开始设置两步验证"))?;
    if enabled {
        return Err(AppError::conflict("两步验证已启用"));
    }
    let account = account_label(&state.pools.core, user.id).await;
    let totp = build_totp(&secret, &account)?;
    let ok = totp
        .check_current(req.code.trim())
        .map_err(|_| AppError::internal("时间校验失败"))?;
    if !ok {
        return Err(AppError::bad_request("验证码不正确"));
    }
    sqlx::query(
        "UPDATE user_totp SET enabled = 1, confirmed_at = datetime('now') WHERE user_id = ?",
    )
    .bind(user.id)
    .execute(&state.pools.core)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct PwReq {
    password: String,
}

async fn disable(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<PwReq>,
) -> AppResult<Json<Value>> {
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(&state.pools.core)
        .await?;
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized);
    }
    sqlx::query("DELETE FROM user_totp WHERE user_id = ?")
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    sqlx::query("DELETE FROM user_backup_codes WHERE user_id = ?")
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

// ---------- 登录二次验证（无需登录态，凭 pendingToken） ----------

#[derive(Deserialize)]
struct Login2faReq {
    #[serde(rename = "pendingToken")]
    pending_token: String,
    code: String,
    #[serde(default)]
    backup: bool,
}

async fn login_2fa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Login2faReq>,
) -> AppResult<(HeaderMap, Json<Value>)> {
    // peek：先校验令牌有效并取 user_id，但不消费（验码失败可重试）。
    let token_hash = hash_token(req.pending_token.trim());
    let user_id: Option<i64> = sqlx::query_scalar(
        "SELECT user_id FROM user_tokens \
         WHERE token_hash = ? AND kind = '2fa_pending' AND used_at IS NULL \
         AND expires_at > datetime('now')",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pools.core)
    .await?;
    let user_id = user_id.ok_or(AppError::Unauthorized)?;

    let code = req.code.trim();
    let verified = if req.backup {
        let h = sha256_b64(code);
        let bid: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM user_backup_codes WHERE user_id = ? AND code_hash = ? AND used_at IS NULL",
        )
        .bind(user_id)
        .bind(&h)
        .fetch_optional(&state.pools.core)
        .await?;
        if let Some(bid) = bid {
            sqlx::query("UPDATE user_backup_codes SET used_at = datetime('now') WHERE id = ?")
                .bind(bid)
                .execute(&state.pools.core)
                .await?;
            true
        } else {
            false
        }
    } else {
        let secret: Option<String> =
            sqlx::query_scalar("SELECT secret FROM user_totp WHERE user_id = ? AND enabled = 1")
                .bind(user_id)
                .fetch_optional(&state.pools.core)
                .await?;
        let secret = secret.ok_or(AppError::Unauthorized)?;
        let account = account_label(&state.pools.core, user_id).await;
        build_totp(&secret, &account)?
            .check_current(code)
            .unwrap_or(false)
    };

    if !verified {
        return Err(AppError::bad_request("验证码不正确"));
    }

    // 验码通过，作废待验令牌（一次性）。
    sqlx::query("UPDATE user_tokens SET used_at = datetime('now') WHERE token_hash = ? AND kind = '2fa_pending'")
        .bind(&token_hash)
        .execute(&state.pools.core)
        .await
        .ok();

    let is_super: bool =
        sqlx::query_scalar("SELECT is_super_admin FROM users WHERE id = ? AND deleted_at IS NULL")
            .bind(user_id)
            .fetch_one(&state.pools.core)
            .await?;
    login_response(&state, &headers, user_id, is_super).await
}
