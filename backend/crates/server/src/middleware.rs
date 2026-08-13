//! 跨域请求伪造（CSRF）防护中间件。
//!
//! 仅当请求**携带会话 cookie**（即浏览器 cookie 鉴权）且为写方法时才校验：
//! 要求 `X-CSRF-Token` 请求头 == `haruhi_csrf` cookie（双提交）。
//! - 安全方法（GET/HEAD/OPTIONS）放行；
//! - 无会话 cookie（未登录 / 旧 Bearer 鉴权 / 登录注册等首次请求）放行——它们不在浏览器
//!   自动带 cookie 的 CSRF 攻击面内；
//! - 会话 cookie 本身是 httpOnly + SameSite=Lax（首道防线），此中间件是纵深防御。

use axum::extract::Request;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use haruhi_auth::session::{cookie_value, CSRF_COOKIE, CSRF_HEADER, SESSION_COOKIE};
use haruhi_core::AppError;

use crate::game_auth_routes::{
    GAME_CSRF_COOKIE, GAME_CSRF_COOKIE_DEV, GAME_SESSION_COOKIE, GAME_SESSION_COOKIE_DEV,
};

pub async fn csrf_guard(req: Request, next: Next) -> Result<Response, AppError> {
    // 安全方法不改状态，放行
    if matches!(
        *req.method(),
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    ) {
        return Ok(next.run(req).await);
    }

    // 授权码交换没有既有游戏会话；其安全性来自一次性授权码、state 与 PKCE。
    // 即使浏览器残留旧游戏 cookie，也不能让它阻断重新登录。
    if req.uri().path() == "/api/game/session/exchange" {
        return Ok(next.run(req).await);
    }

    let is_game_api = req.uri().path().starts_with("/api/game/");
    let headers = req.headers();

    let (session_names, csrf_names): (&[&str], &[&str]) = if is_game_api {
        (
            &[GAME_SESSION_COOKIE, GAME_SESSION_COOKIE_DEV],
            &[GAME_CSRF_COOKIE, GAME_CSRF_COOKIE_DEV],
        )
    } else {
        (&[SESSION_COOKIE], &[CSRF_COOKIE])
    };

    // 没有会话 cookie → 不是 cookie 鉴权请求，放行（登录/注册/旧 Bearer 等）
    if !session_names
        .iter()
        .any(|name| cookie_value(headers, name).is_some())
    {
        return Ok(next.run(req).await);
    }

    // 双提交：请求头必须与 csrf cookie 一致且非空
    let cookie_csrf = csrf_names
        .iter()
        .find_map(|name| cookie_value(headers, name));
    let header_csrf = headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match (cookie_csrf, header_csrf) {
        (Some(c), Some(h)) if !c.is_empty() && c == h => Ok(next.run(req).await),
        _ => Err(AppError::Forbidden),
    }
}
