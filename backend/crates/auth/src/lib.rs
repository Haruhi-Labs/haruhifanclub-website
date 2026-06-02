//! haruhi-auth：统一 JWT 单点登录 + argon2 密码 + RBAC 授权。
//!
//! 鉴权身份来自 JWT（无状态）；权限判定每次查 core.db（保证角色变更即时生效）。

use std::sync::Arc;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use haruhi_core::{AppError, AppResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// 从 AppState 中取出的 JWT 密钥（通过 FromRef 提供给提取器）。
#[derive(Clone)]
pub struct AuthSecret(pub Arc<String>);

/// JWT 载荷。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user_id
    #[serde(rename = "super")]
    pub is_super: bool,
    pub iat: i64,
    pub exp: i64,
}

/// 已认证用户（仅身份，权限另查 DB）。
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: i64,
    pub is_super: bool,
}

// ---------- 令牌 ----------

pub fn issue_token(
    secret: &str,
    user_id: i64,
    is_super: bool,
    ttl_seconds: i64,
) -> AppResult<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        is_super,
        iat: now,
        exp: now + ttl_seconds,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::internal(format!("签发令牌失败: {e}")))
}

pub fn decode_token(secret: &str, token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized)
}

// ---------- 密码 ----------

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::internal(format!("密码哈希失败: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

// ---------- RBAC ----------

/// 操作动作 → 所需最小角色等级。
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Read,
    Write,
    Moderate,
    Manage,
}

impl Action {
    pub fn level(self) -> i64 {
        match self {
            Action::Read => 1,
            Action::Write => 2,
            Action::Moderate => 3,
            Action::Manage => 4,
        }
    }
}

/// 查询用户在某 app 的角色等级（无角色返回 None）。
pub async fn role_level(core: &SqlitePool, user_id: i64, app: &str) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT r.level FROM user_app_roles ua \
         JOIN roles r ON r.id = ua.role_id \
         WHERE ua.user_id = ? AND ua.app = ?",
    )
    .bind(user_id)
    .bind(app)
    .fetch_optional(core)
    .await?;
    Ok(row.map(|r| r.0))
}

/// 授权检查：超管全通过；否则要求账号 active 且在该 app 的角色等级达标。
pub async fn authorize(
    core: &SqlitePool,
    user: &AuthUser,
    app: &str,
    action: Action,
) -> AppResult<()> {
    if user.is_super {
        return Ok(());
    }
    // 账号必须存在且 active
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_optional(core)
        .await?;
    if status.as_deref() != Some("active") {
        return Err(AppError::Forbidden);
    }
    // 作用域链：拥有父作用域(如 "news")角色的用户，对其子作用域(如 "news.activity")同样有效。
    let mut best = 0;
    for scope in scope_chain(app) {
        if let Some(l) = role_level(core, user.id, &scope).await? {
            if l > best {
                best = l;
            }
        }
    }
    if best >= action.level() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// 作用域链：从最具体到最顶层。"news.activity" -> ["news.activity", "news"]；"news" -> ["news"]。
pub fn scope_chain(app: &str) -> Vec<String> {
    let mut chain = vec![app.to_string()];
    let mut cur = app;
    while let Some(idx) = cur.rfind('.') {
        cur = &cur[..idx];
        chain.push(cur.to_string());
    }
    chain
}

/// 仅超级管理员可通过。
pub fn require_super(user: &AuthUser) -> AppResult<()> {
    if user.is_super {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// ---------- axum 提取器 ----------

impl<S> FromRequestParts<S> for AuthUser
where
    AuthSecret: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let secret = AuthSecret::from_ref(state);
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let claims = decode_token(&secret.0, token)?;
        Ok(AuthUser {
            id: claims.sub,
            is_super: claims.is_super,
        })
    }
}

/// 可选鉴权：`Option<AuthUser>`。无 Bearer 头或 token 非法时为 None，
/// 仅在“可选登录”的端点（如游客投稿、公开详情）使用。axum 0.8 要求单独实现此 trait。
impl<S> OptionalFromRequestParts<S> for AuthUser
where
    AuthSecret: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let secret = AuthSecret::from_ref(state);
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        match token {
            Some(token) => match decode_token(&secret.0, token) {
                Ok(claims) => Ok(Some(AuthUser {
                    id: claims.sub,
                    is_super: claims.is_super,
                })),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }
}
