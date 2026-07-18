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
        apps_dir: PathBuf::from("./apps"),
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
        // 资源站（download）语雀同步配置：测试不启用（token None → 同步不启动）
        yuque_token: None,
        yuque_repo: "staff-sqlmik/phgf5z".into(),
        yuque_sync_interval_secs: 21_600,
        // 语音工坊（voice）：测试不启探活任务，基址随意（不会被访问）
        voice_tts_base: "http://127.0.0.1:9872".into(),
        voice_rvc_base: "http://127.0.0.1:7865".into(),
        voice_shared_key: None,
        voice_probe_interval_secs: 60,
        voice_tts_timeout_secs: 180,
        voice_rvc_timeout_secs: 600,
        voice_user_cooldown_secs: 30,
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
        download: haruhi_server::modules::download::new_cache(),
        voice: haruhi_server::modules::voice::VoiceState::new(),
        seo_templates: haruhi_server::modules::seo::template::new_cache(),
        creator_feed: haruhi_server::modules::art::CreatorFeedCache::default(),
        recommendation_feed: haruhi_server::modules::art::RecommendationFeedCache::default(),
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

fn get_with_cookie(path: &str, cookie: &str) -> Request<Body> {
    let mut req = get(path, None);
    req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
    req
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

fn put_json(path: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method("PUT")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    b.body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn patch_json(path: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method("PATCH")
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

#[tokio::test]
async fn admin_update_article_ignores_unknown_fields() {
    // 回归：后台编辑器提交的 payload 可能带 articles 表不存在的字段。
    // update_article 必须按列白名单过滤，否则动态 UPDATE 会拼出未知列 → SQLite 报
    // "no such column" → 500。此测试钉住「带未知字段的编辑仍成功且脏字段被丢弃」。
    let app = setup().await;
    let admin = login(&app.router, ADMIN_USER, ADMIN_PASS).await;

    // 超管建文章（status=published），拿到 id
    let (s, j) = send(
        &app.router,
        post_json(
            "/api/news/articles",
            json!({ "title": "原标题", "type": "news", "content": [] }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "建文章应成功: {j:?}");
    let id = j["data"]["id"].as_i64().expect("应返回文章 id");

    // 编辑并携带脏字段 bogusColumn（表中无此列）→ 应 200，而非 500
    let (s, j) = send(
        &app.router,
        put_json(
            &format!("/api/news/articles/{id}"),
            json!({ "title": "新标题", "bogusColumn": "x", "type": "news" }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "带未知字段的编辑应被过滤后成功: {j:?}");

    // 白名单字段确实落库
    let (s, detail) = send(
        &app.router,
        get(&format!("/api/news/articles/{id}"), Some(&admin)),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        detail["data"]["title"], "新标题",
        "标题应已更新: {detail:?}"
    );
}

#[tokio::test]
async fn admin_article_header_note_round_trips() {
    // 角标（headerNote）作为真实列：建文章可写入、编辑可改、GET 回显；留空回退按 type。
    let app = setup().await;
    let admin = login(&app.router, ADMIN_USER, ADMIN_PASS).await;

    // 建文章带角标
    let (s, j) = send(
        &app.router,
        post_json(
            "/api/news/articles",
            json!({ "title": "标题", "type": "news", "headerNote": "独家", "content": [] }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "建文章应成功: {j:?}");
    let id = j["data"]["id"].as_i64().expect("应返回文章 id");

    // GET 详情回显角标
    let (s, detail) = send(
        &app.router,
        get(&format!("/api/news/articles/{id}"), Some(&admin)),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        detail["data"]["headerNote"], "独家",
        "角标应回显: {detail:?}"
    );

    // 列表项也应带角标
    let (s, list) = send(&app.router, get("/api/news/articles", None)).await;
    assert_eq!(s, StatusCode::OK);
    let items = list
        .get("data")
        .and_then(|d| d.as_array())
        .unwrap_or_else(|| list.as_array().expect("列表应为数组或含 data 数组"));
    let found = items
        .iter()
        .find(|it| it["id"].as_i64() == Some(id))
        .expect("列表应含刚建的文章");
    assert_eq!(found["headerNote"], "独家", "列表项应带角标: {found:?}");

    // 编辑改角标
    let (s, _) = send(
        &app.router,
        put_json(
            &format!("/api/news/articles/{id}"),
            json!({ "title": "标题", "type": "news", "headerNote": "重磅" }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (_, detail) = send(
        &app.router,
        get(&format!("/api/news/articles/{id}"), Some(&admin)),
    )
    .await;
    assert_eq!(
        detail["data"]["headerNote"], "重磅",
        "角标应被更新: {detail:?}"
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
async fn every_public_artwork_preview_includes_history_popularity() {
    let app = setup().await;
    seed_artwork(
        &app.state,
        "latest-with-popularity",
        "approved",
        "2026-07-01 00:00:00",
    )
    .await;

    let (status, body) = send(
        &app.router,
        get("/api/art/artworks?sort=time&pageSize=6", None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item = &body["data"][0];
    assert_eq!(item["popularity"]["range"], "history");
    assert!(item["popularity_score"].as_i64().is_some());
}

#[tokio::test]
async fn creator_exhibits_default_to_top_three_then_honor_explicit_selection() {
    let app = setup().await;
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    for index in 1..=4_i64 {
        sqlx::query(
            "INSERT INTO artworks(\
                title, uploader_name, uploader_uid, author_user_id, source_type, content_type, \
                status, like_total, created_at, reviewed_at, random_key\
             ) VALUES(?, '测试作者', 'u1', 1, 'personal', 'haruhi', 'approved', ?, \
                      '2026-07-01 00:00:00', '2026-07-01 00:00:00', ?)",
        )
        .bind(format!("展位作品{index}"))
        .bind(index * 10)
        .bind(index)
        .execute(&app.state.pools.art)
        .await
        .unwrap();
    }

    let (status, initial) = send(&app.router, get("/api/art/creator-exhibits", None)).await;
    assert_eq!(status, StatusCode::OK);
    let initial_items = initial["data"][0]["items"].as_array().unwrap();
    assert_eq!(initial_items.len(), 3);
    assert_eq!(initial_items[0]["title"], "展位作品4");
    assert!(initial_items
        .iter()
        .all(|item| item["popularity_score"].is_i64()));

    let top_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='展位作品4'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    let (status, toggled) = send(
        &app.router,
        patch_json(
            &format!("/api/art/me/artworks/{top_id}"),
            json!({ "exhibit_enabled": false }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "首次调整应固化默认选择: {toggled:?}"
    );

    let (_, after) = send(&app.router, get("/api/art/creator-exhibits", None)).await;
    let after_items = after["data"][0]["items"].as_array().unwrap();
    assert_eq!(after_items.len(), 2);
    assert!(after_items.iter().all(|item| item["title"] != "展位作品4"));

    let (_, mine) = send(
        &app.router,
        get("/api/art/me/artworks?pageSize=24", Some(&token)),
    )
    .await;
    let enabled_count = mine["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["exhibit_enabled"] == true)
        .count();
    assert_eq!(enabled_count, 2);
}

#[tokio::test]
async fn creator_feed_uses_cached_random_pagination_and_recommends_three_works() {
    let app = setup().await;
    for creator in 1..=5_i64 {
        for artwork in 1..=4_i64 {
            sqlx::query(
                "INSERT INTO artworks(\
                    title, uploader_name, uploader_uid, source_type, content_type, status, \
                    like_total, exhibit_enabled, created_at, reviewed_at, random_key\
                 ) VALUES(?, ?, ?, 'personal', 'haruhi', 'approved', ?, 0, ?, ?, ?)",
            )
            .bind(format!("作者{creator}作品{artwork}"))
            .bind(format!("作者{creator}"))
            .bind(format!("feed-{creator}"))
            .bind(artwork * 10)
            .bind(format!("2026-07-{artwork:02} 00:00:00"))
            .bind(format!("2026-07-{artwork:02} 00:00:00"))
            .bind(creator * 100 + artwork)
            .execute(&app.state.pools.art)
            .await
            .unwrap();
        }
    }

    let (status, first) = send(
        &app.router,
        get("/api/art/creators/feed?page=1&pageSize=2", None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["total"], 5);
    assert_eq!(first["page"], 1);
    assert_eq!(first["hasMore"], true);
    assert_eq!(first["algorithmVersion"], "hybrid-v1");
    let feed_id = first["feedId"].as_str().unwrap();
    let first_data = first["data"].as_array().unwrap();
    assert_eq!(first_data.len(), 2);
    assert!(first_data.iter().all(|creator| {
        let items = creator["items"].as_array().unwrap();
        items.len() == 3 && items.iter().all(|item| item["exhibit_enabled"] == false)
    }));

    let (_, repeated) = send(
        &app.router,
        get(
            &format!("/api/art/creators/feed?page=1&pageSize=2&feedId={feed_id}"),
            None,
        ),
    )
    .await;
    assert_eq!(repeated["feedId"], first["feedId"]);
    assert_eq!(repeated["data"], first["data"]);

    let (_, second) = send(
        &app.router,
        get(
            &format!("/api/art/creators/feed?page=2&pageSize=2&feedId={feed_id}"),
            None,
        ),
    )
    .await;
    let first_uids: std::collections::HashSet<&str> = first_data
        .iter()
        .filter_map(|creator| creator["uid"].as_str())
        .collect();
    assert!(second["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|creator| creator["uid"].as_str())
        .all(|uid| !first_uids.contains(uid)));

    let (_, last) = send(
        &app.router,
        get(
            &format!("/api/art/creators/feed?page=3&pageSize=2&feedId={feed_id}"),
            None,
        ),
    )
    .await;
    assert_eq!(last["data"].as_array().unwrap().len(), 1);
    assert_eq!(last["hasMore"], false);

    let (_, reset) = send(
        &app.router,
        get(
            "/api/art/creators/feed?page=3&pageSize=2&feedId=expired-feed",
            None,
        ),
    )
    .await;
    assert_eq!(reset["page"], 1);
    assert_eq!(reset["cacheReset"], true);
    assert_ne!(reset["feedId"], "expired-feed");
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
async fn art_recommendation_feed_cursor_avoids_duplicates_without_event_writes() {
    let app = setup().await;
    for i in 0..30 {
        seed_artwork(
            &app.state,
            &format!("feed-recommendation-{i}"),
            "approved",
            &format!("2026-07-{:02} 00:00:00", (i % 28) + 1),
        )
        .await;
    }

    let (status, headers, first) = send_full(
        &app.router,
        get(
            "/api/art/recommendations?limit=8&content_type=haruhi&source_type=network",
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["data"].as_array().unwrap().len(), 8);
    assert_eq!(first["hasMore"], true);
    assert_eq!(first["cacheReset"], false);
    let feed_id = first["feedId"].as_str().unwrap();
    let cookie = cookie_header_from_set_cookie(&headers);
    let first_ids: std::collections::HashSet<i64> = first["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();

    let (status, second) = send(
        &app.router,
        get_with_cookie(
            &format!(
                "/api/art/recommendations?limit=8&content_type=haruhi&source_type=network&feedId={feed_id}"
            ),
            &cookie,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(second["feedId"], first["feedId"]);
    assert_eq!(second["cacheReset"], false);
    let second_ids: std::collections::HashSet<i64> = second["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert!(
        first_ids.is_disjoint(&second_ids),
        "同一推荐流的连续批次不应依赖曝光事件也能严格去重"
    );

    let (_, refreshed) = send(
        &app.router,
        get_with_cookie(
            "/api/art/recommendations?limit=8&content_type=haruhi&source_type=network",
            &cookie,
        ),
    )
    .await;
    assert_ne!(refreshed["feedId"], first["feedId"]);
}

#[tokio::test]
async fn art_recommendations_do_not_persist_served_events_and_impressions_avoid_repeats() {
    let app = setup().await;
    for i in 0..20 {
        seed_artwork(
            &app.state,
            &format!("recommendation-art-{i}"),
            "approved",
            &format!("2026-06-{:02} 00:00:00", i + 1),
        )
        .await;
    }

    let (status, headers, first) =
        send_full(&app.router, get("/api/art/recommendations?limit=8", None)).await;
    assert_eq!(status, StatusCode::OK, "首次推荐应成功: {first:?}");
    assert_eq!(first["algorithmVersion"], "hybrid-v1");
    assert_eq!(first["data"].as_array().unwrap().len(), 8);
    assert!(first["batchId"].as_str().is_some());
    let cookie = cookie_header_from_set_cookie(&headers);
    assert!(cookie.contains("haruhi_anon="));

    let first_ids: std::collections::HashSet<i64> = first["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    let event_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM recommendation_events")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    assert_eq!(event_count, 0, "推荐 GET 不应持久化 served 事件");

    let batch_id = first["batchId"].as_str().unwrap();
    let impressions: Vec<Value> = first["data"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(position, item)| {
            json!({
                "artwork_id": item["id"],
                "event_type": "impression",
                "batch_id": batch_id,
                "source": "recommendation",
                "position": position,
            })
        })
        .collect();
    let (record_status, recorded) = send(
        &app.router,
        post_json_with_cookie(
            "/api/art/recommendation-events",
            json!({ "session_id": "read-only-test", "events": impressions }),
            &cookie,
        ),
    )
    .await;
    assert_eq!(
        record_status,
        StatusCode::OK,
        "曝光事件应成功: {recorded:?}"
    );
    assert_eq!(recorded["accepted"], 8);

    let (status, second) = send(
        &app.router,
        get_with_cookie("/api/art/recommendations?limit=8", &cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let second_ids: std::collections::HashSet<i64> = second["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert!(
        first_ids.is_disjoint(&second_ids),
        "作品池充足时，相邻推荐批次不应重复"
    );
}

#[tokio::test]
async fn art_recommendations_learn_tag_affinity_from_open_and_dwell() {
    let app = setup().await;
    for i in 0..24 {
        let title = if i < 10 {
            format!("favorite-{i}")
        } else {
            format!("other-{i}")
        };
        seed_artwork(
            &app.state,
            &title,
            "approved",
            &format!("2026-05-{:02} 00:00:00", (i % 28) + 1),
        )
        .await;
        let tag = if i < 10 { "favorite" } else { "other" };
        sqlx::query("UPDATE artworks SET tags_json=?, tags_norm=?, uploader_uid=? WHERE title=?")
            .bind(format!("[\"{tag}\"]"))
            .bind(format!(" {tag} "))
            .bind(format!("creator-{i}"))
            .bind(&title)
            .execute(&app.state.pools.art)
            .await
            .unwrap();
    }

    let (_, headers, _) =
        send_full(&app.router, get("/api/art/recommendations?limit=8", None)).await;
    let cookie = cookie_header_from_set_cookie(&headers);
    let favorite_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='favorite-0'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    let (status, recorded) = send(
        &app.router,
        post_json_with_cookie(
            "/api/art/recommendation-events",
            json!({
                "session_id": "recommendation-test",
                "events": [
                    { "artwork_id": favorite_id, "event_type": "open", "source": "test" },
                    { "artwork_id": favorite_id, "event_type": "dwell", "dwell_ms": 120000, "source": "test" }
                ]
            }),
            &cookie,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "行为记录应成功: {recorded:?}");
    assert_eq!(recorded["accepted"], 2);

    let (status, personalized) = send(
        &app.router,
        get_with_cookie("/api/art/recommendations?limit=8", &cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(personalized["personalized"], true);
    let first_two = &personalized["data"].as_array().unwrap()[..2];
    assert!(
        first_two.iter().all(|item| item["title"]
            .as_str()
            .unwrap_or("")
            .starts_with("favorite-")),
        "偏好通道前两位应优先召回同标签作品: {first_two:?}"
    );
}

#[tokio::test]
async fn art_popular_list_uses_windowed_views_and_unique_engagement() {
    let app = setup().await;
    let pool = &app.state.pools.art;
    for title in ["week-popular", "year-popular", "history-popular"] {
        seed_artwork(&app.state, title, "approved", "2024-04-01 00:00:00").await;
    }

    let activity = [
        ("week-popular", "week", 40_i64, 8_i64, 4_i64, "-1 day"),
        ("year-popular", "year", 200, 30, 10, "-30 days"),
        ("history-popular", "history", 600, 60, 18, "-500 days"),
    ];
    for (title, actor_prefix, views, likes, comments, age) in activity {
        let artwork_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title=?")
            .bind(title)
            .fetch_one(pool)
            .await
            .unwrap();
        sqlx::query("UPDATE artworks SET like_total=? WHERE id=?")
            .bind(likes)
            .bind(artwork_id)
            .execute(pool)
            .await
            .unwrap();
        for index in 0..views {
            sqlx::query(
                "INSERT INTO artwork_views(artwork_id, actor_key, view_bucket, viewed_at) VALUES(?,?,?,datetime('now', ?))",
            )
            .bind(artwork_id)
            .bind(format!("{actor_prefix}-viewer-{index}"))
            .bind(index)
            .bind(age)
            .execute(pool)
            .await
            .unwrap();
        }
        for index in 0..likes {
            sqlx::query(
                "INSERT INTO likes_daily(anon_id, target_type, target_id, day, count, created_at, updated_at) VALUES(?, 'artwork', ?, date('now', ?), 1, datetime('now', ?), datetime('now', ?))",
            )
            .bind(format!("{actor_prefix}-liker-{index}"))
            .bind(artwork_id)
            .bind(age)
            .bind(age)
            .bind(age)
            .execute(pool)
            .await
            .unwrap();
        }
        for index in 0..comments {
            sqlx::query(
                "INSERT INTO comments(artwork_id, anon_id, body, status, created_at) VALUES(?, ?, '公开评论', 'public', datetime('now', ?))",
            )
            .bind(artwork_id)
            .bind(format!("{actor_prefix}-commenter-{index}"))
            .bind(age)
            .execute(pool)
            .await
            .unwrap();
        }
    }

    for (range, expected, views, likes, comments) in [
        ("week", "week-popular", 40, 8, 4),
        ("year", "year-popular", 200, 30, 10),
        ("history", "history-popular", 600, 60, 18),
    ] {
        let path = format!("/api/art/artworks?sort=popular&range={range}&pageSize=6");
        let (status, body) = send(&app.router, get(&path, None)).await;
        assert_eq!(status, StatusCode::OK);
        let first = &body["data"][0];
        assert_eq!(first["title"], expected, "{range} 榜首应使用对应窗口");
        assert_eq!(first["popularity"]["range"], range);
        assert_eq!(first["popularity"]["views"], views);
        assert_eq!(first["popularity"]["likes"], likes);
        assert_eq!(first["popularity"]["comments"], comments);
        assert!(first["popularity_score"].as_i64().unwrap_or(0) > 0);
    }
}

#[tokio::test]
async fn artwork_views_dedupe_the_same_visitor_within_thirty_minutes() {
    let app = setup().await;
    seed_artwork(
        &app.state,
        "deduped-view",
        "approved",
        "2026-07-01 00:00:00",
    )
    .await;
    let artwork_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='deduped-view'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();

    let detail_path = format!("/api/art/artworks/{artwork_id}");
    let (status, headers, _) = send_full(&app.router, get(&detail_path, None)).await;
    assert_eq!(status, StatusCode::OK);
    let cookie = cookie_header_from_set_cookie(&headers);
    let (status, _) = send(&app.router, get_with_cookie(&detail_path, &cookie)).await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = send(
        &app.router,
        get("/api/art/artworks?sort=popular&range=week&pageSize=6", None),
    )
    .await;
    assert_eq!(body["data"][0]["title"], "deduped-view");
    assert_eq!(body["data"][0]["popularity"]["views"], 1);
}

#[tokio::test]
async fn artwork_detail_exposes_metadata_and_related_works_follow_current_artwork_affinity() {
    let app = setup().await;
    let pool = &app.state.pools.art;
    for title in [
        "关联源作品",
        "同作者作品",
        "同标签强关联",
        "同主题弱关联",
        "无关高热作品",
        "备用关联作品",
    ] {
        seed_artwork(&app.state, title, "approved", "2026-07-01 00:00:00").await;
    }

    sqlx::query(
        "UPDATE artworks SET uploader_uid='u1', uploader_name='旧署名', tags_json='[\"夏日\",\"列车\"]', tags_norm=' 夏日 列车 ', content_type='haruhi', source_type='personal' WHERE title='关联源作品'",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET uploader_uid='u1', tags_json='[\"夏日\",\"列车\"]', tags_norm=' 夏日 列车 ' WHERE title='同作者作品'",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET uploader_uid='creator-a', tags_json='[\"夏日\",\"列车\"]', tags_norm=' 夏日 列车 ', content_type='haruhi', source_type='personal' WHERE title='同标签强关联'",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET uploader_uid='creator-b', tags_json='[\"夏日\"]', tags_norm=' 夏日 ', content_type='haruhi', source_type='personal' WHERE title='同主题弱关联'",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET uploader_uid='creator-c', tags_json='[\"冬日\"]', tags_norm=' 冬日 ', content_type='other', source_type='network', like_total=999 WHERE title='无关高热作品'",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET uploader_uid='creator-d', tags_json='[\"列车\"]', tags_norm=' 列车 ', content_type='haruhi', source_type='personal' WHERE title='备用关联作品'",
    )
    .execute(pool)
    .await
    .unwrap();

    let source_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='关联源作品'")
        .fetch_one(pool)
        .await
        .unwrap();
    let (status, detail) = send(
        &app.router,
        get(&format!("/api/art/artworks/{source_id}"), None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(detail["data"]["uploader_display_name"].is_string());
    assert_eq!(detail["data"]["popularity"]["range"], "history");
    assert_eq!(detail["data"]["popularity"]["views"], 1);

    let (status, related) = send(
        &app.router,
        get(
            &format!("/api/art/artworks/{source_id}/related?limit=4"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(related["strategy"], "artwork-affinity-v1");
    let items = related["data"].as_array().unwrap();
    assert_eq!(items.len(), 4);
    assert_eq!(items[0]["title"], "同标签强关联");
    assert!(items.iter().all(|item| item["id"] != source_id));
    assert!(items.iter().all(|item| item["uploader_uid"] != "u1"));
    assert!(items.iter().all(|item| item["popularity_score"].is_i64()));
}

#[tokio::test]
async fn artwork_creator_neighbors_are_the_immediate_newer_and_older_submissions() {
    let app = setup().await;
    for (title, created_at) in [
        ("作者稿件-最旧", "2026-01-01 00:00:00"),
        ("作者稿件-较旧", "2026-02-01 00:00:00"),
        ("作者稿件-当前", "2026-03-01 00:00:00"),
        ("作者稿件-较新", "2026-04-01 00:00:00"),
        ("作者稿件-最新", "2026-05-01 00:00:00"),
    ] {
        seed_artwork(&app.state, title, "approved", created_at).await;
    }
    sqlx::query("UPDATE artworks SET uploader_uid='neighbor-author' WHERE title LIKE '作者稿件-%'")
        .execute(&app.state.pools.art)
        .await
        .unwrap();
    let current_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='作者稿件-当前'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();

    let (status, body) = send(
        &app.router,
        get(
            &format!("/api/art/artworks/{current_id}/creator-neighbors"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["currentIndex"], 1);
    let titles: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["title"].as_str())
        .collect();
    assert_eq!(
        titles,
        vec!["作者稿件-较新", "作者稿件-当前", "作者稿件-较旧"]
    );
}

#[tokio::test]
async fn artwork_creator_timeline_returns_every_public_submission_newest_first() {
    let app = setup().await;
    for (title, created_at) in [
        ("时间线-最旧", "2026-01-01 00:00:00"),
        ("时间线-较旧", "2026-02-01 00:00:00"),
        ("时间线-当前", "2026-03-01 00:00:00"),
        ("时间线-较新", "2026-04-01 00:00:00"),
        ("时间线-最新", "2026-05-01 00:00:00"),
    ] {
        seed_artwork(&app.state, title, "approved", created_at).await;
    }
    sqlx::query("UPDATE artworks SET uploader_uid='timeline-author' WHERE title LIKE '时间线-%'")
        .execute(&app.state.pools.art)
        .await
        .unwrap();
    let current_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='时间线-当前'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();

    let (status, body) = send(
        &app.router,
        get(
            &format!("/api/art/artworks/{current_id}/creator-timeline"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["currentIndex"], 2);
    assert_eq!(body["total"], 5);
    let titles: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["title"].as_str())
        .collect();
    assert_eq!(
        titles,
        vec![
            "时间线-最新",
            "时间线-较新",
            "时间线-当前",
            "时间线-较旧",
            "时间线-最旧"
        ]
    );
}

#[tokio::test]
async fn guild_profile_artworks_are_paginated_ten_per_page() {
    let app = setup().await;
    for index in 1..=23 {
        let title = format!("分页作品-{index:02}");
        let created_at = format!("2026-01-{index:02} 00:00:00");
        seed_artwork(&app.state, &title, "approved", &created_at).await;
    }
    seed_artwork(
        &app.state,
        "分页作品-未通过",
        "pending",
        "2026-01-24 00:00:00",
    )
    .await;
    sqlx::query("UPDATE artworks SET uploader_uid='paged-author' WHERE title LIKE '分页作品-%'")
        .execute(&app.state.pools.art)
        .await
        .unwrap();

    let (status, body) = send(
        &app.router,
        get(
            "/api/art/guild/profile/paged-author/artworks?page=2&pageSize=10",
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 23);
    assert_eq!(body["page"], 2);
    assert_eq!(body["pageSize"], 10);
    assert_eq!(body["pageCount"], 3);
    assert_eq!(body["hasMore"], true);
    let items = body["data"].as_array().unwrap();
    assert_eq!(items.len(), 10);
    assert_eq!(items.first().unwrap()["title"], "分页作品-13");
    assert_eq!(items.last().unwrap()["title"], "分页作品-04");
}

#[tokio::test]
async fn art_latest_list_uses_review_time_instead_of_upload_time() {
    let app = setup().await;
    seed_artwork(
        &app.state,
        "uploaded-later",
        "approved",
        "2024-05-10 00:00:00",
    )
    .await;
    seed_artwork(
        &app.state,
        "approved-later",
        "approved",
        "2024-05-01 00:00:00",
    )
    .await;
    sqlx::query(
        "UPDATE artworks SET reviewed_at='2024-05-11 00:00:00' WHERE title='approved-later'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET reviewed_at='2024-05-10 00:00:00' WHERE title='uploaded-later'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        get("/api/art/artworks?sort=time&pageSize=6", None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let titles: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|artwork| artwork["title"].as_str())
        .collect();
    assert_eq!(titles, ["approved-later", "uploaded-later"]);
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
async fn art_upload_rejects_unsafe_origin_url_without_persisting() {
    let app = setup().await;
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let req = multipart_req(
        "/api/art/artworks",
        &[
            ("images", Some("ok.png"), "\u{89}PNG-fake-bytes"),
            ("title", None, "危险出处测试"),
            ("origin_url", None, "javascript:alert(document.domain)"),
        ],
        Some(&token),
    );
    let (status, body) = send(&app.router, req).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "危险协议应被拒绝: {body:?}"
    );
    let stored: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM artworks WHERE title='危险出处测试'")
            .fetch_one(&app.state.pools.art)
            .await
            .unwrap();
    assert_eq!(stored, 0);
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
async fn admin_reject_rating_requires_reason_and_user_can_see_it() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let uid = format!("u{user_id}");
    let ts = "2026-07-01T00:00:00Z";

    sqlx::query(
        "INSERT OR REPLACE INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at) \
         VALUES(?, 100, 'F', 'observer_clearance', ?, ?)",
    )
    .bind(&uid)
    .bind(ts)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();
    let app_id: i64 = sqlx::query_scalar(
        "INSERT INTO guild_rating_applications(uid, from_rating, target_rating,
         reputation_snapshot, haruhi_count_snapshot, status, user_note, created_at)
         VALUES(?, 'F', 'E', 100, 1, 'pending', 'auto apply', ?) RETURNING id",
    )
    .bind(&uid)
    .bind(ts)
    .fetch_one(&art)
    .await
    .unwrap();

    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/rating-applications/{app_id}/reject"),
            json!({ "note": "  " }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let reason = "Need more complete Haruhi personal artworks";
    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/rating-applications/{app_id}/reject"),
            json!({ "note": reason }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reject should pass: {body:?}");

    let (status, body) = send(&app.router, get("/api/art/guild/quests", Some(&token))).await;
    assert_eq!(status, StatusCode::OK);
    let applications = body["ratingApplications"].as_array().unwrap();
    let item = applications
        .iter()
        .find(|item| item["id"].as_i64() == Some(app_id))
        .expect("rejected rating application should be visible to user");
    assert_eq!(item["status"], "rejected");
    assert_eq!(item["adminNote"], reason);
    assert_eq!(item["targetRating"], "E");
    assert!(item["reviewedAt"].as_str().is_some());
}

#[tokio::test]
async fn admin_points_penalty_reduces_earned_and_cancels_pending_redemptions() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let uid = "u_penalty_case";
    let ts = "2026-07-01T00:00:00Z";

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
    for (points, source_type) in [(100_i64, "upload"), (3_i64, "manual")] {
        sqlx::query(
            "INSERT INTO points_ledger(uid, artwork_id, points, note, source_type, created_at, granted_at) \
             VALUES(?, NULL, ?, 'earned before penalty', ?, ?, ?)",
        )
        .bind(uid)
        .bind(points)
        .bind(source_type)
        .bind(ts)
        .bind(ts)
        .execute(&art)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO guild_reward_redemptions(reward_id, uid, frozen_coins, status, user_note, created_at) \
         VALUES(1, ?, 30, 'pending', 'pending redemption', ?)",
    )
    .bind(uid)
    .bind(ts)
    .execute(&art)
    .await
    .unwrap();

    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/admin/points/penalize",
            json!({ "uid": uid, "divisor": 3, "note": "bad divisor" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/admin/points/penalize",
            json!({ "uid": uid, "divisor": 5, "note": "" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/admin/points/penalize",
            json!({ "uid": "u_missing_penalty", "divisor": 5, "note": "missing uid" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let reason = "speed sketch farming";
    let (status, body) = send(
        &app.router,
        post_json(
            "/api/art/admin/points/penalize",
            json!({ "uid": uid, "divisor": 5, "note": reason }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "penalty should pass: {body:?}");
    assert_eq!(body["previousTotal"], 103);
    assert_eq!(body["targetTotal"], 20);
    assert_eq!(body["deductedPoints"], 83);
    assert_eq!(body["cancelledRedemptions"], 1);

    let total: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE uid=?")
            .bind(uid)
            .fetch_one(&art)
            .await
            .unwrap();
    assert_eq!(total, 20);
    let penalty_points: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(points), 0) FROM points_ledger WHERE uid=? AND source_type='penalty'",
    )
    .bind(uid)
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(penalty_points, -83);
    let (redemption_status, admin_note): (String, Option<String>) = sqlx::query_as(
        "SELECT status, admin_note FROM guild_reward_redemptions WHERE uid=? ORDER BY id DESC LIMIT 1",
    )
    .bind(uid)
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(redemption_status, "cancelled");
    assert!(admin_note.unwrap_or_default().contains(reason));

    let (status, body) = send(&app.router, get("/api/art/guild/leaderboard", None)).await;
    assert_eq!(status, StatusCode::OK);
    let entry = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["uid"] == uid)
        .expect("penalized user should remain on leaderboard with positive earned coins");
    assert_eq!(entry["earned"], 20);
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
async fn guild_access_application_requires_haruhi_art_and_admin_review() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let admin = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    insert_active_user(&app.state, "alice", "alice-pass").await;
    let alice = login(&app.router, "alice", "alice-pass").await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username='alice'")
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let uid = format!("u{user_id}");

    let eligible: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('凉宫个人作品', ?, 'personal', 'haruhi', 'approved', '2026-07-01T10:00:00.000Z', '2026-07-01T11:00:00.000Z')
         RETURNING id",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    let other_content: i64 = sqlx::query_scalar(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('其他个人作品', ?, 'personal', 'other', 'approved', '2026-07-01T12:00:00.000Z', '2026-07-01T12:30:00.000Z')
         RETURNING id",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('待审凉宫作品', ?, 'personal', 'haruhi', 'pending', '2026-07-01T13:00:00.000Z', NULL)",
    )
    .bind(&uid)
    .execute(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO artworks(title, uploader_uid, source_type, content_type, status, created_at, reviewed_at)
         VALUES('他人凉宫作品', 'u_other', 'personal', 'haruhi', 'approved', '2026-07-01T14:00:00.000Z', '2026-07-01T14:30:00.000Z')",
    )
    .execute(&art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        get("/api/art/guild/access/submission-artworks", Some(&alice)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let artwork_ids: Vec<i64> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert_eq!(artwork_ids, vec![eligible]);

    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/access/apply",
            json!({ "artworkIds": [] }),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/access/apply",
            json!({ "artworkIds": [other_content] }),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = send(
        &app.router,
        post_json(
            "/api/art/guild/access/apply",
            json!({ "artworkIds": [eligible], "note": "" }),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "合格作品应能申请访问许可: {body:?}");
    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/access/apply",
            json!({ "artworkIds": [eligible] }),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "已有 pending 时不能重复申请"
    );

    let (status, body) = send(
        &app.router,
        get("/api/art/admin/guild/access-applications", Some(&admin)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let app_id = body["data"][0]["id"].as_i64().unwrap();
    assert_eq!(body["data"][0]["targetAccess"], "anomaly_research");
    assert_eq!(
        body["data"][0]["submittedArtworks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/access-applications/{app_id}/reject"),
            json!({ "note": "" }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "驳回理由必填");

    let (status, body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/access-applications/{app_id}/approve"),
            json!({ "note": "通过" }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "管理员应可批准权限申请: {body:?}");
    let access: String = sqlx::query_scalar("SELECT access_tier FROM guild_profiles WHERE uid=?")
        .bind(&uid)
        .fetch_one(&art)
        .await
        .unwrap();
    assert_eq!(access, "anomaly_research");

    let (status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/access/apply",
            json!({ "artworkIds": [eligible] }),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "批准后可继续申请下一档");
    let second_id: i64 = sqlx::query_scalar(
        "SELECT id FROM guild_access_applications WHERE uid=? AND status='pending'",
    )
    .bind(&uid)
    .fetch_one(&art)
    .await
    .unwrap();
    let (status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/access-applications/{second_id}/reject"),
            json!({ "note": "请补充更完整的凉宫作品" }),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access: String = sqlx::query_scalar("SELECT access_tier FROM guild_profiles WHERE uid=?")
        .bind(&uid)
        .fetch_one(&art)
        .await
        .unwrap();
    assert_eq!(access, "anomaly_research", "驳回不应改变当前许可");

    let (status, body) = send(&app.router, get("/api/art/guild/quests", Some(&alice))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["profile"]["accessTier"], "anomaly_research");
    assert!(
        body["accessApplications"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["status"] == "rejected"
                && item["adminNote"] == "请补充更完整的凉宫作品"),
        "用户侧应能看到权限申请驳回理由"
    );
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

#[tokio::test]
async fn creator_profile_messages_are_public_but_posting_requires_login() {
    let app = setup().await;
    let art = app.state.pools.art.clone();
    let core = app.state.pools.core.clone();
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let author_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&core)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE users SET nickname='留言测试员', avatar='/uploads/avatars/test.webp' WHERE id=?",
    )
    .bind(author_id)
    .execute(&core)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO creators(uid, avatar_url, created_at) VALUES('creator-message-test', '', '2026-01-01T00:00:00Z')",
    )
    .execute(&art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO creator_profile_messages
         (creator_uid, author_user_id, user_name, body, created_at, status)
         VALUES('creator-message-test', ?, '旧署名', '公开档案留言', '2026-01-01T00:00:00Z', 'public')",
    )
    .bind(author_id)
    .execute(&art)
    .await
    .unwrap();

    let (status, body) = send(
        &app.router,
        get(
            "/api/art/guild/profile/creator-message-test/messages?page=1&pageSize=16",
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["data"][0]["user_name"], "留言测试员");
    assert_eq!(body["data"][0]["avatar_url"], "/uploads/avatars/test.webp");
    assert_eq!(body["data"][0]["body"], "公开档案留言");

    let (anonymous_status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/profile/creator-message-test/messages",
            json!({ "body": "匿名留言" }),
            None,
        ),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);

    let (member_status, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/profile/creator-message-test/messages",
            json!({ "body": "登录成员留言" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(member_status, StatusCode::OK);
    let stored: (i64, String, String, i64) = sqlx::query_as(
        "SELECT author_user_id, user_name, status, id
         FROM creator_profile_messages WHERE body='登录成员留言'",
    )
    .fetch_one(&art)
    .await
    .unwrap();
    assert_eq!(stored.0, author_id);
    assert_eq!(stored.1, "留言测试员");
    assert_eq!(stored.2, "flagged", "AI 离线时留言应进入人工审核队列");

    let (admin_list_status, admin_list) = send(
        &app.router,
        get(
            "/api/art/admin/guild/profile-messages?status=flagged",
            Some(&token),
        ),
    )
    .await;
    assert_eq!(admin_list_status, StatusCode::OK);
    assert!(admin_list["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|message| message["id"] == stored.3));

    let (approve_status, approve_body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/profile-messages/{}/status", stored.3),
            json!({ "status": "public" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(
        approve_status,
        StatusCode::OK,
        "审核通过应成功: {approve_body:?}"
    );

    let (_, public_after_approve) = send(
        &app.router,
        get(
            "/api/art/guild/profile/creator-message-test/messages?page=1&pageSize=16",
            None,
        ),
    )
    .await;
    assert!(public_after_approve["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|message| message["id"] == stored.3));

    let (hide_status, hide_body) = send(
        &app.router,
        post_json(
            &format!("/api/art/admin/guild/profile-messages/{}/status", stored.3),
            json!({ "status": "hidden" }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(hide_status, StatusCode::OK, "隐藏留言应成功: {hide_body:?}");
}

#[tokio::test]
async fn art_public_guild_profile_includes_unified_account_bio() {
    let app = setup().await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET bio=? WHERE id=?")
        .bind("在闭锁空间之外画画。")
        .bind(user_id)
        .execute(&app.state.pools.core)
        .await
        .unwrap();

    let (status, body) = send(
        &app.router,
        get(&format!("/api/art/guild/profile/u{user_id}"), None),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["profile"]["bio"], "在闭锁空间之外画画。");
    assert!(body["profile"]["email"].is_null());
}

#[tokio::test]
async fn art_follow_and_favorite_state_is_persistent_and_visible_on_profiles() {
    let app = setup().await;
    let token = login(&app.router, ADMIN_USER, ADMIN_PASS).await;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username=?")
        .bind(ADMIN_USER)
        .fetch_one(&app.state.pools.core)
        .await
        .unwrap();
    let own_uid = format!("u{user_id}");

    sqlx::query(
        "INSERT INTO creators(uid, avatar_url, created_at) VALUES('social-target', '', '2026-01-01T00:00:00Z')",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO guild_profiles(uid, reputation, rating, access_tier, created_at, updated_at)
         VALUES('social-target', 0, 'F', 'public_archive', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();

    let (anonymous_follow, _) = send(
        &app.router,
        post_json(
            "/api/art/guild/profile/social-target/follow",
            json!({}),
            None,
        ),
    )
    .await;
    assert_eq!(anonymous_follow, StatusCode::UNAUTHORIZED);

    let (follow_status, follow) = send(
        &app.router,
        post_json(
            "/api/art/guild/profile/social-target/follow",
            json!({}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(follow_status, StatusCode::OK);
    assert_eq!(follow["following"], true);
    assert_eq!(follow["followerCount"], 1);

    let (profile_status, profile) = send(
        &app.router,
        get("/api/art/guild/profile/social-target", Some(&token)),
    )
    .await;
    assert_eq!(profile_status, StatusCode::OK);
    assert_eq!(profile["social"]["isFollowing"], true);
    assert_eq!(profile["social"]["followerCount"], 1);
    assert_eq!(profile["social"]["followers"][0]["uid"], own_uid);

    for index in 0..7 {
        sqlx::query(
            "INSERT INTO user_follows(follower_uid, followed_uid, created_at) VALUES(?,?,?)",
        )
        .bind(format!("list-follower-{index}"))
        .bind("social-target")
        .bind(format!("2026-02-{:02}T00:00:00Z", index + 1))
        .execute(&app.state.pools.art)
        .await
        .unwrap();
    }
    let (connections_status, connections) = send(
        &app.router,
        get(
            "/api/art/guild/profile/social-target/connections?kind=followers&page=2&pageSize=6",
            None,
        ),
    )
    .await;
    assert_eq!(connections_status, StatusCode::OK);
    assert_eq!(connections["kind"], "followers");
    assert_eq!(connections["total"], 8);
    assert_eq!(connections["page"], 2);
    assert_eq!(connections["data"].as_array().unwrap().len(), 2);

    let (self_follow_status, _) = send(
        &app.router,
        post_json(
            &format!("/api/art/guild/profile/{own_uid}/follow"),
            json!({}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(self_follow_status, StatusCode::BAD_REQUEST);

    seed_artwork(
        &app.state,
        "社交收藏测试作品",
        "approved",
        "2026-07-01 00:00:00",
    )
    .await;
    let artwork_id: i64 =
        sqlx::query_scalar("SELECT id FROM artworks WHERE title='社交收藏测试作品'")
            .fetch_one(&app.state.pools.art)
            .await
            .unwrap();

    let (favorite_status, favorite) = send(
        &app.router,
        post_json(
            &format!("/api/art/artworks/{artwork_id}/favorite"),
            json!({}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(favorite_status, StatusCode::OK);
    assert_eq!(favorite["favorited"], true);
    assert_eq!(favorite["favoriteCount"], 1);

    let (_, detail) = send(
        &app.router,
        get(&format!("/api/art/artworks/{artwork_id}"), Some(&token)),
    )
    .await;
    assert_eq!(detail["data"]["favorited"], true);
    assert_eq!(detail["data"]["favorite_count"], 1);

    let (favorites_status, favorites) = send(
        &app.router,
        get(&format!("/api/art/guild/profile/{own_uid}/favorites"), None),
    )
    .await;
    assert_eq!(favorites_status, StatusCode::OK);
    assert_eq!(favorites["total"], 1);
    assert_eq!(favorites["data"][0]["id"], artwork_id);
}

#[tokio::test]
async fn artwork_related_uses_text_affinity_when_tags_are_empty() {
    let app = setup().await;
    for title in [
        "星空列车的约定",
        "银河列车夜行",
        "泳池边的夏日",
        "夏日泳池假期",
        "无关高热作品",
        "普通校园作品",
    ] {
        seed_artwork(&app.state, title, "approved", "2026-07-01 00:00:00").await;
    }
    sqlx::query(
        "UPDATE artworks SET description='银河 星空 夜车', tags_json='[]' WHERE title='星空列车的约定'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET description='银河 星空 夜行', tags_json='[]' WHERE title='银河列车夜行'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET description='游泳 水花 盛夏', tags_json='[]' WHERE title='泳池边的夏日'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE artworks SET description='游泳 水花 夏天', tags_json='[]' WHERE title='夏日泳池假期'",
    )
    .execute(&app.state.pools.art)
    .await
    .unwrap();
    sqlx::query("UPDATE artworks SET like_total=999 WHERE title='无关高热作品'")
        .execute(&app.state.pools.art)
        .await
        .unwrap();

    let train_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='星空列车的约定'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    let pool_id: i64 = sqlx::query_scalar("SELECT id FROM artworks WHERE title='泳池边的夏日'")
        .fetch_one(&app.state.pools.art)
        .await
        .unwrap();
    let (_, train_related) = send(
        &app.router,
        get(
            &format!("/api/art/artworks/{train_id}/related?limit=4"),
            None,
        ),
    )
    .await;
    let (_, pool_related) = send(
        &app.router,
        get(
            &format!("/api/art/artworks/{pool_id}/related?limit=4"),
            None,
        ),
    )
    .await;

    assert_eq!(train_related["data"][0]["title"], "银河列车夜行");
    assert_eq!(pool_related["data"][0]["title"], "夏日泳池假期");
    assert_ne!(
        train_related["data"][0]["id"], pool_related["data"][0]["id"],
        "不同作品不应退化成同一套人气榜"
    );
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
