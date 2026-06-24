//! 服务端会话 + 一次性令牌 + Cookie/CSRF 助手。
//!
//! 设计：cookie 里是不透明随机串，库里只存它的 sha256（DB 泄露也拿不到活跃会话）。
//! 登出/改密 = 删 sessions 行 → 立即失效，这是无状态 JWT 做不到的。
//! CSRF：会话 cookie 走 httpOnly + SameSite=Lax（首道防线），外加可读的 csrf cookie
//! 做双提交（在 server 的 csrf 中间件里校验 header == cookie）。

use axum::http::{header, HeaderMap};
use base64::Engine;
use haruhi_core::{AppError, AppResult};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::AuthUser;

/// 会话 cookie 名（httpOnly，JS 读不到）。
pub const SESSION_COOKIE: &str = "haruhi_session";
/// CSRF cookie 名（非 httpOnly，前端读出来回填到请求头）。
pub const CSRF_COOKIE: &str = "haruhi_csrf";
/// CSRF 请求头名（双提交：须等于 CSRF cookie）。
pub const CSRF_HEADER: &str = "x-csrf-token";

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// 32 字节密码学随机 → base64url。用于会话原值、csrf、邮件令牌。
fn random_token() -> String {
    let mut buf = [0u8; 32];
    OsRng.fill_bytes(&mut buf);
    B64.encode(buf)
}

/// 取原值的 sha256（base64url），用作库内主键/索引，原值只在 cookie/邮件里出现。
fn hash_token(raw: &str) -> String {
    B64.encode(Sha256::digest(raw.as_bytes()))
}

/// 由 cookie 原值算出其库内会话主键（= sha256）。用于"列设备/排除当前会话"等。
pub fn session_id_of(raw_cookie: &str) -> String {
    hash_token(raw_cookie)
}

/// 从请求头解析某个 cookie 值（手写解析，避免引入 tower-cookies 依赖）。
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())?
        .split(';')
        .filter_map(|kv| {
            let (k, v) = kv.trim().split_once('=')?;
            (k == name).then(|| v.to_string())
        })
        .next()
}

/// 新建会话：返回 (cookie 原值, csrf_token)。库内主键存 sha256(cookie 原值)。
pub async fn create_session(
    core: &SqlitePool,
    user_id: i64,
    ttl_seconds: i64,
    user_agent: Option<&str>,
    ip: Option<&str>,
) -> AppResult<(String, String)> {
    let raw = random_token();
    let id = hash_token(&raw);
    let csrf = random_token();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, csrf_token, expires_at, last_seen_at, user_agent, ip) \
         VALUES (?, ?, ?, datetime('now', ?), datetime('now'), ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&csrf)
    .bind(format!("+{ttl_seconds} seconds"))
    .bind(user_agent)
    .bind(ip)
    .execute(core)
    .await?;
    Ok((raw, csrf))
}

/// 会话查询结果（提取器/handler 用）。
pub struct SessionInfo {
    pub user: AuthUser,
    /// 库内主键 = sha256(cookie 原值)，登出/管理会话时用。
    pub id: String,
    pub csrf_token: String,
}

/// 按 cookie 原值查会话：校验未过期、用户 active 未软删；顺手刷新 last_seen_at。
pub async fn lookup_session(core: &SqlitePool, raw_cookie: &str) -> AppResult<Option<SessionInfo>> {
    let id = hash_token(raw_cookie);
    let row: Option<(i64, String, bool, String, Option<String>)> = sqlx::query_as(
        "SELECT s.user_id, s.csrf_token, u.is_super_admin, u.status, u.deleted_at \
         FROM sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.id = ? AND s.expires_at > datetime('now')",
    )
    .bind(&id)
    .fetch_optional(core)
    .await?;

    match row {
        Some((uid, csrf, is_super, status, deleted_at))
            if status == "active" && deleted_at.is_none() =>
        {
            // 刷新最近活跃时间（失败不阻塞鉴权）
            let _ = sqlx::query("UPDATE sessions SET last_seen_at = datetime('now') WHERE id = ?")
                .bind(&id)
                .execute(core)
                .await;
            Ok(Some(SessionInfo {
                user: AuthUser { id: uid, is_super },
                id,
                csrf_token: csrf,
            }))
        }
        _ => Ok(None),
    }
}

/// 删除单个会话（按库内主键 = sha256）。
pub async fn revoke_session(core: &SqlitePool, session_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(core)
        .await?;
    Ok(())
}

/// 按 cookie 原值删除会话（登出用）。
pub async fn revoke_session_by_cookie(core: &SqlitePool, raw_cookie: &str) -> AppResult<()> {
    revoke_session(core, &hash_token(raw_cookie)).await
}

/// 删除某用户的所有会话（改密/重置后踢下线）；except 保留当前会话不被踢。
pub async fn revoke_user_sessions(
    core: &SqlitePool,
    user_id: i64,
    except: Option<&str>,
) -> AppResult<()> {
    match except {
        Some(keep) => {
            sqlx::query("DELETE FROM sessions WHERE user_id = ? AND id <> ?")
                .bind(user_id)
                .bind(keep)
                .execute(core)
                .await?;
        }
        None => {
            sqlx::query("DELETE FROM sessions WHERE user_id = ?")
                .bind(user_id)
                .execute(core)
                .await?;
        }
    }
    Ok(())
}

// ---------- 一次性令牌（邮箱验证 / 找回密码）----------

/// 签发一次性令牌：返回邮件里要发的原值；库存 sha256。
pub async fn issue_user_token(
    core: &SqlitePool,
    user_id: i64,
    kind: &str,
    ttl_seconds: i64,
) -> AppResult<String> {
    let raw = random_token();
    let hash = hash_token(&raw);
    sqlx::query(
        "INSERT INTO user_tokens (user_id, kind, token_hash, expires_at) \
         VALUES (?, ?, ?, datetime('now', ?))",
    )
    .bind(user_id)
    .bind(kind)
    .bind(&hash)
    .bind(format!("+{ttl_seconds} seconds"))
    .execute(core)
    .await?;
    Ok(raw)
}

/// 消费一次性令牌：校验未用未过期、kind 匹配，标记已用，返回 user_id。
pub async fn consume_user_token(core: &SqlitePool, raw: &str, kind: &str) -> AppResult<i64> {
    let hash = hash_token(raw);
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT id, user_id FROM user_tokens \
         WHERE token_hash = ? AND kind = ? AND used_at IS NULL AND expires_at > datetime('now')",
    )
    .bind(&hash)
    .bind(kind)
    .fetch_optional(core)
    .await?;
    let (tid, uid) = row.ok_or_else(|| AppError::bad_request("链接无效或已过期"))?;
    sqlx::query("UPDATE user_tokens SET used_at = datetime('now') WHERE id = ?")
        .bind(tid)
        .execute(core)
        .await?;
    Ok(uid)
}

// ---------- Cookie 头构造 ----------

fn secure_attr(secure: bool) -> &'static str {
    if secure {
        "; Secure"
    } else {
        ""
    }
}

/// 会话 Set-Cookie（httpOnly + SameSite=Lax + Path=/）。
pub fn session_set_cookie(raw: &str, ttl_seconds: i64, secure: bool) -> String {
    format!(
        "{SESSION_COOKIE}={raw}; HttpOnly{}; SameSite=Lax; Path=/; Max-Age={ttl_seconds}",
        secure_attr(secure)
    )
}

/// CSRF Set-Cookie（非 httpOnly，前端读出来回填到 X-CSRF-Token）。
pub fn csrf_set_cookie(csrf: &str, ttl_seconds: i64, secure: bool) -> String {
    format!(
        "{CSRF_COOKIE}={csrf}; SameSite=Lax; Path=/; Max-Age={ttl_seconds}{}",
        secure_attr(secure)
    )
}

/// 清除某 cookie（登出用，Max-Age=0）。
pub fn clear_cookie(name: &str, secure: bool) -> String {
    format!(
        "{name}=; HttpOnly{}; SameSite=Lax; Path=/; Max-Age=0",
        secure_attr(secure)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn mem() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        for ddl in [
            "CREATE TABLE users (id INTEGER PRIMARY KEY, is_super_admin INTEGER DEFAULT 0, status TEXT DEFAULT 'active', deleted_at TEXT)",
            "CREATE TABLE sessions (id TEXT PRIMARY KEY, user_id INTEGER, csrf_token TEXT, created_at TEXT DEFAULT (datetime('now')), expires_at TEXT, last_seen_at TEXT, user_agent TEXT, ip TEXT)",
            "CREATE TABLE user_tokens (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER, kind TEXT, token_hash TEXT, expires_at TEXT, used_at TEXT, created_at TEXT DEFAULT (datetime('now')))",
            "INSERT INTO users (id, is_super_admin, status) VALUES (7, 0, 'active')",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        pool
    }

    #[tokio::test]
    async fn session_create_lookup_revoke() {
        let pool = mem().await;
        let (raw, csrf) = create_session(&pool, 7, 3600, Some("UA"), Some("1.2.3.4"))
            .await
            .unwrap();

        let info = lookup_session(&pool, &raw).await.unwrap().expect("应命中");
        assert_eq!(info.user.id, 7);
        assert_eq!(info.csrf_token, csrf);

        // 错误 cookie 不命中
        assert!(lookup_session(&pool, "garbage").await.unwrap().is_none());

        // 登出后失效
        revoke_session_by_cookie(&pool, &raw).await.unwrap();
        assert!(lookup_session(&pool, &raw).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn session_rejects_expired_and_inactive() {
        let pool = mem().await;
        // 过期会话
        let (raw, _) = create_session(&pool, 7, -10, None, None).await.unwrap();
        assert!(lookup_session(&pool, &raw).await.unwrap().is_none());

        // disabled 用户的会话不命中
        let (raw2, _) = create_session(&pool, 7, 3600, None, None).await.unwrap();
        sqlx::query("UPDATE users SET status='disabled' WHERE id=7")
            .execute(&pool)
            .await
            .unwrap();
        assert!(lookup_session(&pool, &raw2).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn one_time_token_consumes_once() {
        let pool = mem().await;
        let raw = issue_user_token(&pool, 7, "verify_email", 3600)
            .await
            .unwrap();
        assert_eq!(
            consume_user_token(&pool, &raw, "verify_email")
                .await
                .unwrap(),
            7
        );
        // 第二次失败（已用）
        assert!(consume_user_token(&pool, &raw, "verify_email")
            .await
            .is_err());
        // kind 不匹配失败
        let raw2 = issue_user_token(&pool, 7, "reset_password", 3600)
            .await
            .unwrap();
        assert!(consume_user_token(&pool, &raw2, "verify_email")
            .await
            .is_err());
    }

    #[test]
    fn cookie_parsing() {
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            "a=1; haruhi_session=abc.def; haruhi_csrf=xyz"
                .parse()
                .unwrap(),
        );
        assert_eq!(cookie_value(&h, SESSION_COOKIE).as_deref(), Some("abc.def"));
        assert_eq!(cookie_value(&h, CSRF_COOKIE).as_deref(), Some("xyz"));
        assert_eq!(cookie_value(&h, "missing"), None);
    }
}
