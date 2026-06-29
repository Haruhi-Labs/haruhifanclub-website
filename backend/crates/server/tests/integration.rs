//! 路由级集成测试：用 tower oneshot 打真实 router，覆盖健康检查、登录拿 JWT、
//! /auth/me、以及 /api/admin/* 的 RBAC（无 token 401 / 普通用户 403 / 超管 200）+ 一个公开模块 GET。
//!
//! 这些测试在 `tests/` 下、是独立编译单元，仅在完整 `cargo test` 时运行
//! （PR 的 `cargo test --lib` 不含它们，合入 main 后的完整测试才跑）。

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use haruhi_auth::hash_password;
use haruhi_core::{Config, MailConfig};
use haruhi_db::Pools;
use haruhi_server::ratelimit::RateLimiter;
use haruhi_server::state::AppState;
use haruhi_server::{routes, seed};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt; // oneshot

const ADMIN_USER: &str = "admin";
const ADMIN_PASS: &str = "testpass123";

struct TestApp {
    router: Router,
    state: AppState,
    _dir: tempfile::TempDir, // 持有以防临时目录在测试期间被删
}

fn test_config(data_dir: PathBuf, uploads_dir: PathBuf) -> Config {
    Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir,
        uploads_dir,
        jwt_secret: "test-secret-please-change-32-chars-long".into(),
        jwt_ttl_seconds: 3600,
        session_ttl_seconds: 3600,
        cookie_secure: false,
        superadmin_user: Some(ADMIN_USER.into()),
        superadmin_password: Some(ADMIN_PASS.into()),
        dashscope_api_key: None,
        ai_api_url: "http://localhost/v1".into(),
        ai_text_model: "qwen-plus".into(),
        ai_image_model: "qwen-vl-plus".into(),
        art_cookie_secret: "test-art-cookie-secret".into(),
        shop_free_shipping_threshold: 150,
        mail: MailConfig {
            enabled: false,
            provider: "auto".into(),
            from_name: "测试".into(),
            from_address: None,
            reply_to: None,
            resend_api_key: None,
            resend_api_base_url: "https://api.resend.com".into(),
            smtp_host: None,
            smtp_port: 465,
            smtp_secure: true,
            smtp_user: None,
            smtp_pass: None,
        },
        public_site_url: "http://localhost".into(),
        account_web_base: "http://localhost/news".into(),
        cors_origins: vec![],
    }
}

async fn setup() -> TestApp {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let uploads_dir = dir.path().join("uploads");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&uploads_dir).unwrap();

    let cfg = Arc::new(test_config(data_dir, uploads_dir));
    let pools = Pools::connect(&cfg).await.unwrap();
    pools.migrate().await.unwrap();
    seed::seed_superadmin(&cfg, &pools.core).await.unwrap();

    let state = AppState {
        cfg,
        pools,
        login_limiter: Arc::new(RateLimiter::new(10, 600)),
        upload_limiter: Arc::new(RateLimiter::new(60, 600)),
        account_limiter: Arc::new(RateLimiter::new(5, 3600)),
        mailer: None,
    };
    let router = routes::router(state.clone());
    TestApp {
        router,
        state,
        _dir: dir,
    }
}

// ---- 请求/响应小工具 ----

fn get(path: &str, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    b.body(Body::empty()).unwrap()
}

fn post_json(path: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    b.body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

async fn send(router: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

async fn send_full(router: &Router, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, headers, json)
}

fn post_json_with_cookie(path: &str, body: Value, cookie: &str) -> Request<Body> {
    let mut req = post_json(path, body, None);
    req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
    req
}

fn cookie_header_from_set_cookie(headers: &HeaderMap) -> String {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .collect::<Vec<_>>()
        .join("; ")
}

async fn login(router: &Router, user: &str, pass: &str) -> String {
    let (s, j) = send(
        router,
        post_json(
            "/api/auth/login",
            json!({ "username": user, "password": pass }),
            None,
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "登录应成功，实际: {s} {j:?}");
    j["token"].as_str().expect("应返回 token").to_string()
}

async fn insert_active_user(state: &AppState, user: &str, pass: &str) {
    let hash = hash_password(pass).unwrap();
    sqlx::query(
        "INSERT INTO users (username, password_hash, display_name, is_super_admin, status) \
         VALUES (?, ?, ?, 0, 'active')",
    )
    .bind(user)
    .bind(hash)
    .bind(user)
    .execute(&state.pools.core)
    .await
    .unwrap();
}

// ---- 测试 ----

#[tokio::test]
async fn health_and_ready_ok() {
    let app = setup().await;

    let (s, j) = send(&app.router, get("/api/health", None)).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(j["status"], "ok");

    // readiness 探针应连通 core 库 → 200
    let (s, _) = send(&app.router, get("/api/health/ready", None)).await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn login_success_then_me_reflects_superadmin() {
    let app = setup().await;

    let (s, j) = send(
        &app.router,
        post_json(
            "/api/auth/login",
            json!({ "username": ADMIN_USER, "password": ADMIN_PASS }),
            None,
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "种子超管应能登录: {j:?}");
    let token = j["token"].as_str().expect("应返回 token").to_string();
    assert_eq!(j["user"]["isSuperAdmin"], true);

    let (s, me) = send(&app.router, get("/api/auth/me", Some(&token))).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(me["username"], ADMIN_USER);
    assert_eq!(me["isSuperAdmin"], true);
}

#[tokio::test]
async fn login_wrong_password_is_unauthorized() {
    let app = setup().await;
    let (s, _) = send(
        &app.router,
        post_json(
            "/api/auth/login",
            json!({ "username": ADMIN_USER, "password": "wrong-password" }),
            None,
        ),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_endpoint_enforces_rbac() {
    let app = setup().await;

    // 1) 无 token → 401
    let (s, _) = send(&app.router, get("/api/admin/users", None)).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "无 token 应 401");

    // 2) 超管 token → 200
    let admin = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let (s, _) = send(&app.router, get("/api/admin/users", Some(&admin))).await;
    assert_eq!(s, StatusCode::OK, "超管应可访问 /api/admin/users");

    // 3) 普通 active 用户（无任何角色）→ 403
    insert_active_user(&app.state, "bob", "bob-pass").await;
    let bob = login(&app.router, "bob", "bob-pass").await;
    let (s, _) = send(&app.router, get("/api/admin/users", Some(&bob))).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "普通用户访问超管接口应 403");
}

#[tokio::test]
async fn public_news_articles_list_is_accessible() {
    let app = setup().await;
    // 公开列表：空库也应 200，且返回 JSON（数组或带 data 的对象）
    let (s, j) = send(&app.router, get("/api/news/articles", None)).await;
    assert_eq!(s, StatusCode::OK, "公开文章列表应可匿名访问: {j:?}");
    assert!(
        j.is_array() || j.is_object(),
        "应返回 JSON 结构，实际: {j:?}"
    );
}

// ============================================================
// 特征化回归网：钉住各模块「列表/分页 + RBAC + 上传校验」当前行为，
// 作为后续抽取公共 CRUD/分页层的安全网（重构后这些必须保持全绿）。
// ============================================================

// ---- 额外测试工具 ----

/// 给用户在某 app 授予角色（JWT 不含角色，authorize 实时查库，同一 token 立即生效）。
async fn grant_role(state: &AppState, username: &str, app: &str, role_key: &str) {
    let uid: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(&state.pools.core)
        .await
        .unwrap();
    let rid: i64 = sqlx::query_scalar("SELECT id FROM roles WHERE key = ?")
        .bind(role_key)
        .fetch_one(&state.pools.core)
        .await
        .unwrap();
    sqlx::query("INSERT OR REPLACE INTO user_app_roles (user_id, app, role_id) VALUES (?, ?, ?)")
        .bind(uid)
        .bind(app)
        .bind(rid)
        .execute(&state.pools.core)
        .await
        .unwrap();
}

async fn seed_artwork(state: &AppState, title: &str, status: &str, created_at: &str) {
    sqlx::query(
        "INSERT INTO artworks (title, status, content_type, source_type, created_at, like_total) \
         VALUES (?, ?, 'haruhi', 'network', ?, 0)",
    )
    .bind(title)
    .bind(status)
    .bind(created_at)
    .execute(&state.pools.art)
    .await
    .unwrap();
}

async fn seed_exam(state: &AppState, id: &str, title: &str, status: &str) {
    sqlx::query(
        "INSERT INTO exams (id, title, status, config, questions) VALUES (?, ?, ?, '{}', '[]')",
    )
    .bind(id)
    .bind(title)
    .bind(status)
    .execute(&state.pools.exam)
    .await
    .unwrap();
}

async fn seed_book(state: &AppState, id: &str, title: &str, sort_order: f64) {
    sqlx::query("INSERT INTO books (id, title, author, sort_order) VALUES (?, ?, '佚名', ?)")
        .bind(id)
        .bind(title)
        .bind(sort_order)
        .execute(&state.pools.novel)
        .await
        .unwrap();
}

async fn seed_coupon(state: &AppState, code: &str) {
    sqlx::query("INSERT INTO coupons (code, name, status) VALUES (?, ?, 1)")
        .bind(code)
        .bind(code)
        .execute(&state.pools.shop)
        .await
        .unwrap();
}

/// 构造 multipart/form-data 请求。parts: (字段名, 文件名(None=普通字段), 内容)。
fn multipart_req(
    path: &str,
    parts: &[(&str, Option<&str>, &str)],
    token: Option<&str>,
) -> Request<Body> {
    let boundary = "----haruhitestboundary";
    let mut body: Vec<u8> = Vec::new();
    for (name, filename, content) in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match filename {
            Some(fname) => body.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"{name}\"; filename=\"{fname}\"\r\n\
                     Content-Type: application/octet-stream\r\n\r\n"
                )
                .as_bytes(),
            ),
            None => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
            ),
        }
        body.extend_from_slice(content.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    let mut b = Request::builder().method("POST").uri(path).header(
        "content-type",
        format!("multipart/form-data; boundary={boundary}"),
    );
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    b.body(Body::from(body)).unwrap()
}

// ---- 列表 / 分页 ----

#[tokio::test]
async fn art_list_default_filters_to_approved() {
    let app = setup().await;
    seed_artwork(&app.state, "a1", "approved", "2024-01-01 00:00:00").await;
    seed_artwork(&app.state, "a2", "approved", "2024-01-02 00:00:00").await;
    seed_artwork(&app.state, "a3", "approved", "2024-01-03 00:00:00").await;
    seed_artwork(&app.state, "p1", "pending", "2024-01-04 00:00:00").await;

    let (s, j) = send(&app.router, get("/api/art/artworks", None)).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(j["ok"], true);
    assert_eq!(j["total"], 3, "默认仅统计 approved: {j:?}");
    assert_eq!(j["data"].as_array().unwrap().len(), 3);

    let (_, all) = send(&app.router, get("/api/art/artworks?status=all", None)).await;
    assert_eq!(all["total"], 4, "status=all 含 pending");
}

#[tokio::test]
async fn art_list_pagination_math() {
    let app = setup().await;
    for i in 0..8 {
        seed_artwork(
            &app.state,
            &format!("art{i}"),
            "approved",
            &format!("2024-02-{:02} 00:00:00", i + 1),
        )
        .await;
    }
    // pageSize 下限为 6（clamp）
    let (_, p1) = send(
        &app.router,
        get("/api/art/artworks?pageSize=6&page=1", None),
    )
    .await;
    assert_eq!(p1["total"], 8);
    assert_eq!(p1["data"].as_array().unwrap().len(), 6, "首页 6 条");
    let (_, p2) = send(
        &app.router,
        get("/api/art/artworks?pageSize=6&page=2", None),
    )
    .await;
    assert_eq!(p2["total"], 8);
    assert_eq!(p2["data"].as_array().unwrap().len(), 2, "次页 2 条");
}

#[tokio::test]
async fn art_visitors_count_independent_visit_after_ten_minutes() {
    let app = setup().await;

    let (s, headers, first) =
        send_full(&app.router, post_json("/api/art/visitors", json!({}), None)).await;
    assert_eq!(s, StatusCode::OK, "首次访客统计应成功: {first:?}");
    assert_eq!(first["ok"], true);
    assert_eq!(first["total"], 1);
    assert_eq!(first["uniqueVisitors"], 1);
    assert_eq!(first["isNew"], true);
    assert_eq!(first["isNewVisitor"], true);

    let cookie = cookie_header_from_set_cookie(&headers);
    assert!(
        cookie.contains("haruhi_anon=") && cookie.contains("haruhi_anon_sig="),
        "首次响应应下发匿名身份 Cookie，实际: {cookie}"
    );

    let (s, second) = send(
        &app.router,
        post_json_with_cookie("/api/art/visitors", json!({}), &cookie),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        second["total"], 1,
        "10 分钟内重复访问不增加总数: {second:?}"
    );
    assert_eq!(second["uniqueVisitors"], 1);
    assert_eq!(second["isNew"], false);
    assert_eq!(second["isNewVisitor"], false);

    let old_seen = (chrono::Utc::now() - chrono::Duration::minutes(11))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    sqlx::query("UPDATE art_visitors SET last_seen_at=?")
        .bind(old_seen)
        .execute(&app.state.pools.art)
        .await
        .unwrap();

    let (s, third) = send(
        &app.router,
        post_json_with_cookie("/api/art/visitors", json!({}), &cookie),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        third["total"], 2,
        "同一匿名身份超过 10 分钟后应计作新的独立访问: {third:?}"
    );
    assert_eq!(third["uniqueVisitors"], 1);
    assert_eq!(third["isNew"], true);
    assert_eq!(third["isNewVisitor"], false);

    let stored_count: i64 = sqlx::query_scalar("SELECT visit_count FROM art_visitors")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    assert_eq!(stored_count, 2);
}

#[tokio::test]
async fn exam_list_reserves_first_page_slot() {
    let app = setup().await;
    for i in 0..10 {
        seed_exam(
            &app.state,
            &format!("e{i}"),
            &format!("exam{i}"),
            "published",
        )
        .await;
    }
    let (s, j) = send(&app.router, get("/api/exam/exams", None)).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(j["pagination"]["total"], 10);
    // 非搜索模式首页为官方试卷预留 1 位 → 展示 limit-1 = 8 条
    assert_eq!(
        j["data"].as_array().unwrap().len(),
        8,
        "首页应保留 1 位、展示 8 条: {j:?}"
    );
}

#[tokio::test]
async fn novel_books_listed_in_sort_order() {
    let app = setup().await;
    seed_book(&app.state, "b-mid", "中", 2.0).await;
    seed_book(&app.state, "b-first", "前", 1.0).await;
    seed_book(&app.state, "b-last", "后", 3.0).await;
    let (s, j) = send(&app.router, get("/api/novel/books", None)).await;
    assert_eq!(s, StatusCode::OK);
    let arr = j.as_array().expect("应返回数组");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["title"], "前", "应按 sort_order 升序");
    assert_eq!(arr[2]["title"], "后");
}

#[tokio::test]
async fn shop_coupons_pagination_meta() {
    let app = setup().await;
    let supert = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    for i in 0..25 {
        seed_coupon(&app.state, &format!("C{i:03}")).await;
    }
    let (s, j) = send(
        &app.router,
        get("/api/shop/admin/coupons?pageSize=10&page=2", Some(&supert)),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(j["pagination"]["total"], 25);
    assert_eq!(j["pagination"]["totalPages"], 3, "25/10 向上取整=3");
    assert_eq!(j["pagination"]["page"], 2);
    assert_eq!(j["pagination"]["pageSize"], 10);
    assert_eq!(j["items"].as_array().unwrap().len(), 10, "第 2 页 10 条");
}

// ---- RBAC 边界 ----

#[tokio::test]
async fn admin_lists_enforce_rbac_across_modules() {
    let app = setup().await;
    let supert = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    insert_active_user(&app.state, "carol", "carol-pass").await;
    let carol = login(&app.router, "carol", "carol-pass").await;

    for path in [
        "/api/art/admin/pending-artworks",
        "/api/exam/admin/list",
        "/api/shop/admin/coupons",
    ] {
        let (s, _) = send(&app.router, get(path, None)).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "{path} 匿名应 401");
        let (s, _) = send(&app.router, get(path, Some(&carol))).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{path} 无角色应 403");
        let (s, _) = send(&app.router, get(path, Some(&supert))).await;
        assert_eq!(s, StatusCode::OK, "{path} 超管应 200");
    }

    // 授予 art viewer(Read) 后，同一 token 即可访问 art 后台（authorize 实时查库）
    grant_role(&app.state, "carol", "art", "viewer").await;
    let (s, _) = send(
        &app.router,
        get("/api/art/admin/pending-artworks", Some(&carol)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "授予 art viewer 后应放行");
    // 但未授予的 exam 仍 403
    let (s, _) = send(&app.router, get("/api/exam/admin/list", Some(&carol))).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "未授予 exam 应仍 403");
}

// ---- 匿名上传校验（保留匿名、拒绝非法类型）----

#[tokio::test]
async fn art_upload_rejects_non_image() {
    let app = setup().await;
    // 画廊上传已改为必须登录；超管 token 绕过邮箱验证，得以走到图片类型校验
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let req = multipart_req(
        "/api/art/artworks",
        &[
            ("images", Some("evil.txt"), "not an image"),
            ("title", None, "标题"),
        ],
        Some(&token),
    );
    let (s, _) = send(&app.router, req).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "非图片扩展名应被拒");
}

#[tokio::test]
async fn art_upload_requires_login() {
    let app = setup().await;
    // 未登录上传 → 401（取代过去的匿名上传）
    let req = multipart_req(
        "/api/art/artworks",
        &[
            ("images", Some("ok.png"), "\u{89}PNG"),
            ("title", None, "x"),
        ],
        None,
    );
    let (s, _) = send(&app.router, req).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "未登录上传应 401");
}

#[tokio::test]
async fn art_upload_accepts_image_and_persists() {
    let app = setup().await;
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let req = multipart_req(
        "/api/art/artworks",
        &[
            ("images", Some("ok.png"), "\u{89}PNG-fake-bytes"),
            ("title", None, "我的作品"),
        ],
        Some(&token),
    );
    let (s, j) = send(&app.router, req).await;
    assert_eq!(s, StatusCode::OK, "合法图片登录上传应成功: {j:?}");
    assert_eq!(j["ok"], true);
    // 落库可验证（AI 离线 → 状态 pending，用 status=all 查得到）
    let (_, all) = send(&app.router, get("/api/art/artworks?status=all", None)).await;
    assert_eq!(all["total"], 1, "上传的作品应已落库");
}

#[tokio::test]
async fn exam_upload_rejects_non_media() {
    let app = setup().await;
    let req = multipart_req(
        "/api/exam/upload",
        &[("file", Some("payload.html"), "<script>alert(1)</script>")],
        None,
    );
    let (s, _) = send(&app.router, req).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "非图片/音频应被拒");
}
