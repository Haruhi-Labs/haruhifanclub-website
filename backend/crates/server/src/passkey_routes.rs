//! Passkey / WebAuthn：注册管理（登录态）+ 无用户名登录（discoverable）。
//! 路由全部挂在 `/api/auth/passkey/*` 与 `/api/auth/passkeys`。
//!
//! 凭据持久化：`user_passkeys.public_key` 列存放「序列化后的 webauthn-rs Passkey（JSON）」，
//! 登录验证后回写以更新签名计数器。ceremony 中间状态存 `webauthn_states`（5 分钟有效）。
//!
//! rp_id / origin 由 `PUBLIC_SITE_URL` 推导；本地开发（localhost）放开端口。
//! 注意：生产环境各前台同处一个站点域名（同源），rp_id 取该域名即可。

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine;
use haruhi_auth::AuthUser;
use haruhi_core::{AppError, AppResult, Config};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use webauthn_rs::prelude::*;

use crate::auth_routes::login_response;
use crate::state::AppState;

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::URL_SAFE_NO_PAD;
const RP_NAME: &str = "凉宫春日应援团";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/passkey/register/start", post(register_start))
        .route("/auth/passkey/register/finish", post(register_finish))
        .route("/auth/passkeys", get(list_passkeys))
        .route(
            "/auth/passkeys/{id}",
            delete(delete_passkey).patch(rename_passkey),
        )
        .route("/auth/passkey/login/start", post(login_start))
        .route("/auth/passkey/login/finish", post(login_finish))
}

// ---------- 公共助手 ----------

fn random_id() -> String {
    let mut buf = [0u8; 24];
    OsRng.fill_bytes(&mut buf);
    B64.encode(buf)
}

fn wa_err(e: WebauthnError) -> AppError {
    tracing::warn!("webauthn 验证失败: {e:?}");
    AppError::bad_request("通行密钥验证失败，请重试")
}

fn json_err(e: serde_json::Error) -> AppError {
    AppError::internal(format!("通行密钥序列化失败: {e}"))
}

/// 由配置推导 WebAuthn 实例。rp_id = `PUBLIC_SITE_URL` 的主机；本地开发放开端口。
fn build_webauthn(cfg: &Config) -> AppResult<Webauthn> {
    let site = cfg.public_site_url.trim_end_matches('/');
    let url = Url::parse(site).map_err(|_| AppError::internal("PUBLIC_SITE_URL 非法"))?;
    let host = url.host_str().unwrap_or("localhost").to_string();
    let is_local = host == "localhost" || host == "127.0.0.1";
    let mut builder = WebauthnBuilder::new(&host, &url)
        .map_err(|e| AppError::internal(format!("WebAuthn 初始化失败: {e:?}")))?
        .rp_name(RP_NAME);
    if is_local {
        builder = builder.allow_any_port(true);
    }
    builder
        .build()
        .map_err(|e| AppError::internal(format!("WebAuthn 构建失败: {e:?}")))
}

/// 读取某用户全部已注册凭据（含库行 id），用于排除重复注册 / 登录校验 / 计数器回写。
async fn load_user_passkeys(
    core: &sqlx::SqlitePool,
    user_id: i64,
) -> AppResult<Vec<(i64, Passkey)>> {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, public_key FROM user_passkeys WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(core)
            .await?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, blob) in rows {
        match serde_json::from_str::<Passkey>(&blob) {
            Ok(pk) => out.push((id, pk)),
            Err(e) => tracing::warn!("跳过损坏的 passkey 记录 #{id}: {e}"),
        }
    }
    Ok(out)
}

/// 存 ceremony 中间状态（start→finish 之间），5 分钟有效，顺带清理过期。
async fn save_state(
    core: &sqlx::SqlitePool,
    id: &str,
    kind: &str,
    user_id: Option<i64>,
    state_json: &str,
) -> AppResult<()> {
    sqlx::query("DELETE FROM webauthn_states WHERE expires_at < datetime('now')")
        .execute(core)
        .await
        .ok();
    sqlx::query(
        "INSERT INTO webauthn_states (id, kind, user_id, state, expires_at) \
         VALUES (?, ?, ?, ?, datetime('now', '+300 seconds'))",
    )
    .bind(id)
    .bind(kind)
    .bind(user_id)
    .bind(state_json)
    .execute(core)
    .await?;
    Ok(())
}

/// 取出并删除 ceremony 状态（一次性）。返回 (发起用户, 状态 JSON)。
async fn take_state(
    core: &sqlx::SqlitePool,
    id: &str,
    kind: &str,
) -> AppResult<(Option<i64>, String)> {
    let row: Option<(Option<i64>, String)> = sqlx::query_as(
        "SELECT user_id, state FROM webauthn_states \
         WHERE id = ? AND kind = ? AND expires_at >= datetime('now')",
    )
    .bind(id)
    .bind(kind)
    .fetch_optional(core)
    .await?;
    let pair = row.ok_or_else(|| AppError::bad_request("通行密钥会话已过期，请重试"))?;
    sqlx::query("DELETE FROM webauthn_states WHERE id = ?")
        .bind(id)
        .execute(core)
        .await
        .ok();
    Ok(pair)
}

// ---------- 注册（需登录） ----------

async fn register_start(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let webauthn = build_webauthn(&state.cfg)?;
    let (email, nickname): (Option<String>, Option<String>) =
        sqlx::query_as("SELECT email, nickname FROM users WHERE id = ? AND deleted_at IS NULL")
            .bind(user.id)
            .fetch_one(&state.pools.core)
            .await?;
    let name = email.unwrap_or_else(|| format!("u{}", user.id));
    let display = nickname.unwrap_or_else(|| name.clone());

    let existing = load_user_passkeys(&state.pools.core, user.id).await?;
    let exclude: Vec<CredentialID> = existing.iter().map(|(_, p)| p.cred_id().clone()).collect();

    let uuid = Uuid::from_u128(user.id as u128);
    let (ccr, reg) = webauthn
        .start_passkey_registration(uuid, &name, &display, Some(exclude))
        .map_err(wa_err)?;

    let flow = random_id();
    save_state(
        &state.pools.core,
        &flow,
        "reg",
        Some(user.id),
        &serde_json::to_string(&reg).map_err(json_err)?,
    )
    .await?;

    let options = serde_json::to_value(&ccr).map_err(json_err)?;
    Ok(Json(json!({ "flowId": flow, "options": options })))
}

#[derive(Deserialize)]
struct RegFinish {
    #[serde(rename = "flowId")]
    flow_id: String,
    name: Option<String>,
    credential: RegisterPublicKeyCredential,
}

async fn register_finish(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RegFinish>,
) -> AppResult<Json<Value>> {
    let webauthn = build_webauthn(&state.cfg)?;
    let (owner, st) = take_state(&state.pools.core, &req.flow_id, "reg").await?;
    if owner != Some(user.id) {
        return Err(AppError::Unauthorized);
    }
    let reg: PasskeyRegistration = serde_json::from_str(&st).map_err(json_err)?;
    let passkey = webauthn
        .finish_passkey_registration(&req.credential, &reg)
        .map_err(wa_err)?;

    let cred_id_b64 = B64.encode(passkey.cred_id().as_ref());
    let cred_json = serde_json::to_string(&passkey).map_err(json_err)?;
    let name = req
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(40).collect::<String>());

    sqlx::query(
        "INSERT INTO user_passkeys (user_id, credential_id, public_key, name) VALUES (?, ?, ?, ?)",
    )
    .bind(user.id)
    .bind(&cred_id_b64)
    .bind(&cred_json)
    .bind(&name)
    .execute(&state.pools.core)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db) if db.message().contains("UNIQUE") => {
            AppError::conflict("该通行密钥已注册")
        }
        other => AppError::Database(other),
    })?;

    Ok(Json(json!({ "ok": true })))
}

// ---------- 列表 / 删除 / 重命名（需登录） ----------

async fn list_passkeys(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let rows: Vec<(i64, Option<String>, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, created_at, last_used_at FROM user_passkeys \
         WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.pools.core)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, created, last)| {
            json!({ "id": id, "name": name, "createdAt": created, "lastUsedAt": last })
        })
        .collect();
    Ok(Json(json!({ "passkeys": items })))
}

async fn delete_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let res = sqlx::query("DELETE FROM user_passkeys WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("通行密钥不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct Rename {
    name: String,
}

async fn rename_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<Rename>,
) -> AppResult<Json<Value>> {
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(AppError::bad_request("名称需为 1-40 字"));
    }
    let res = sqlx::query("UPDATE user_passkeys SET name = ? WHERE id = ? AND user_id = ?")
        .bind(name)
        .bind(id)
        .bind(user.id)
        .execute(&state.pools.core)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("通行密钥不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

// ---------- 无用户名登录（discoverable） ----------

async fn login_start(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let webauthn = build_webauthn(&state.cfg)?;
    let (rcr, auth_state) = webauthn
        .start_discoverable_authentication()
        .map_err(wa_err)?;
    let flow = random_id();
    save_state(
        &state.pools.core,
        &flow,
        "auth",
        None,
        &serde_json::to_string(&auth_state).map_err(json_err)?,
    )
    .await?;
    let options = serde_json::to_value(&rcr).map_err(json_err)?;
    Ok(Json(json!({ "flowId": flow, "options": options })))
}

#[derive(Deserialize)]
struct LoginFinish {
    #[serde(rename = "flowId")]
    flow_id: String,
    credential: PublicKeyCredential,
}

async fn login_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginFinish>,
) -> AppResult<(HeaderMap, Json<Value>)> {
    let webauthn = build_webauthn(&state.cfg)?;
    let (_owner, st) = take_state(&state.pools.core, &req.flow_id, "auth").await?;
    let auth_state: DiscoverableAuthentication = serde_json::from_str(&st).map_err(json_err)?;

    // 凭据里的 userHandle 即注册时写入的 uuid（from_u128(user_id)）。
    let (user_uuid, _cred) = webauthn
        .identify_discoverable_authentication(&req.credential)
        .map_err(wa_err)?;
    let user_id = user_uuid.as_u128() as i64;

    let passkeys = load_user_passkeys(&state.pools.core, user_id).await?;
    if passkeys.is_empty() {
        return Err(AppError::Unauthorized);
    }
    let dkeys: Vec<DiscoverableKey> = passkeys.iter().map(|(_, p)| DiscoverableKey::from(p)).collect();
    let auth_result = webauthn
        .finish_discoverable_authentication(&req.credential, auth_state, &dkeys)
        .map_err(wa_err)?;

    // 回写签名计数器到匹配的那把 passkey，并刷新 last_used_at。
    for (row_id, pk) in &passkeys {
        if pk.cred_id() == auth_result.cred_id() {
            let mut updated = pk.clone();
            if updated.update_credential(&auth_result).is_some() {
                let blob = serde_json::to_string(&updated).map_err(json_err)?;
                sqlx::query(
                    "UPDATE user_passkeys SET public_key = ?, last_used_at = datetime('now') WHERE id = ?",
                )
                .bind(blob)
                .bind(row_id)
                .execute(&state.pools.core)
                .await?;
            } else {
                sqlx::query("UPDATE user_passkeys SET last_used_at = datetime('now') WHERE id = ?")
                    .bind(row_id)
                    .execute(&state.pools.core)
                    .await
                    .ok();
            }
            break;
        }
    }

    // 校验账号可登录。
    let row: Option<(String, bool)> =
        sqlx::query_as("SELECT status, is_super_admin FROM users WHERE id = ? AND deleted_at IS NULL")
            .bind(user_id)
            .fetch_optional(&state.pools.core)
            .await?;
    let (status, is_super) = row.ok_or(AppError::Unauthorized)?;
    if status != "active" {
        return Err(AppError::Forbidden);
    }

    login_response(&state, &headers, user_id, is_super).await
}
