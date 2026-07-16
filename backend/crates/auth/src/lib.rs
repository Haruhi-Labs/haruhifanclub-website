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

pub mod session;
pub use session::{
    clear_cookie, consume_user_token, cookie_value, create_session, csrf_set_cookie, hash_token,
    issue_user_token, lookup_session, revoke_session, revoke_session_by_cookie,
    revoke_user_sessions, session_id_of, session_set_cookie, SessionInfo, CSRF_COOKIE, CSRF_HEADER,
    SESSION_COOKIE,
};

/// 从 AppState 中取出的 JWT 密钥（通过 FromRef 提供给提取器）。
#[derive(Clone)]
pub struct AuthSecret(pub Arc<String>);

/// 从 AppState 中取出的 core.db 连接池（会话提取器据此查 sessions 表）。
#[derive(Clone)]
pub struct CoreDb(pub SqlitePool);

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

/// 实例级能力授权，供地方支部等多租户模块使用。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityGrant {
    pub capability: String,
    pub scope_type: String,
    pub scope_id: String,
    pub expires_at: Option<String>,
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

/// 能力授权检查：超级管理员全通；普通账号必须 active，并持有精确实例授权，
/// 或持有 chapter 平台作用域下的同名能力。
pub async fn authorize_capability(
    core: &SqlitePool,
    user: &AuthUser,
    capability: &str,
    scope_type: &str,
    scope_id: &str,
) -> AppResult<()> {
    if user.is_super {
        return Ok(());
    }
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_optional(core)
        .await?;
    if status.as_deref() != Some("active") {
        return Err(AppError::Forbidden);
    }
    let allowed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM capability_grants \
         WHERE user_id = ? AND capability = ? \
           AND (expires_at IS NULL OR expires_at > datetime('now')) \
           AND ((scope_type = ? AND scope_id = ?) \
                OR (scope_type = 'platform' AND scope_id = 'chapter'))",
    )
    .bind(user.id)
    .bind(capability)
    .bind(scope_type)
    .bind(scope_id)
    .fetch_one(core)
    .await?;
    if allowed > 0 {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub async fn capability_grants(core: &SqlitePool, user_id: i64) -> AppResult<Vec<CapabilityGrant>> {
    Ok(sqlx::query_as(
        "SELECT capability, scope_type, scope_id, expires_at FROM capability_grants \
         WHERE user_id = ? AND (expires_at IS NULL OR expires_at > datetime('now')) \
         ORDER BY scope_type, scope_id, capability",
    )
    .bind(user_id)
    .fetch_all(core)
    .await?)
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

/// 从请求里解析当前用户：**优先**会话 cookie（可吊销、抗 XSS），
/// **回退**到旧的 `Authorization: Bearer <JWT>`（兼容迁移期，待全站切 cookie 后删除）。
async fn resolve_user<S>(parts: &Parts, state: &S) -> Option<AuthUser>
where
    AuthSecret: FromRef<S>,
    CoreDb: FromRef<S>,
    S: Send + Sync,
{
    // 1) 会话 cookie
    if let Some(raw) = session::cookie_value(&parts.headers, SESSION_COOKIE) {
        let core = CoreDb::from_ref(state).0;
        if let Ok(Some(info)) = lookup_session(&core, &raw).await {
            return Some(info.user);
        }
    }
    // 2) 兼容回退：Bearer JWT
    let secret = AuthSecret::from_ref(state);
    let token = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))?;
    decode_token(&secret.0, token).ok().map(|claims| AuthUser {
        id: claims.sub,
        is_super: claims.is_super,
    })
}

impl<S> FromRequestParts<S> for AuthUser
where
    AuthSecret: FromRef<S>,
    CoreDb: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        resolve_user(parts, state)
            .await
            .ok_or(AppError::Unauthorized)
    }
}

/// 可选鉴权：`Option<AuthUser>`。无有效会话/令牌时为 None，
/// 仅在“可选登录”的端点（如公开详情、个性化展示）使用。axum 0.8 要求单独实现此 trait。
impl<S> OptionalFromRequestParts<S> for AuthUser
where
    AuthSecret: FromRef<S>,
    CoreDb: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(resolve_user(parts, state).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn token_roundtrip_and_rejects() {
        let secret = "test-secret";
        let t = issue_token(secret, 42, true, 3600).unwrap();
        let c = decode_token(secret, &t).unwrap();
        assert_eq!(c.sub, 42);
        assert!(c.is_super);
        assert!(decode_token("wrong-secret", &t).is_err()); // 错误密钥拒绝
                                                            // 过期 120 秒（超过 jsonwebtoken 默认 60s leeway）应被拒
        let expired = issue_token(secret, 1, false, -120).unwrap();
        assert!(decode_token(secret, &expired).is_err());
    }

    #[test]
    fn password_hash_and_verify() {
        let h = hash_password("hunter2").unwrap();
        assert!(verify_password("hunter2", &h));
        assert!(!verify_password("wrong", &h));
        assert!(!verify_password("hunter2", "not-a-valid-hash"));
    }

    #[test]
    fn scope_chain_walks_parents() {
        assert_eq!(scope_chain("news.activity"), vec!["news.activity", "news"]);
        assert_eq!(scope_chain("news"), vec!["news"]);
        assert_eq!(scope_chain("a.b.c"), vec!["a.b.c", "a.b", "a"]);
    }

    #[test]
    fn action_levels_ordered() {
        assert!(Action::Read.level() < Action::Write.level());
        assert!(Action::Write.level() < Action::Moderate.level());
        assert!(Action::Moderate.level() < Action::Manage.level());
    }

    async fn mem_pool() -> SqlitePool {
        // max_connections=1：保证 :memory: 所有查询共享同一连接(同一内存库)
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        for ddl in [
            "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, password_hash TEXT, is_super_admin INTEGER DEFAULT 0, status TEXT DEFAULT 'active')",
            "CREATE TABLE roles (id INTEGER PRIMARY KEY, key TEXT, name TEXT, level INTEGER)",
            "CREATE TABLE user_app_roles (user_id INTEGER, app TEXT, role_id INTEGER, PRIMARY KEY(user_id,app))",
            "CREATE TABLE capability_grants (id INTEGER PRIMARY KEY, user_id INTEGER, capability TEXT, scope_type TEXT, scope_id TEXT, expires_at TEXT)",
            "INSERT INTO roles (id,key,name,level) VALUES (3,'moderator','审核',3),(4,'admin','管理',4)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        pool
    }

    async fn add_user(pool: &SqlitePool, id: i64, status: &str, app: &str, role_id: i64) {
        sqlx::query("INSERT INTO users (id,username,password_hash,status) VALUES (?,?,?,?)")
            .bind(id)
            .bind(format!("u{id}"))
            .bind("x")
            .bind(status)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO user_app_roles (user_id,app,role_id) VALUES (?,?,?)")
            .bind(id)
            .bind(app)
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rbac_subscope_isolation_inheritance_and_gating() {
        let pool = mem_pool().await;

        // 仅 news.activity:admin
        add_user(&pool, 1, "active", "news.activity", 4).await;
        let act = AuthUser {
            id: 1,
            is_super: false,
        };
        assert!(authorize(&pool, &act, "news.activity", Action::Manage)
            .await
            .is_ok());
        assert!(authorize(&pool, &act, "news.blog", Action::Read)
            .await
            .is_err()); // 子作用域隔离
        assert!(authorize(&pool, &act, "news", Action::Read).await.is_err()); // 子不授父

        // 父级 news:admin → 继承所有 news.*
        add_user(&pool, 2, "active", "news", 4).await;
        let mgr = AuthUser {
            id: 2,
            is_super: false,
        };
        assert!(authorize(&pool, &mgr, "news.activity", Action::Manage)
            .await
            .is_ok());
        assert!(authorize(&pool, &mgr, "news.points", Action::Manage)
            .await
            .is_ok());

        // 超管全通过
        let sup = AuthUser {
            id: 999,
            is_super: true,
        };
        assert!(authorize(&pool, &sup, "anything", Action::Manage)
            .await
            .is_ok());

        // disabled 账号被拒
        add_user(&pool, 3, "disabled", "news", 4).await;
        let dis = AuthUser {
            id: 3,
            is_super: false,
        };
        assert!(authorize(&pool, &dis, "news", Action::Read).await.is_err());

        // 等级门控：moderator(3) 可 Moderate 不可 Manage
        add_user(&pool, 4, "active", "art", 3).await;
        let m = AuthUser {
            id: 4,
            is_super: false,
        };
        assert!(authorize(&pool, &m, "art", Action::Moderate).await.is_ok());
        assert!(authorize(&pool, &m, "art", Action::Manage).await.is_err());
    }

    #[tokio::test]
    async fn capability_grants_isolate_branch_and_allow_platform_inheritance() {
        let pool = mem_pool().await;
        add_user(&pool, 8, "active", "news", 1).await;
        sqlx::query("INSERT INTO capability_grants (user_id,capability,scope_type,scope_id) VALUES (8,'branch.posts.write','branch','12'),(8,'branch.audit.read','platform','chapter')")
            .execute(&pool).await.unwrap();
        let user = AuthUser {
            id: 8,
            is_super: false,
        };
        assert!(
            authorize_capability(&pool, &user, "branch.posts.write", "branch", "12")
                .await
                .is_ok()
        );
        assert!(
            authorize_capability(&pool, &user, "branch.posts.write", "branch", "13")
                .await
                .is_err()
        );
        assert!(
            authorize_capability(&pool, &user, "branch.audit.read", "branch", "13")
                .await
                .is_ok()
        );
    }
}
