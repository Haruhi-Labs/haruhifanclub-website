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
    let random_key = title
        .bytes()
        .fold(0_i64, |acc, b| (acc * 31 + i64::from(b)) % 2_147_483_646)
        + 1;
    sqlx::query(
        "INSERT INTO artworks (title, status, content_type, source_type, created_at, like_total, random_key) \
         VALUES (?, ?, 'haruhi', 'network', ?, 0, ?)",
    )
    .bind(title)
    .bind(status)
    .bind(created_at)
    .bind(random_key)
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
async fn art_random_list_uses_stable_pagination() {
    let app = setup().await;
    for i in 0..12 {
        seed_artwork(
            &app.state,
            &format!("random-art-{i}"),
            "approved",
            &format!("2024-03-{:02} 00:00:00", i + 1),
        )
        .await;
    }

    let (_, p1) = send(
        &app.router,
        get(
            "/api/art/artworks?sort=random&seed=42&pageSize=6&page=1",
            None,
        ),
    )
    .await;
    let (_, p1_again) = send(
        &app.router,
        get(
            "/api/art/artworks?sort=random&seed=42&pageSize=6&page=1",
            None,
        ),
    )
    .await;
    let (_, p2) = send(
        &app.router,
        get(
            "/api/art/artworks?sort=random&seed=42&pageSize=6&page=2",
            None,
        ),
    )
    .await;

    assert_eq!(p1["seedUsed"], 42);
    assert_eq!(p1["total"], 12);
    assert_eq!(p1["data"], p1_again["data"], "同 seed 的随机页应稳定");
    let ids1: std::collections::HashSet<i64> = p1["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["id"].as_i64())
        .collect();
    let ids2: std::collections::HashSet<i64> = p2["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["id"].as_i64())
        .collect();
    assert_eq!(ids1.len(), 6);
    assert_eq!(ids2.len(), 6);
    assert!(ids1.is_disjoint(&ids2), "随机分页不应在相邻页重复");
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
        "/api/art/admin/guild/creator-production-stats",
        "/api/art/admin/guild/budget",
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
    assert_eq!(j["status"], "pending");
    assert_eq!(j["aiReviewPending"], false);
    // 落库可验证（AI 离线 → 状态 pending，用 status=all 查得到）
    let (_, all) = send(&app.router, get("/api/art/artworks?status=all", None)).await;
    assert_eq!(all["total"], 1, "上传的作品应已落库");
    let (status, random_key): (String, i64) =
        sqlx::query_as("SELECT status, random_key FROM artworks WHERE title='我的作品'")
            .fetch_one(&app.state.pools.art)
            .await
            .unwrap();
    assert_eq!(status, "pending");
    assert!(random_key > 0, "上传作品应写入随机排序键");
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

// 画廊积分语义：作品「公开存在」时应有积分，撤稿（隐藏/拒绝/删除）应扣回，
// 复审再公开应重新发放；兑换消耗不计入「历史累计获得积分」，撤稿扣回则计入。
#[tokio::test]
async fn artwork_points_follow_public_state() {
    use haruhi_server::modules::art_guild;

    let app = setup().await;
    let art = app.state.pools.art.clone();
    let uid = "u9001";
    let ts = "2026-01-01T00:00:00Z";

    sqlx::query(
        "INSERT INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at) \
         VALUES(?, 0, 'F', 'observer_clearance', ?, ?)",
    )
    .bind(uid)
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    // 个人凉宫作品，初始 pending（应得 120）
    let aid: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at) \
         VALUES('测试作品', ?, 'personal', 'haruhi', 'pending', ?) RETURNING id",
    )
    .bind(uid)
    .bind(ts)
    .fetch_one(&art)
    .await
    .unwrap();

    async fn net(pool: &sqlx::SqlitePool, aid: i64) -> i64 {
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE artwork_id=?")
            .bind(aid)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn balance(pool: &sqlx::SqlitePool, uid: &str) -> i64 {
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE uid=?")
            .bind(uid)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn earned(pool: &sqlx::SqlitePool, uid: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT COALESCE(SUM(CASE WHEN source_type='redemption' THEN 0 ELSE points END), 0) \
             FROM points_ledger WHERE uid=?",
        )
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap()
    }
    async fn reputation_for_artwork(pool: &sqlx::SqlitePool, aid: i64) -> i64 {
        sqlx::query_scalar(
            "SELECT COALESCE(SUM(reputation), 0) FROM reputation_ledger WHERE artwork_id=?",
        )
        .bind(aid)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    // pending：无积分
    assert_eq!(net(&art, aid).await, 0);

    // 公开 → +120 积分 / +120 声望
    art_guild::on_artwork_published(&app.state, aid, "")
        .await
        .unwrap();
    assert_eq!(net(&art, aid).await, 120, "首次公开应发放 120");
    assert_eq!(
        reputation_for_artwork(&art, aid).await,
        120,
        "首次公开应发放投稿声望"
    );

    // 幂等：重复公开不重复发放积分或声望
    art_guild::on_artwork_published(&app.state, aid, "")
        .await
        .unwrap();
    assert_eq!(net(&art, aid).await, 120, "重复公开应幂等");
    assert_eq!(
        reputation_for_artwork(&art, aid).await,
        120,
        "重复公开不应重复发放声望"
    );

    // 撤稿 → 扣回到 0
    art_guild::on_artwork_withdrawn(&app.state, aid)
        .await
        .unwrap();
    assert_eq!(net(&art, aid).await, 0, "撤稿应扣回全部投稿积分");
    assert_eq!(
        reputation_for_artwork(&art, aid).await,
        120,
        "撤稿不回收投稿声望"
    );

    // 复审再公开 → 重新发放（隐藏后再公开理应有积分）
    art_guild::on_artwork_published(&app.state, aid, "")
        .await
        .unwrap();
    assert_eq!(net(&art, aid).await, 120, "复审公开应重新发放");

    // 兑换消耗 -50（redemption）：余额下降，但历史获得不受影响
    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
         VALUES(?, NULL, -50, '兑换「徽章」扣除金币', 'redemption', ?, ?)",
    )
    .bind(uid)
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();
    assert_eq!(balance(&art, uid).await, 70, "兑换后余额应扣减");
    assert_eq!(earned(&art, uid).await, 120, "兑换消耗不计入历史获得");

    // 撤稿：历史获得被扣回 0；兑换消耗仍在，余额可为负
    art_guild::on_artwork_withdrawn(&app.state, aid)
        .await
        .unwrap();
    assert_eq!(earned(&art, uid).await, 0, "撤稿应把历史获得积分扣回");
    assert_eq!(
        balance(&art, uid).await,
        -50,
        "撤稿扣回叠加既有兑换消耗，余额可为负"
    );
}

#[tokio::test]
async fn artwork_reward_settings_apply_to_new_snapshots_only() {
    use haruhi_server::modules::art_guild;

    let app = setup().await;
    let art = app.state.pools.art.clone();
    let uid = "u9010";
    let ts = "2026-01-01T00:00:00Z";

    sqlx::query(
        "INSERT INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at) \
         VALUES(?, 0, 'F', 'observer_clearance', ?, ?)",
    )
    .bind(uid)
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    let old_aid: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at) \
         VALUES('旧规则作品', ?, 'personal', 'haruhi', 'pending', ?) RETURNING id",
    )
    .bind(uid)
    .bind(ts)
    .fetch_one(&art)
    .await
    .unwrap();
    art_guild::on_artwork_published(&app.state, old_aid, "")
        .await
        .unwrap();

    sqlx::query(
        "UPDATE art_reward_settings
         SET points_multiplier_bps=30000, reputation_multiplier_bps=50000, updated_at=?
         WHERE id=1",
    )
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    art_guild::on_artwork_withdrawn(&app.state, old_aid)
        .await
        .unwrap();
    art_guild::on_artwork_published(&app.state, old_aid, "")
        .await
        .unwrap();

    let old_points: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE artwork_id=?")
            .bind(old_aid)
            .fetch_one(&art)
            .await
            .unwrap();
    let old_reputation: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(reputation), 0) FROM reputation_ledger WHERE artwork_id=?",
    )
    .bind(old_aid)
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(old_points, 120, "旧作品应按首次通过快照恢复积分");
    assert_eq!(old_reputation, 120, "旧作品不应被新声望倍率追溯");

    let new_aid: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at) \
         VALUES('活动作品', ?, 'personal', 'haruhi', 'pending', ?) RETURNING id",
    )
    .bind(uid)
    .bind(ts)
    .fetch_one(&art)
    .await
    .unwrap();
    art_guild::on_artwork_published(&app.state, new_aid, "")
        .await
        .unwrap();

    let new_points: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE artwork_id=?")
            .bind(new_aid)
            .fetch_one(&art)
            .await
            .unwrap();
    let new_reputation: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(reputation), 0) FROM reputation_ledger WHERE artwork_id=?",
    )
    .bind(new_aid)
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(new_points, 360, "新作品应使用当前积分倍率");
    assert_eq!(new_reputation, 600, "新作品应使用独立声望倍率");
}

#[tokio::test]
async fn guild_leaderboard_hides_zero_earned_coins_by_default() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let ts = "2026-01-01T00:00:00Z";

    for uid in [
        "u_lb_positive",
        "u_lb_zero",
        "u_lb_redemption_only",
        "u_lb_withdrawn",
    ] {
        sqlx::query(
            "INSERT INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at) \
             VALUES(?, 0, 'F', 'observer_clearance', ?, ?)",
        )
        .bind(uid)
        .bind(ts)
        .bind(ts)
        .execute(&art)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
         VALUES(?, NULL, 10, '测试获得金币', 'quest', ?, ?)",
    )
    .bind("u_lb_positive")
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
         VALUES(?, NULL, -10, '兑换「徽章」扣除金币', 'redemption', ?, ?)",
    )
    .bind("u_lb_redemption_only")
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    for (points, source_type) in [(10_i64, "quest"), (-10_i64, "withdraw")] {
        sqlx::query(
            "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
             VALUES(?, NULL, ?, '获得后撤回', ?, ?, ?)",
        )
        .bind("u_lb_withdrawn")
        .bind(points)
        .bind(source_type)
        .bind(ts)
        .bind(ts)
        .execute(&art)
        .await
        .unwrap();
    }

    let (status, body) = send(&app.router, get("/api/art/guild/leaderboard", None)).await;
    assert_eq!(status, StatusCode::OK);
    let uids: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["uid"].as_str().map(str::to_string))
        .collect();

    assert!(
        uids.contains(&"u_lb_positive".to_string()),
        "历史累计获得金币大于 0 的用户应显示"
    );
    assert!(
        !uids.contains(&"u_lb_zero".to_string()),
        "没有获得过金币的用户不应默认显示"
    );
    assert!(
        !uids.contains(&"u_lb_redemption_only".to_string()),
        "只有兑换消耗、历史累计获得金币为 0 的用户不应默认显示"
    );
    assert!(
        !uids.contains(&"u_lb_withdrawn".to_string()),
        "获得后撤回导致历史累计获得金币为 0 的用户不应默认显示"
    );
}

#[tokio::test]
async fn guild_budget_uses_manual_supplies_and_physical_spends() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    for body in [
        json!({ "budgetType": "quarterly", "amountUnit": "rmb", "amount": 600 }),
        json!({ "budgetType": "activity", "amountUnit": "coins", "amount": 1500 }),
    ] {
        let (status, body) = send(
            &app.router,
            post_json("/api/art/admin/guild/budget/supplies", body, Some(&token)),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "budget supply should be created: {body:?}"
        );
        assert_eq!(body["ok"], true);
    }

    for (reward_id, uid, frozen, status) in [
        (2_i64, "u_budget_pending", 100_i64, "pending"),
        (2_i64, "u_budget_approved", 300_i64, "approved"),
        (2_i64, "u_budget_fulfilled", 200_i64, "fulfilled"),
        (1_i64, "u_budget_virtual", 80_i64, "approved"),
    ] {
        sqlx::query(
            "INSERT INTO guild_reward_redemptions(reward_id, uid, frozen_coins, status, created_at) \
             VALUES(?,?,?,?,?)",
        )
        .bind(reward_id)
        .bind(uid)
        .bind(frozen)
        .bind(status)
        .bind(&ts)
        .execute(&art)
        .await
        .unwrap();
    }

    let (status, body) = send(&app.router, get("/api/art/guild/rewards", None)).await;
    assert_eq!(status, StatusCode::OK);
    let budget = &body["budget"];
    assert_eq!(budget["coinPerRmb"], 15);
    assert_eq!(budget["totalSupplyCoins"], 10500);
    assert_eq!(budget["spentPhysicalCoins"], 500);
    assert_eq!(budget["currentBudgetCoins"], 10000);
    assert!(
        budget["recentSupplies"].as_array().unwrap().len() == 2,
        "public rewards should expose recent manual supplies"
    );

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/budget", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["summary"]["totalSupplyCoins"], 10500);
    assert_eq!(body["summary"]["spentPhysicalCoins"], 500);
    assert_eq!(body["summary"]["currentBudgetCoins"], 10000);

    let supplies = body["supplies"].as_array().unwrap();
    assert_eq!(supplies.len(), 2);
    assert!(supplies.iter().any(|item| {
        item["budgetType"] == "quarterly"
            && item["amountUnit"] == "rmb"
            && item["amountInput"] == 600
            && item["amountCoins"] == 9000
    }));
    assert!(supplies.iter().any(|item| {
        item["budgetType"] == "activity"
            && item["amountUnit"] == "coins"
            && item["amountInput"] == 1500
            && item["amountCoins"] == 1500
    }));

    let spends = body["spends"].as_array().unwrap();
    assert_eq!(spends.len(), 2);
    assert_eq!(
        spends
            .iter()
            .map(|item| item["spentCoins"].as_i64().unwrap())
            .sum::<i64>(),
        500
    );
    assert!(
        spends.iter().all(|item| item["rewardType"] == "physical"
            && matches!(item["status"].as_str(), Some("approved" | "fulfilled"))),
        "budget spends should only include approved or fulfilled physical redemptions"
    );
}

#[tokio::test]
async fn guild_redemption_records_review_and_fulfillment_history() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let uid = format!("u{user_id}");
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let redemption_id: i64 = sqlx::query_scalar(
        "INSERT INTO guild_reward_redemptions(reward_id, uid, frozen_coins, status, user_note, created_at)
         VALUES(1, ?, 80, 'pending', '用于主页展示', ?) RETURNING id",
    )
    .bind(&uid)
    .bind(&ts)
    .fetch_one(&art)
    .await
    .unwrap();

    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/redemptions/{redemption_id}/fulfilled"),
            json!({ "note": "不能跳过审核" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/redemptions/{redemption_id}/approve"),
            json!({ "note": "审核通过，等待发放" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "approve failed: {body:?}");

    let (status, body) = send(
        &app.router,
        get("/api/art/guild/redemptions/me", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let redemption = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == redemption_id)
        .unwrap();
    assert_eq!(redemption["status"], "approved");
    assert_eq!(redemption["reviewNote"], "审核通过，等待发放");
    assert_eq!(redemption["history"].as_array().unwrap().len(), 2);
    assert!(redemption["history"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["label"] == "审核批准" && item["note"] == "审核通过，等待发放"));

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/redemptions/{redemption_id}/fulfilled"),
            json!({ "note": "奖励已发放" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "fulfill failed: {body:?}");

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/redemptions", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let redemption = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == redemption_id)
        .unwrap();
    assert_eq!(redemption["status"], "fulfilled");
    assert_eq!(redemption["reviewNote"], "审核通过，等待发放");
    assert_eq!(redemption["fulfilledNote"], "奖励已发放");
    assert!(redemption["history"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["label"] == "发放完成" && item["note"] == "奖励已发放"));
}

#[tokio::test]
async fn guild_budget_recounts_redemptions_when_reward_becomes_physical() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let (status, body) = send(
        &app.router,
        post_json(
            "/api/art/admin/guild/budget/supplies",
            json!({ "budgetType": "activity", "amountUnit": "coins", "amount": 1000 }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "budget supply failed: {body:?}");

    sqlx::query(
        "INSERT INTO guild_reward_redemptions(reward_id, uid, frozen_coins, status, created_at, reviewed_at) \
         VALUES(1, 'u_budget_retro', 80, 'approved', ?, ?)",
    )
    .bind(&ts)
    .bind(&ts)
    .execute(&art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/budget", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["summary"]["spentPhysicalCoins"], 0);
    assert_eq!(body["summary"]["currentBudgetCoins"], 1000);
    assert_eq!(body["spends"].as_array().unwrap().len(), 0);

    sqlx::query("UPDATE guild_rewards SET reward_type='physical', updated_at=? WHERE id=1")
        .bind(&ts)
        .execute(&art)
        .await
        .unwrap();

    let (status, body) = send(&app.router, get("/api/art/guild/rewards", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["budget"]["spentPhysicalCoins"], 80);
    assert_eq!(body["budget"]["currentBudgetCoins"], 920);

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/budget", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["summary"]["spentPhysicalCoins"], 80);
    assert_eq!(body["summary"]["currentBudgetCoins"], 920);
    assert!(body["spends"].as_array().unwrap().iter().any(|item| {
        item["rewardId"] == 1
            && item["uid"] == "u_budget_retro"
            && item["spentCoins"] == 80
            && item["rewardType"] == "physical"
    }));
}

#[tokio::test]
async fn guild_terminal_limits_claim_history_to_latest_ten() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let uid = format!("u{user_id}");

    for i in 0..12 {
        let title = format!("历史委托 {i:02}");
        let ts = format!("2099-01-{:02}T04:00:00Z", i + 1);
        let quest_id: i64 = sqlx::query_scalar(
            "INSERT INTO guild_quests(
                title, description, quest_type, difficulty, required_rating, required_access,
                condition_kind, target_count, reward_reputation, reward_coins,
                deadline_days, cycle_days, status, sort_order, created_at, updated_at
             ) VALUES(
                ?, '历史记录测试', 'limited', 'normal', 'F', 'observer_clearance',
                'browse_artworks', 1, 1, 0, NULL, NULL, 'active', ?, ?, ?
             ) RETURNING id",
        )
        .bind(&title)
        .bind(i)
        .bind(&ts)
        .bind(&ts)
        .fetch_one(&art)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO guild_quest_claims(
                quest_id, uid, cycle_key, status, progress, target_count,
                claimed_at, cycle_start_at, completed_at, rewarded_at
             ) VALUES(?, ?, ?, 'completed', 1, 1, ?, ?, ?, ?)",
        )
        .bind(quest_id)
        .bind(&uid)
        .bind(format!("history-{i:02}"))
        .bind(&ts)
        .bind(&ts)
        .bind(&ts)
        .bind(&ts)
        .execute(&art)
        .await
        .unwrap();
    }

    let (status, body) = send(&app.router, get("/api/art/guild/terminal", Some(&token))).await;
    assert_eq!(status, StatusCode::OK);
    let claims = body["claims"].as_array().unwrap();
    assert_eq!(claims.len(), 10);

    let titles: Vec<String> = claims
        .iter()
        .filter_map(|item| item["title"].as_str().map(str::to_string))
        .collect();
    assert_eq!(titles.first().map(String::as_str), Some("历史委托 11"));
    assert_eq!(titles.last().map(String::as_str), Some("历史委托 02"));
    assert!(!titles.iter().any(|title| title == "历史委托 00"));
    assert!(!titles.iter().any(|title| title == "历史委托 01"));
}

#[tokio::test]
async fn guild_reward_categories_are_managed_and_returned_with_rewards() {
    let app = setup().await;
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;

    let (status, body) = send(&app.router, get("/api/art/guild/rewards", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["categories"].as_array().unwrap().is_empty(),
        "fresh database should not seed reward categories"
    );

    let (status, body) = send(
        &app.router,
        post_json(
            "/api/art/admin/guild/reward-categories",
            json!({ "name": "游戏", "sortOrder": 5 }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "category create failed: {body:?}");
    let category_id = body["data"]["id"].as_i64().unwrap();

    for body in [
        json!({
            "name": "游戏兑换券",
            "description": "可兑换游戏相关补给",
            "rewardType": "virtual",
            "priceCoins": 150,
            "stock": -1,
            "requiredRating": "F",
            "requiredAccess": "observer_clearance",
            "status": "active",
            "sortOrder": 30,
            "categoryId": category_id
        }),
        json!({
            "name": "未分类补给",
            "description": "只在所有分类展示",
            "rewardType": "physical",
            "priceCoins": 220,
            "stock": 3,
            "requiredRating": "F",
            "requiredAccess": "observer_clearance",
            "status": "active",
            "sortOrder": 31
        }),
    ] {
        let (status, body) = send(
            &app.router,
            post_json("/api/art/admin/guild/rewards", body, Some(&token)),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "reward create failed: {body:?}");
    }

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/rewards", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["categories"][0]["id"], category_id);
    let admin_rewards = body["data"].as_array().unwrap();
    let categorized = admin_rewards
        .iter()
        .find(|item| item["name"] == "游戏兑换券")
        .expect("categorized reward should be returned to admin");
    assert_eq!(categorized["categoryId"], category_id);
    assert_eq!(categorized["categoryName"], "游戏");

    let (status, body) = send(&app.router, get("/api/art/guild/rewards", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["categories"][0]["name"], "游戏");
    let public_rewards = body["data"].as_array().unwrap();
    let uncategorized = public_rewards
        .iter()
        .find(|item| item["name"] == "未分类补给")
        .expect("uncategorized reward should still be visible");
    assert!(uncategorized["categoryId"].is_null());

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/reward-categories/{category_id}/status"),
            json!({ "status": "paused" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "category pause failed: {body:?}");

    let (status, body) = send(&app.router, get("/api/art/guild/rewards", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["categories"].as_array().unwrap().is_empty(),
        "paused categories should not create public category tabs"
    );
}

#[tokio::test]
async fn manual_guild_quest_accepts_artwork_submissions_before_admin_approval() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let uid = format!("u{user_id}");
    let quest_created_at = "2026-06-30T20:30:00.000Z";

    let quest_id: i64 = sqlx::query_scalar(
        "INSERT INTO guild_quests(
            title, description, quest_type, difficulty, required_rating, required_access,
            condition_kind, target_count, reward_reputation, reward_coins,
            deadline_days, cycle_days, status, sort_order, created_at, updated_at
         ) VALUES(
            '指定画作委托', '提交委托发布日后的作品。', 'limited', 'hard', 'F', 'observer_clearance',
            'manual_admin_verify', 2, 80, 45, NULL, NULL, 'active', 1, ?, ?
         ) RETURNING id",
    )
    .bind(quest_created_at)
    .bind(quest_created_at)
    .fetch_one(&art)
    .await
    .unwrap();

    let eligible_a: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('合格作品 A', ?, 'personal', 'haruhi', 'approved', '2026-06-30T16:00:00.000Z', '2026-06-30T16:00:00.000Z')
         RETURNING id",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    let eligible_b: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('合格作品 B', ?, 'personal', 'other', 'approved', '2026-07-01T01:00:00.000Z', '2026-07-01T01:00:00.000Z')
         RETURNING id",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    let too_old: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('过早作品', ?, 'personal', 'haruhi', 'approved', '2026-06-30T15:59:59.000Z', '2026-06-30T15:59:59.000Z')
         RETURNING id",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('待审作品', ?, 'personal', 'haruhi', 'pending', '2026-07-01T02:00:00.000Z', NULL)",
    )
    .bind(&uid)
    .execute(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('他人作品', 'u_other', 'personal', 'haruhi', 'approved', '2026-07-01T02:00:00.000Z', '2026-07-01T02:00:00.000Z')",
    )
    .execute(&art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/guild/quests/{quest_id}/claim"),
            json!({}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "manual quest should be claimable: {body:?}"
    );

    let (status, body) = send(
        &app.router,
        get(
            &format!("/api/art/guild/quests/{quest_id}/submission-artworks"),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let artwork_ids: Vec<i64> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert_eq!(artwork_ids, vec![eligible_b, eligible_a]);
    assert!(
        !artwork_ids.contains(&too_old),
        "委托发布当天北京时间 00:00 之前的作品不应可提交"
    );

    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/guild/quests/{quest_id}/submit-artworks"),
            json!({ "artworkIds": [too_old] }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/guild/quests/{quest_id}/submit-artworks"),
            json!({ "artworkIds": [eligible_a] }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["submittedCount"], 1);

    let claim_id: i64 = sqlx::query_scalar(
        "SELECT id FROM guild_quest_claims WHERE quest_id=? AND uid=? AND status='active'",
    )
    .bind(quest_id)
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/quest-claims/{claim_id}/approve"),
            json!({ "note": "验收通过" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "提交数量不足时不能批准");

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/guild/quests/{quest_id}/submit-artworks"),
            json!({ "artworkIds": [eligible_a, eligible_b] }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["submittedCount"], 2);

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/quest-claims", Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let claim = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_i64() == Some(claim_id))
        .unwrap();
    assert_eq!(claim["submittedCount"], 2);
    assert_eq!(claim["submittedArtworks"].as_array().unwrap().len(), 2);

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/quest-claims/{claim_id}/approve"),
            json!({ "note": "验收通过" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "补齐作品后应可批准: {body:?}");
    let (claim_status, progress): (String, i64) =
        sqlx::query_as("SELECT status, progress FROM guild_quest_claims WHERE id=?")
            .bind(claim_id)
            .fetch_one(&art)
            .await
            .unwrap();
    assert_eq!(claim_status, "completed");
    assert_eq!(progress, 2);
    let reward_coins: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE uid=? AND source_type='quest'",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(reward_coins, 45);
}

#[tokio::test]
async fn admin_creator_production_stats_use_recent_approved_art_and_positive_coins() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let old = "2000-01-01T00:00:00Z";

    for uid in ["u_stats_a", "u_stats_b"] {
        sqlx::query("INSERT INTO creators(uid, avatar_url, created_at) VALUES(?,'',?)")
            .bind(uid)
            .bind(&now)
            .execute(&art)
            .await
            .unwrap();
    }

    for (title, uid, status, created_at, reviewed_at) in [
        (
            "近期作品 A1",
            "u_stats_a",
            "approved",
            now.as_str(),
            now.as_str(),
        ),
        (
            "近期作品 A2",
            "u_stats_a",
            "approved",
            now.as_str(),
            now.as_str(),
        ),
        (
            "待审作品 A3",
            "u_stats_a",
            "pending",
            now.as_str(),
            now.as_str(),
        ),
        (
            "近期作品 B1",
            "u_stats_b",
            "approved",
            now.as_str(),
            now.as_str(),
        ),
        ("历史作品 B2", "u_stats_b", "approved", old, old),
    ] {
        sqlx::query(
            "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at) \
             VALUES(?, ?, 'personal', 'haruhi', ?, ?, ?)",
        )
        .bind(title)
        .bind(uid)
        .bind(status)
        .bind(created_at)
        .bind(reviewed_at)
        .execute(&art)
        .await
        .unwrap();
    }

    for (uid, points, source_type, created_at) in [
        ("u_stats_a", 90_i64, "upload", now.as_str()),
        ("u_stats_a", 30_i64, "quest", now.as_str()),
        ("u_stats_a", -10_i64, "redemption", now.as_str()),
        ("u_stats_b", 45_i64, "upload", now.as_str()),
        ("u_stats_b", 100_i64, "upload", old),
    ] {
        sqlx::query(
            "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
             VALUES(?, NULL, ?, '统计测试', ?, ?, ?)",
        )
        .bind(uid)
        .bind(points)
        .bind(source_type)
        .bind(created_at)
        .bind(created_at)
        .execute(&art)
        .await
        .unwrap();
    }

    let (status, body) = send(
        &app.router,
        get(
            "/api/art/admin/guild/creator-production-stats?months=3",
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["months"], 3);
    assert_eq!(body["overall"]["artworksTotal"], 3);
    assert_eq!(body["overall"]["coinsTotal"], 165);

    let rows = body["data"].as_array().unwrap();
    let find = |uid: &str| {
        rows.iter()
            .find(|row| row["uid"].as_str() == Some(uid))
            .unwrap()
    };
    let a = find("u_stats_a");
    assert_eq!(a["artworksTotal"], 2);
    assert_eq!(a["coinsTotal"], 120);
    assert!((a["avgArtworksPerMonth"].as_f64().unwrap() - 0.7).abs() < 0.01);
    assert_eq!(a["avgCoinsPerMonth"], 40.0);

    let b = find("u_stats_b");
    assert_eq!(b["artworksTotal"], 1);
    assert_eq!(b["coinsTotal"], 45);
    assert_eq!(b["avgCoinsPerMonth"], 15.0);

    let ten_days_ago = (chrono::Utc::now() - chrono::Duration::days(10))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    sqlx::query(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at) \
         VALUES('十天前作品', 'u_stats_a', 'personal', 'haruhi', 'approved', ?, ?)",
    )
    .bind(&ten_days_ago)
    .bind(&ten_days_ago)
    .execute(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
         VALUES('u_stats_a', NULL, 60, '统计测试', 'upload', ?, ?)",
    )
    .bind(&ten_days_ago)
    .bind(&ten_days_ago)
    .execute(&art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        get(
            "/api/art/admin/guild/creator-production-stats?window=week",
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["window"], "week");
    assert_eq!(body["windowLabel"], "近1周");
    assert_eq!(body["days"], 7);
    assert_eq!(body["overall"]["artworksTotal"], 3);
    assert_eq!(body["overall"]["coinsTotal"], 165);
}

// 评论署名：登录用户自动用账号昵称署名（忽略前端自报、归属 author_user_id）；
// 未登录用户不允许评论（强制登录，返回 401）——前端在评论区改为展示登录提示。
#[tokio::test]
async fn comment_uses_nickname_for_member_and_rejects_anonymous() {
    let app = setup().await;
    let art = app.state.pools.art.clone();

    let aid: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, status, created_at) \
         VALUES('作品', 'approved', '2026-01-01T00:00:00Z') RETURNING id",
    )
    .fetch_one(&art)
    .await
    .unwrap();

    // 1) 登录用户：忽略前端自报署名，使用账号昵称 + 归属 author_user_id
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let (s1, _) = send(
        &app.router,
        post_json(
            "/api/art/comments",
            json!({ "artwork_id": aid, "user_name": "前端自报应忽略", "body": "登录用户评论" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "登录用户评论应成功");
    let (uname, author): (String, Option<i64>) =
        sqlx::query_as("SELECT user_name, author_user_id FROM comments WHERE body='登录用户评论'")
            .fetch_one(&art)
            .await
            .unwrap();
    assert_ne!(uname, "前端自报应忽略", "登录用户署名不应取前端自报值");
    assert!(author.is_some(), "登录用户评论应归属 author_user_id");

    // 2) 未登录用户：强制登录，评论被拒（401），且不落库
    let (s2, _) = send(
        &app.router,
        post_json(
            "/api/art/comments",
            json!({ "artwork_id": aid, "user_name": "路人甲", "body": "匿名评论" }),
            None,
        ),
    )
    .await;
    assert_eq!(s2, StatusCode::UNAUTHORIZED, "未登录用户评论应被拒（401）");
    let leaked: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM comments WHERE body='匿名评论'")
        .fetch_one(&art)
        .await
        .unwrap();
    assert_eq!(leaked, 0, "未登录评论不应落库");
}

// 书架封面缩略图端点：校验白名单宽度 / novel 前缀 / 路径穿越 / 源存在性；
// 合法封面在有 vips 时返回 200 webp，无 vips 时回退原图重定向（两者皆非 4xx/5xx）。
#[tokio::test]
async fn novel_cover_thumb_validates_and_serves() {
    let app = setup().await;
    let covers = app.state.cfg.uploads_dir.join("novel").join("covers");
    std::fs::create_dir_all(&covers).unwrap();
    std::fs::write(covers.join("c1.png"), b"\x89PNG\r\n\x1a\n-fake-bytes").unwrap();

    // 合法封面：有 vips → 200 webp；无 vips → 回退原图重定向（3xx）。不得是错误。
    let (s, _) = send(
        &app.router,
        get("/api/novel/thumb?path=novel/covers/c1.png&w=320", None),
    )
    .await;
    assert!(
        s == StatusCode::OK || s.is_redirection(),
        "合法封面应 200 或回退重定向，实际 {s}"
    );

    // 非白名单宽度 → 400
    let (s, _) = send(
        &app.router,
        get("/api/novel/thumb?path=novel/covers/c1.png&w=999", None),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "非白名单宽度应 400");

    // 越权前缀（art/）→ 400
    let (s, _) = send(
        &app.router,
        get("/api/novel/thumb?path=art/x.png&w=320", None),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "非 novel/ 前缀应 400");

    // 路径穿越 → 400
    let (s, _) = send(
        &app.router,
        get("/api/novel/thumb?path=novel/../secret.png&w=320", None),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "路径穿越应 400");

    // 源不存在 → 404
    let (s, _) = send(
        &app.router,
        get("/api/novel/thumb?path=novel/covers/missing.png&w=320", None),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "封面不存在应 404");
}
