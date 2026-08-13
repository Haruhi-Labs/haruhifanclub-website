//! 游戏统一身份客户端：Authorization Code + PKCE、独立 host-only 会话与 Ed25519 短期票据。
//!
//! 主站会话只用于授权页；游戏通过同源代理交换授权码后获得自己的 Cookie 名称，两个站点
//! 不共享父域 Cookie。游客无需访问本模块，游戏仍可匿名运行。

use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, PRAGMA, SET_COOKIE};
use axum::http::HeaderMap;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use haruhi_auth::{
    cookie_value, create_session, hash_token, lookup_session, revoke_session_by_cookie, AuthUser,
};
use haruhi_core::{AppError, AppResult, Config};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::ratelimit::client_ip;
use crate::state::AppState;

pub const GAME_CLIENT_ID: &str = "star-game";
pub const GAME_TICKET_AUDIENCE: &str = "haruhi-game-ws";
pub const GAME_TICKET_KID: &str = "star-game-v1";
pub const GAME_SESSION_COOKIE: &str = "__Host-haruhi_game_session";
pub const GAME_CSRF_COOKIE: &str = "__Host-haruhi_game_csrf";
pub const GAME_SESSION_COOKIE_DEV: &str = "haruhi_game_session";
pub const GAME_CSRF_COOKIE_DEV: &str = "haruhi_game_csrf";

const AUTHORIZATION_CODE_TTL: i64 = 60;
const GAME_TICKET_TTL: i64 = 60;

#[derive(Clone)]
pub struct GameTicketSigner {
    signing_key: SigningKey,
    issuer: String,
}

impl GameTicketSigner {
    pub fn from_config(cfg: &Config) -> anyhow::Result<Self> {
        let decoded = URL_SAFE_NO_PAD
            .decode(cfg.game_ticket_private_key.trim())
            .map_err(|e| {
                anyhow::anyhow!("HARUHI_GAME_TICKET_PRIVATE_KEY 不是合法 base64url: {e}")
            })?;
        let seed: [u8; 32] = decoded.try_into().map_err(|bytes: Vec<u8>| {
            anyhow::anyhow!(
                "HARUHI_GAME_TICKET_PRIVATE_KEY 解码后必须为 32 字节，实际 {} 字节",
                bytes.len()
            )
        })?;
        Ok(Self {
            signing_key: SigningKey::from_bytes(&seed),
            issuer: cfg.public_site_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn public_key_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.signing_key.verifying_key().as_bytes())
    }

    fn issue(&self, identity: &GameIdentity) -> AppResult<String> {
        let now = chrono::Utc::now().timestamp();
        let header = json!({ "alg": "EdDSA", "typ": "JWT", "kid": GAME_TICKET_KID });
        let claims = GameTicketClaims {
            issuer: &self.issuer,
            subject: &identity.id,
            audience: GAME_TICKET_AUDIENCE,
            nickname: &identity.nickname,
            avatar: identity.avatar.as_deref(),
            issued_at: now,
            expires_at: now + GAME_TICKET_TTL,
            token_id: Uuid::new_v4().to_string(),
        };
        let header = serde_json::to_vec(&header)
            .map(|value| URL_SAFE_NO_PAD.encode(value))
            .map_err(|e| AppError::internal(format!("序列化游戏票据头失败: {e}")))?;
        let claims = serde_json::to_vec(&claims)
            .map(|value| URL_SAFE_NO_PAD.encode(value))
            .map_err(|e| AppError::internal(format!("序列化游戏票据失败: {e}")))?;
        let signing_input = format!("{header}.{claims}");
        let signature = self.signing_key.sign(signing_input.as_bytes());
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(signature.to_bytes())
        ))
    }
}

#[derive(Serialize)]
struct GameTicketClaims<'a> {
    #[serde(rename = "iss")]
    issuer: &'a str,
    #[serde(rename = "sub")]
    subject: &'a str,
    #[serde(rename = "aud")]
    audience: &'a str,
    nickname: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<&'a str>,
    #[serde(rename = "iat")]
    issued_at: i64,
    #[serde(rename = "exp")]
    expires_at: i64,
    #[serde(rename = "jti")]
    token_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameIdentity {
    id: String,
    nickname: String,
    avatar: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/game/authorize", get(authorize))
        .route("/game/session/exchange", post(exchange_code))
        .route("/game/session", get(game_session))
        .route("/game/session/logout", post(game_logout))
        .route("/game/ticket", post(game_ticket))
        .route("/game/jwks", get(game_jwks))
}

#[derive(Deserialize)]
struct AuthorizeQuery {
    client_id: String,
    redirect_uri: String,
    code_challenge: String,
    code_challenge_method: String,
    state: String,
}

async fn authorize(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<AuthorizeQuery>,
) -> AppResult<(HeaderMap, Redirect)> {
    validate_client(&state.cfg, &query.client_id, &query.redirect_uri)?;
    if query.code_challenge_method != "S256" || !valid_s256_challenge(&query.code_challenge) {
        return Err(AppError::bad_request("仅支持合法的 PKCE S256 challenge"));
    }
    if query.state.is_empty() || query.state.len() > 512 {
        return Err(AppError::bad_request("state 缺失或过长"));
    }

    let code = random_token();
    let token_hash = hash_token(&code);
    let _ = sqlx::query(
        "DELETE FROM game_authorization_codes \
         WHERE expires_at <= datetime('now') OR consumed_at <= datetime('now', '-5 minutes')",
    )
    .execute(&state.pools.core)
    .await;
    sqlx::query(
        "INSERT INTO game_authorization_codes \
         (token_hash, user_id, client_id, redirect_uri, code_challenge, expires_at) \
         VALUES (?, ?, ?, ?, ?, datetime('now', ?))",
    )
    .bind(token_hash)
    .bind(user.id)
    .bind(GAME_CLIENT_ID)
    .bind(&query.redirect_uri)
    .bind(&query.code_challenge)
    .bind(format!("+{AUTHORIZATION_CODE_TTL} seconds"))
    .execute(&state.pools.core)
    .await?;

    let mut callback = Url::parse(&query.redirect_uri)
        .map_err(|_| AppError::bad_request("redirect_uri 不合法"))?;
    callback
        .query_pairs_mut()
        .append_pair("code", &code)
        .append_pair("state", &query.state)
        .append_pair("iss", state.cfg.public_site_url.trim_end_matches('/'));
    Ok((no_store_headers(), Redirect::to(callback.as_str())))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangeRequest {
    client_id: String,
    redirect_uri: String,
    code: String,
    code_verifier: String,
}

async fn exchange_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ExchangeRequest>,
) -> AppResult<(HeaderMap, Json<Value>)> {
    validate_client(&state.cfg, &request.client_id, &request.redirect_uri)?;
    if !valid_code_verifier(&request.code_verifier) {
        return Err(AppError::bad_request("PKCE verifier 不合法"));
    }

    // UPDATE ... RETURNING 令授权码在并发交换中只可能被一个请求消费。
    // 合法格式但不匹配的 verifier 也会烧毁授权码，避免对 challenge 反复试探。
    let consumed: Option<(i64, String)> = sqlx::query_as(
        "UPDATE game_authorization_codes SET consumed_at = datetime('now') \
         WHERE token_hash = ? AND client_id = ? AND redirect_uri = ? \
           AND consumed_at IS NULL AND expires_at > datetime('now') \
         RETURNING user_id, code_challenge",
    )
    .bind(hash_token(request.code.trim()))
    .bind(GAME_CLIENT_ID)
    .bind(&request.redirect_uri)
    .fetch_optional(&state.pools.core)
    .await?;
    let (user_id, expected_challenge) =
        consumed.ok_or_else(|| AppError::bad_request("授权码无效、已使用或已过期"))?;
    let actual_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(request.code_verifier.as_bytes()));
    if actual_challenge != expected_challenge {
        return Err(AppError::bad_request("PKCE 校验失败，请重新登录"));
    }

    let identity = load_identity(&state, user_id).await?;
    let ip = client_ip(&headers);
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let (raw, csrf) = create_session(
        &state.pools.core,
        user_id,
        state.cfg.session_ttl_seconds,
        user_agent,
        Some(&ip),
    )
    .await?;
    let cookies = game_cookies(&state.cfg, &raw, &csrf);
    audit(&state, user_id, "game_session_created").await;
    Ok((cookies, Json(json!({ "user": identity }))))
}

async fn game_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, Json<Value>)> {
    let user_id = game_user_id(&state, &headers).await?;
    let identity = load_identity(&state, user_id).await?;
    Ok((no_store_headers(), Json(json!({ "user": identity }))))
}

async fn game_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, Json<Value>)> {
    if let Some(raw) = game_cookie_value(&state.cfg, &headers) {
        revoke_session_by_cookie(&state.pools.core, &raw).await?;
    }
    Ok((clear_game_cookies(&state.cfg), Json(json!({ "ok": true }))))
}

async fn game_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, Json<Value>)> {
    let user_id = game_user_id(&state, &headers).await?;
    let identity = load_identity(&state, user_id).await?;
    let ticket = state.game_ticket_signer.issue(&identity)?;
    Ok((
        no_store_headers(),
        Json(json!({
            "ticket": ticket,
            "expiresIn": GAME_TICKET_TTL,
            "user": identity,
        })),
    ))
}

async fn game_jwks(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "keys": [{
            "kty": "OKP",
            "crv": "Ed25519",
            "use": "sig",
            "alg": "EdDSA",
            "kid": GAME_TICKET_KID,
            "x": state.game_ticket_signer.public_key_base64(),
        }]
    }))
}

fn validate_client(cfg: &Config, client_id: &str, redirect_uri: &str) -> AppResult<()> {
    if client_id != GAME_CLIENT_ID {
        return Err(AppError::bad_request("未知的统一身份客户端"));
    }
    if !cfg
        .game_sso_redirect_uris
        .iter()
        .any(|allowed| allowed == redirect_uri)
    {
        return Err(AppError::bad_request("redirect_uri 未登记"));
    }
    let parsed =
        Url::parse(redirect_uri).map_err(|_| AppError::bad_request("redirect_uri 不合法"))?;
    if parsed.fragment().is_some() || parsed.username() != "" || parsed.password().is_some() {
        return Err(AppError::bad_request("redirect_uri 不合法"));
    }
    let local_http = parsed.scheme() == "http"
        && matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if parsed.scheme() != "https" && !local_http {
        return Err(AppError::bad_request(
            "redirect_uri 必须使用 HTTPS（仅本机开发可用 HTTP）",
        ));
    }
    Ok(())
}

fn valid_s256_challenge(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_code_verifier(value: &str) -> bool {
    (43..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn game_user_id(state: &AppState, headers: &HeaderMap) -> AppResult<i64> {
    let raw = game_cookie_value(&state.cfg, headers).ok_or(AppError::Unauthorized)?;
    let session = lookup_session(&state.pools.core, &raw)
        .await?
        .ok_or(AppError::Unauthorized)?;
    Ok(session.user.id)
}

fn game_cookie_value(cfg: &Config, headers: &HeaderMap) -> Option<String> {
    cookie_value(headers, game_session_cookie_name(cfg))
}

async fn load_identity(state: &AppState, user_id: i64) -> AppResult<GameIdentity> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT nickname, avatar FROM users \
         WHERE id = ? AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.pools.core)
    .await?;
    let (nickname, avatar) = row.ok_or(AppError::Unauthorized)?;
    Ok(GameIdentity {
        id: format!("u{user_id}"),
        nickname,
        avatar,
    })
}

fn game_session_cookie_name(cfg: &Config) -> &'static str {
    if cfg.cookie_secure {
        GAME_SESSION_COOKIE
    } else {
        GAME_SESSION_COOKIE_DEV
    }
}

fn game_csrf_cookie_name(cfg: &Config) -> &'static str {
    if cfg.cookie_secure {
        GAME_CSRF_COOKIE
    } else {
        GAME_CSRF_COOKIE_DEV
    }
}

fn game_cookies(cfg: &Config, raw: &str, csrf: &str) -> HeaderMap {
    let secure = if cfg.cookie_secure { "; Secure" } else { "" };
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        format!(
            "{}={raw}; HttpOnly{secure}; SameSite=Lax; Path=/; Max-Age={}",
            game_session_cookie_name(cfg),
            cfg.session_ttl_seconds
        )
        .parse()
        .expect("游戏会话 Cookie 应为合法 ASCII"),
    );
    headers.append(
        SET_COOKIE,
        format!(
            "{}={csrf}; SameSite=Lax; Path=/; Max-Age={}{}",
            game_csrf_cookie_name(cfg),
            cfg.session_ttl_seconds,
            secure
        )
        .parse()
        .expect("游戏 CSRF Cookie 应为合法 ASCII"),
    );
    insert_no_store(&mut headers);
    headers
}

fn clear_game_cookies(cfg: &Config) -> HeaderMap {
    let secure = if cfg.cookie_secure { "; Secure" } else { "" };
    let mut headers = HeaderMap::new();
    for (name, attributes) in [
        (game_session_cookie_name(cfg), "HttpOnly; "),
        (game_csrf_cookie_name(cfg), ""),
    ] {
        headers.append(
            SET_COOKIE,
            format!("{name}=; {attributes}SameSite=Lax; Path=/; Max-Age=0{secure}")
                .parse()
                .expect("清除游戏 Cookie 头应为合法 ASCII"),
        );
    }
    insert_no_store(&mut headers);
    headers
}

fn insert_no_store(headers: &mut HeaderMap) {
    headers.insert(CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert(PRAGMA, "no-cache".parse().unwrap());
}

fn no_store_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    insert_no_store(&mut headers);
    headers
}

async fn audit(state: &AppState, user_id: i64, action: &str) {
    let _ = sqlx::query(
        "INSERT INTO audit_log (user_id, app, action, target) VALUES (?, 'game', ?, ?)",
    )
    .bind(user_id)
    .bind(action)
    .bind(GAME_CLIENT_ID)
    .execute(&state.pools.core)
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_validation_is_strict() {
        assert!(valid_s256_challenge(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        ));
        assert!(!valid_s256_challenge("short"));
        assert!(valid_code_verifier(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ"
        ));
        assert!(!valid_code_verifier("contains+invalid/chars"));
    }
}
