//! exam 模块：考试平台（试卷创建/导入/编辑、AI 异步审核、上传转码、后台管理）。
//! 忠实移植旧 haruhi-exam-platform 后端（server/index.js + ai.js + db.js），统一挂载于 /api/exam。
//!
//! 与旧实现的差异（详见交付报告）：
//! - 旧路径在根 /api/* 下（如 /api/exams），这里统一前缀 /api/exam/*（router nest 在 /exam）。
//! - 后台管理由旧的 x-admin-key 头比对 ADMIN_KEY，改为统一 JWT + RBAC（authorize/AuthUser，app="exam"）。
//!   /admin/stats、/admin/list → Action::Read；/admin/exams/:id/status → Action::Moderate；
//!   DELETE /admin/exams/:id → Action::Manage。
//! - 上传：旧用 multer 存 server/uploads 并以 `/exam/api/uploads/<file>` 形式返回；
//!   这里存 uploads/exam/，库/响应路径统一为 `/uploads/exam/<file>`（按本项目约定）。
//! - AI 审核：旧 checkContent(examData) 拼标题+题目样本送审，无 key 放行。
//!   这里复用 haruhi_ai::check_text(EXAM_SYSTEM_PROMPT, ...)，仍异步（tokio::spawn）跑完更新状态。
//! - edit_token：业务令牌（与 RBAC 正交），创建/导入返回，PUT/verify 校验。

use std::path::Path as FsPath;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Value};

use crate::state::AppState;

// ============================================================
// 路由装配
// ============================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // ---- 上传 / 清理 ----
        .route("/upload", post(upload))
        .route("/cleanup", post(cleanup))
        // ---- 试卷（公开 + edit_token）----
        .route("/exams", get(list_exams).post(create_exam))
        // import 必须在 /exams/{id} 之前注册（与旧注释一致；axum 静态段优先，仍保留次序）
        .route("/exams/import", post(import_exam))
        .route("/exams/{id}", get(get_exam).put(update_exam))
        .route("/exams/{id}/verify", post(verify_exam))
        // ---- 后台（RBAC）----
        .route("/admin/stats", get(admin_stats))
        .route("/admin/list", get(admin_list))
        .route("/admin/exams/{id}/status", post(admin_set_status))
        .route("/admin/exams/{id}", delete(admin_delete_exam))
}

// ============================================================
// 工具
// ============================================================

/// 上传/响应中的 uploads 路径前缀（库内与返回统一）。
const UPLOAD_URL_PREFIX: &str = "/uploads/exam";

/// 解析存库 JSON 字符串；失败回退给定默认（对齐旧 JSON.parse(x || '{}')/'[]')。
fn parse_json_or(s: Option<&str>, default: Value) -> Value {
    match s {
        Some(text) if !text.trim().is_empty() => {
            serde_json::from_str::<Value>(text).unwrap_or(default)
        }
        _ => default,
    }
}

/// fixUrlPath：把 JSON 序列化后将 "/exam/exam/" 替换为 "/exam/"（忠实旧实现）。
fn fix_url_path(v: Value) -> Value {
    let s = serde_json::to_string(&v).unwrap_or_default();
    let fixed = s.replace("/exam/exam/", "/exam/");
    serde_json::from_str(&fixed).unwrap_or(v)
}

/// 模拟 JS parseInt(s,10) || default：取十进制前缀，无效或 <=0 回退默认（对齐旧 parseInt(...)||N）。
fn parse_int_or(s: Option<&str>, default: i64) -> i64 {
    let n = match s {
        Some(s) => parse_int_radix10(s),
        None => None,
    };
    match n {
        Some(v) if v != 0 => v, // JS 中 0 为 falsy → 取默认
        _ => default,
    }
}

fn parse_int_radix10(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut i = 0;
    let mut sign = 1_i64;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    t[start..i].parse::<i64>().ok().map(|n| sign * n)
}

/// 从试卷 questions/levels JSON 中抽取引用的文件路径（对齐 extractFilePaths）。
fn extract_file_paths(questions_json: &str, levels_json: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    let questions: Value = serde_json::from_str(questions_json).unwrap_or(json!([]));
    let levels: Value = serde_json::from_str(levels_json).unwrap_or(json!([]));

    if let Some(arr) = levels.as_array() {
        for lv in arr {
            if let Some(s) = lv.get("img").and_then(|v| v.as_str()) {
                paths.push(s.to_string());
            }
            if let Some(s) = lv.get("sketch").and_then(|v| v.as_str()) {
                paths.push(s.to_string());
            }
        }
    }

    let collect_blocks = |blocks: Option<&Vec<Value>>, out: &mut Vec<String>| {
        if let Some(blocks) = blocks {
            for b in blocks {
                let ty = b.get("type").and_then(|v| v.as_str());
                if ty == Some("image") {
                    if let Some(src) = b
                        .get("image")
                        .and_then(|i| i.get("src"))
                        .and_then(|v| v.as_str())
                    {
                        out.push(src.to_string());
                    }
                }
                if ty == Some("audio") {
                    if let Some(src) = b
                        .get("audio")
                        .and_then(|i| i.get("src"))
                        .and_then(|v| v.as_str())
                    {
                        out.push(src.to_string());
                    }
                }
            }
        }
    };

    if let Some(arr) = questions.as_array() {
        for q in arr {
            collect_blocks(q.get("stemBlocks").and_then(|v| v.as_array()), &mut paths);
            collect_blocks(
                q.get("analysisBlocks").and_then(|v| v.as_array()),
                &mut paths,
            );
        }
    }
    paths
}

/// 拼接送审文本（对齐 ai.js textToAudit：标题/卷头/副标题 + 前 8 题样本，截断 3000）。
fn build_audit_text(config: &Value, questions: &Value) -> String {
    let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let paper_title = config
        .get("paperTitle")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let paper_subtitle = config
        .get("paperSubtitle")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut samples: Vec<String> = Vec::new();
    if let Some(qs) = questions.as_array() {
        for q in qs.iter().take(8) {
            let mut line = String::new();
            if let Some(blocks) = q.get("stemBlocks").and_then(|v| v.as_array()) {
                for b in blocks {
                    line.push_str(b.get("text").and_then(|v| v.as_str()).unwrap_or(""));
                }
            }
            if let Some(opts) = q.get("options").and_then(|v| v.as_array()) {
                let texts: Vec<&str> = opts
                    .iter()
                    .map(|o| o.get("text").and_then(|v| v.as_str()).unwrap_or(""))
                    .collect();
                line.push_str(&texts.join(" "));
            }
            samples.push(line);
        }
    }

    let text = format!(
        "试卷标题: {title}\n卷头标题: {paper_title}\n副标题: {paper_subtitle}\n题目样本: {}",
        samples.join("\n")
    );
    text.chars().take(3000).collect()
}

/// 异步审核：跑完后把 status 更新为 published / locked + ai_reason（对齐旧 checkContent().then()）。
fn spawn_audit(state: &AppState, id: String, config: Value, questions: Value) {
    let pool = state.pools.exam.clone();
    let ai = haruhi_ai::AiClient::from_config(&state.cfg);
    let audit_text = build_audit_text(&config, &questions);
    tokio::spawn(async move {
        let verdict = ai
            .check_text(haruhi_ai::EXAM_SYSTEM_PROMPT, &audit_text)
            .await;
        let new_status = if verdict.ok { "published" } else { "locked" };
        if let Err(e) = sqlx::query("UPDATE exams SET status = ?, ai_reason = ? WHERE id = ?")
            .bind(new_status)
            .bind(&verdict.reason)
            .bind(&id)
            .execute(&pool)
            .await
        {
            tracing::error!("exam 审核状态更新失败 ({id}): {e}");
        }
    });
}

// ============================================================
// 试卷数据校验（对齐 validateExamData）
// ============================================================

fn validate_exam_data(data: &Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if data.is_null() {
        return vec!["数据为空".to_string()];
    }

    match data.get("config") {
        Some(cfg) if cfg.is_object() => {
            let title_ok = cfg.get("title").map(|v| v.is_string()).unwrap_or(false);
            if !title_ok {
                errors.push("config.title 是必填字段且必须是字符串".to_string());
            }
        }
        _ => errors.push("缺少 config 字段或格式错误".to_string()),
    }

    match data.get("questions").and_then(|v| v.as_array()) {
        Some(qs) => {
            for (idx, q) in qs.iter().enumerate() {
                let id_ok = q.get("id").map(|v| v.is_string()).unwrap_or(false);
                if !id_ok {
                    errors.push(format!("questions[{idx}].id 是必填字段且必须是字符串"));
                }
                let ty = q.get("type").and_then(|v| v.as_str());
                let ty_ok = matches!(ty, Some("choice" | "fill" | "judgment" | "multiple"));
                if !ty_ok {
                    errors.push(format!(
                        "questions[{idx}].type 必须是 choice/fill/judgment/multiple 之一"
                    ));
                }
                if !q.get("stemBlocks").map(|v| v.is_array()).unwrap_or(false) {
                    errors.push(format!("questions[{idx}].stemBlocks 必须是数组"));
                }
                let score_ok = q
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .map(|s| s >= 0.0)
                    .unwrap_or(false);
                if !score_ok {
                    errors.push(format!("questions[{idx}].score 必须是大于等于0的数字"));
                }
            }
        }
        None => errors.push("questions 必须是数组".to_string()),
    }

    match data.get("levels").and_then(|v| v.as_array()) {
        Some(ls) => {
            for (idx, l) in ls.iter().enumerate() {
                let min = l.get("min").and_then(|v| v.as_f64());
                let max = l.get("max").and_then(|v| v.as_f64());
                match (min, max) {
                    (Some(min), Some(max)) => {
                        if min > max {
                            errors.push(format!("levels[{idx}].min 不能大于 max"));
                        }
                    }
                    _ => errors.push(format!("levels[{idx}].min 和 max 必须是数字")),
                }
            }
        }
        None => errors.push("levels 必须是数组".to_string()),
    }

    errors
}

// ============================================================
// 上传 / 清理
// ============================================================

// POST /upload（公开，对齐旧 /api/upload）
async fn upload(State(state): State<AppState>, mut mp: Multipart) -> AppResult<Response> {
    // 读取 file 字段
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_name = String::from("file");
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| haruhi_core::AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        if field.name() == Some("file") {
            if let Some(n) = field.file_name() {
                original_name = n.to_string();
            }
            let bytes = field
                .bytes()
                .await
                .map_err(|e| haruhi_core::AppError::bad_request(format!("读取文件失败: {e}")))?;
            file_bytes = Some(bytes.to_vec());
        } else {
            let _ = field.bytes().await;
        }
    }
    let file_bytes = match file_bytes {
        Some(b) => b,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No file uploaded" })),
            )
                .into_response())
        }
    };

    let ext = haruhi_media::ext_of(&original_name, "").to_lowercase();
    let uuid = uuid::Uuid::new_v4().to_string();
    let exam_dir = state.cfg.uploads_subdir("exam");

    let image_exts = ["jpg", "jpeg", "png", "webp", "gif", "svg", "bmp"];
    let audio_exts = ["mp3", "wav", "ogg", "m4a", "aac", "flac", "wma", "amr"];

    // 图片：转 WebP（复用 encode_webp）；不可解码的（svg/bmp/gif 等）降级原样落盘。
    if image_exts.contains(&ext.as_str()) {
        let data = file_bytes.clone();
        let webp = tokio::task::spawn_blocking(move || haruhi_media::encode_webp(&data, 80.0))
            .await
            .ok()
            .and_then(|r| r.ok());
        if let Some(bytes) = webp {
            let fname = format!("{uuid}.webp");
            haruhi_media::save_file(&exam_dir, &fname, &bytes).await?;
            return Ok(Json(json!({
                "url": format!("{UPLOAD_URL_PREFIX}/{fname}"),
                "originalName": original_name,
            }))
            .into_response());
        }
        // 降级：原始格式落盘
        let fname = if ext.is_empty() {
            uuid.clone()
        } else {
            format!("{uuid}.{ext}")
        };
        haruhi_media::save_file(&exam_dir, &fname, &file_bytes).await?;
        return Ok(Json(json!({
            "url": format!("{UPLOAD_URL_PREFIX}/{fname}"),
            "originalName": original_name,
        }))
        .into_response());
    }

    // 音频：ffmpeg 转 MP3(192k)，失败降级保留原文件（忠实旧逻辑）。
    if audio_exts.contains(&ext.as_str()) {
        let src_name = if ext.is_empty() {
            uuid.clone()
        } else {
            format!("{uuid}.{ext}")
        };
        // 先把原始文件落盘（作为 ffmpeg 输入，也是降级保留对象）
        haruhi_media::save_file(&exam_dir, &src_name, &file_bytes).await?;
        let src_path = exam_dir.join(&src_name);
        let mp3_name = format!("{uuid}.mp3");
        let mp3_path = exam_dir.join(&mp3_name);

        match haruhi_media::audio::transcode_to_mp3(&src_path, &mp3_path).await {
            Ok(()) => {
                // 转码成功，删除原始文件（对齐旧 unlink(file.path)）
                let _ = tokio::fs::remove_file(&src_path).await;
                return Ok(Json(json!({
                    "url": format!("{UPLOAD_URL_PREFIX}/{mp3_name}"),
                    "originalName": original_name,
                }))
                .into_response());
            }
            Err(e) => {
                tracing::warn!("音频转码失败，降级保留原文件: {e}");
                // 清理可能产生的半成品 mp3
                let _ = tokio::fs::remove_file(&mp3_path).await;
                return Ok(Json(json!({
                    "url": format!("{UPLOAD_URL_PREFIX}/{src_name}"),
                    "originalName": original_name,
                    "fallback": true,
                }))
                .into_response());
            }
        }
    }

    // 其他类型：原样落盘，仅返回 url（对齐旧 res.json({url}))
    let fname = if ext.is_empty() {
        uuid.clone()
    } else {
        format!("{uuid}.{ext}")
    };
    haruhi_media::save_file(&exam_dir, &fname, &file_bytes).await?;
    Ok(Json(json!({ "url": format!("{UPLOAD_URL_PREFIX}/{fname}") })).into_response())
}

// POST /cleanup（公开，对齐旧 /api/cleanup）：删除未引用文件
async fn cleanup(State(state): State<AppState>, Json(body): Json<Value>) -> AppResult<Response> {
    let files = match body.get("files").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid body" })),
            )
                .into_response())
        }
    };

    let exam_dir = state.cfg.uploads_subdir("exam");
    let mut deleted = 0_i64;
    for f in files {
        let url = match f.as_str() {
            Some(s) => s,
            None => continue,
        };
        if !url.contains("/uploads/") {
            continue;
        }
        // 取 basename（对齐 path.basename）
        if let Some(name) = basename(url) {
            let p = exam_dir.join(&name);
            match tokio::fs::remove_file(&p).await {
                Ok(()) => deleted += 1,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => tracing::error!("cleanup 删除失败 {name}: {e}"),
            }
        }
    }
    Ok(Json(json!({ "success": true, "deleted": deleted })).into_response())
}

/// 取 URL/路径的文件名部分（对齐 path.basename）。
fn basename(url: &str) -> Option<String> {
    FsPath::new(url)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

// ============================================================
// 试卷
// ============================================================

#[derive(sqlx::FromRow)]
struct ExamRow {
    id: String,
    title: Option<String>,
    subtitle: Option<String>,
    config: Option<String>,
    questions: Option<String>,
    levels: Option<String>,
    status: Option<String>,
    edit_token: Option<String>,
    // 旧数据可能把数值列存成 TEXT，读成 String 容错（避免 sqlx 解码 500）
    visit_count: Option<i64>,
    ai_reason: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

/// 列表项（对齐 list SQL：id,title,subtitle,config,questions,visit_count,created_at）。
#[derive(sqlx::FromRow)]
struct ExamListRow {
    id: String,
    title: Option<String>,
    subtitle: Option<String>,
    config: Option<String>,
    questions: Option<String>,
    visit_count: Option<i64>,
    created_at: Option<String>,
}

// GET /exams（公开，列表分页+搜索；自增总访问量）
async fn list_exams(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Value>> {
    // 总访问量 +1（对齐旧 UPDATE site_stats）
    let _ = sqlx::query("UPDATE site_stats SET value = value + 1 WHERE key = 'total_visits'")
        .execute(&state.pools.exam)
        .await;

    let page = parse_int_or(q.get("page").map(|s| s.as_str()), 1);
    let limit = parse_int_or(q.get("limit").map(|s| s.as_str()), 9);
    let search = q
        .get("search")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // WHERE
    let mut where_clause = String::from("WHERE status = 'published'");
    let mut params: Vec<String> = Vec::new();
    if !search.is_empty() {
        where_clause.push_str(" AND (title LIKE ? OR subtitle LIKE ?)");
        let term = format!("%{search}%");
        params.push(term.clone());
        params.push(term);
    }

    // 总数
    let count_sql = format!("SELECT COUNT(*) FROM exams {where_clause}");
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        count_q = count_q.bind(p);
    }
    let total: i64 = count_q.fetch_one(&state.pools.exam).await?;

    // 分页逻辑（忠实旧实现：非搜索模式第一页为官方试卷预留 1 位）
    let (actual_limit, offset, total_pages) = if !search.is_empty() {
        let tp = ((total as f64) / (limit as f64)).ceil() as i64;
        (limit, (page - 1) * limit, tp.max(1))
    } else {
        let (al, off) = if page == 1 {
            (limit - 1, 0)
        } else {
            (limit, (limit - 1) + (page - 2) * limit)
        };
        let tp = if total <= (limit - 1) {
            1
        } else {
            1 + (((total - (limit - 1)) as f64) / (limit as f64)).ceil() as i64
        };
        (al, off, tp.max(1))
    };

    // 查询分页数据
    let list_sql = format!(
        "SELECT id, title, subtitle, config, questions, CAST(visit_count AS INTEGER) AS visit_count, created_at \
         FROM exams {where_clause} ORDER BY visit_count DESC, created_at DESC LIMIT ? OFFSET ?"
    );
    let mut list_q = sqlx::query_as::<_, ExamListRow>(&list_sql);
    for p in &params {
        list_q = list_q.bind(p);
    }
    list_q = list_q.bind(actual_limit.max(0)).bind(offset.max(0));
    let rows: Vec<ExamListRow> = list_q.fetch_all(&state.pools.exam).await?;

    let exams: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "title": r.title,
                "subtitle": r.subtitle,
                "config": parse_json_or(r.config.as_deref(), json!({})),
                "questions": parse_json_or(r.questions.as_deref(), json!([])),
                "visit_count": r.visit_count.unwrap_or(0),
                "created_at": r.created_at,
            })
        })
        .collect();

    Ok(Json(json!({
        "data": fix_url_path(Value::Array(exams)),
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": total_pages,
        }
    })))
}

// POST /exams（公开，创建 → pending + 异步审核，返回 edit_token）
async fn create_exam(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let config = body.get("config").cloned().unwrap_or(json!({}));
    let questions = body.get("questions").cloned().unwrap_or(json!([]));
    let levels = body.get("levels").cloned().unwrap_or(json!([]));

    let id = uuid::Uuid::new_v4().to_string();
    let edit_token = uuid::Uuid::new_v4().to_string();
    let title = config
        .get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("未命名试卷")
        .to_string();
    let subtitle = config
        .get("paperSubtitle")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    sqlx::query(
        "INSERT INTO exams (id, title, subtitle, config, questions, levels, status, edit_token, visit_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&subtitle)
    .bind(serde_json::to_string(&config).unwrap_or_else(|_| "{}".into()))
    .bind(serde_json::to_string(&questions).unwrap_or_else(|_| "[]".into()))
    .bind(serde_json::to_string(&levels).unwrap_or_else(|_| "[]".into()))
    .bind("pending")
    .bind(&edit_token)
    .execute(&state.pools.exam)
    .await?;

    // 异步审核
    spawn_audit(&state, id.clone(), config, questions);

    Ok(Json(json!({ "id": id, "editToken": edit_token, "message": "提交成功" })).into_response())
}

// POST /exams/import（公开导入：忠实旧实现无 admin 校验）
async fn import_exam(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let errors = validate_exam_data(&body);
    if !errors.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "数据格式验证失败", "details": errors })),
        )
            .into_response());
    }

    let config = body.get("config").cloned().unwrap_or(json!({}));
    let questions = body.get("questions").cloned().unwrap_or(json!([]));
    let levels = body.get("levels").cloned().unwrap_or(json!([]));

    let id = uuid::Uuid::new_v4().to_string();
    let edit_token = uuid::Uuid::new_v4().to_string();
    let title = config
        .get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("未命名试卷")
        .to_string();
    let subtitle = config
        .get("paperSubtitle")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 清理 questions 的 id，重新生成（保持其余字段，对齐旧实现）
    let cleaned: Vec<Value> = questions
        .as_array()
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(idx, q)| {
                    let mut q = q.clone();
                    if let Some(o) = q.as_object_mut() {
                        o.insert("id".into(), json!(format!("q_{id}_{}", idx + 1)));
                    }
                    q
                })
                .collect()
        })
        .unwrap_or_default();
    let cleaned_val = Value::Array(cleaned.clone());
    let levels_len = levels.as_array().map(|a| a.len()).unwrap_or(0);

    sqlx::query(
        "INSERT INTO exams (id, title, subtitle, config, questions, levels, status, edit_token, visit_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&subtitle)
    .bind(serde_json::to_string(&config).unwrap_or_else(|_| "{}".into()))
    .bind(serde_json::to_string(&cleaned_val).unwrap_or_else(|_| "[]".into()))
    .bind(serde_json::to_string(&levels).unwrap_or_else(|_| "[]".into()))
    .bind("pending")
    .bind(&edit_token)
    .execute(&state.pools.exam)
    .await?;

    spawn_audit(&state, id.clone(), config, cleaned_val);

    Ok(Json(json!({
        "id": id,
        "editToken": edit_token,
        "message": "导入成功，正在审核中",
        "importedQuestions": cleaned.len(),
        "importedLevels": levels_len,
    }))
    .into_response())
}

// GET /exams/:id（公开，仅 published；自增 visit_count；剔除 edit_token）
async fn get_exam(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    let row: Option<ExamRow> = sqlx::query_as(
        "SELECT id, title, subtitle, config, questions, levels, status, edit_token, \
         CAST(visit_count AS INTEGER) AS visit_count, ai_reason, created_at, updated_at \
         FROM exams WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.pools.exam)
    .await?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "试卷不存在" })),
            )
                .into_response())
        }
    };

    if row.status.as_deref() != Some("published") {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "EXAM_UNAVAILABLE",
                "status": row.status,
                "reason": row.ai_reason,
            })),
        )
            .into_response());
    }

    // visit_count + 1
    let _ = sqlx::query("UPDATE exams SET visit_count = visit_count + 1 WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.exam)
        .await;

    // edit_token: undefined（剔除）
    let exam = json!({
        "id": row.id,
        "title": row.title,
        "subtitle": row.subtitle,
        "config": parse_json_or(row.config.as_deref(), json!({})),
        "questions": parse_json_or(row.questions.as_deref(), json!([])),
        "levels": parse_json_or(row.levels.as_deref(), json!([])),
        "status": row.status,
        "visit_count": row.visit_count.unwrap_or(0),
        "ai_reason": row.ai_reason,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    });
    Ok(Json(fix_url_path(exam)).into_response())
}

// POST /exams/:id/verify（edit_token 校验，返回完整可编辑试卷）
async fn verify_exam(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let token = body.get("token").and_then(|v| v.as_str()).unwrap_or("");
    if token.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Token is required" })),
        )
            .into_response());
    }

    let row: Option<ExamRow> = sqlx::query_as(
        "SELECT id, title, subtitle, config, questions, levels, status, edit_token, \
         CAST(visit_count AS INTEGER) AS visit_count, ai_reason, created_at, updated_at \
         FROM exams WHERE id = ? AND edit_token = ?",
    )
    .bind(&id)
    .bind(token)
    .fetch_optional(&state.pools.exam)
    .await?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok((
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "权限验证失败或试卷不存在" })),
            )
                .into_response())
        }
    };

    // 返回完整行（含 edit_token，对齐旧 {...row}）
    let data = json!({
        "id": row.id,
        "title": row.title,
        "subtitle": row.subtitle,
        "config": parse_json_or(row.config.as_deref(), json!({})),
        "questions": parse_json_or(row.questions.as_deref(), json!([])),
        "levels": parse_json_or(row.levels.as_deref(), json!([])),
        "status": row.status,
        "edit_token": row.edit_token,
        "visit_count": row.visit_count.unwrap_or(0),
        "ai_reason": row.ai_reason,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    });
    Ok(Json(fix_url_path(data)).into_response())
}

// PUT /exams/:id（edit_token 校验 → 更新 + status=pending + 重新异步审核）
async fn update_exam(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Response> {
    let token = body.get("token").and_then(|v| v.as_str()).unwrap_or("");
    if token.is_empty() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Missing token" })),
        )
            .into_response());
    }

    // 校验 token
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT id FROM exams WHERE id = ? AND edit_token = ?")
            .bind(&id)
            .bind(token)
            .fetch_optional(&state.pools.exam)
            .await?;
    if exists.is_none() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "无权修改此试卷" })),
        )
            .into_response());
    }

    let config = body.get("config").cloned().unwrap_or(json!({}));
    let questions = body.get("questions").cloned().unwrap_or(json!([]));
    let levels = body.get("levels").cloned().unwrap_or(json!([]));
    let title = config.get("title").and_then(|v| v.as_str());
    let subtitle = config.get("paperSubtitle").and_then(|v| v.as_str());

    sqlx::query(
        "UPDATE exams SET title=?, subtitle=?, config=?, questions=?, levels=?, \
         status='pending', ai_reason='重新审核中...', updated_at=CURRENT_TIMESTAMP WHERE id=?",
    )
    .bind(title)
    .bind(subtitle)
    .bind(serde_json::to_string(&config).unwrap_or_else(|_| "{}".into()))
    .bind(serde_json::to_string(&questions).unwrap_or_else(|_| "[]".into()))
    .bind(serde_json::to_string(&levels).unwrap_or_else(|_| "[]".into()))
    .bind(&id)
    .execute(&state.pools.exam)
    .await?;

    // 重新异步审核
    spawn_audit(&state, id.clone(), config, questions);

    Ok(Json(json!({ "success": true, "message": "更新成功，正在重新审核" })).into_response())
}

// ============================================================
// 后台（RBAC）
// ============================================================

// GET /admin/stats（Read）
async fn admin_stats(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "exam", Action::Read).await?;

    let site_visits: Option<i64> = sqlx::query_scalar(
        "SELECT CAST(value AS INTEGER) FROM site_stats WHERE key='total_visits'",
    )
    .fetch_optional(&state.pools.exam)
    .await?
    .flatten();
    let total_exams: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exams")
        .fetch_one(&state.pools.exam)
        .await?;
    let total_views: Option<i64> =
        sqlx::query_scalar("SELECT CAST(SUM(visit_count) AS INTEGER) FROM exams")
            .fetch_one(&state.pools.exam)
            .await?;

    Ok(Json(json!({
        "siteVisits": site_visits.unwrap_or(0),
        "totalExams": total_exams,
        "totalExamViews": total_views.unwrap_or(0),
    })))
}

// GET /admin/list（Read）
async fn admin_list(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "exam", Action::Read).await?;

    let rows: Vec<(String, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, title, status, CAST(visit_count AS INTEGER) AS visit_count, ai_reason, created_at, edit_token, config \
             FROM exams ORDER BY created_at DESC",
        )
        .fetch_all(&state.pools.exam)
        .await?;

    let data: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, title, status, visit_count, ai_reason, created_at, edit_token, config)| {
                let edit_link = format!(
                    "/exam/create?id={id}&token={}",
                    edit_token.clone().unwrap_or_default()
                );
                json!({
                    "id": id,
                    "title": title,
                    "status": status,
                    "visit_count": visit_count,
                    "ai_reason": ai_reason,
                    "created_at": created_at,
                    "edit_token": edit_token,
                    "config": parse_json_or(config.as_deref(), json!({})),
                    "edit_link": edit_link,
                })
            },
        )
        .collect();

    Ok(Json(fix_url_path(Value::Array(data))))
}

// POST /admin/exams/:id/status（Moderate）
async fn admin_set_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "exam", Action::Moderate).await?;
    let status = body.get("status").and_then(|v| v.as_str());
    sqlx::query("UPDATE exams SET status = ? WHERE id = ?")
        .bind(status)
        .bind(&id)
        .execute(&state.pools.exam)
        .await?;
    Ok(Json(json!({ "success": true })))
}

// DELETE /admin/exams/:id（Manage）：删除关联文件 + submissions + exam
async fn admin_delete_exam(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    authorize(&state.pools.core, &user, "exam", Action::Manage).await?;

    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT questions, levels FROM exams WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.pools.exam)
            .await?;
    let (questions_json, levels_json) = match row {
        Some(r) => r,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Exam not found" })),
            )
                .into_response())
        }
    };

    // 删除引用文件
    let file_urls = extract_file_paths(
        questions_json.as_deref().unwrap_or("[]"),
        levels_json.as_deref().unwrap_or("[]"),
    );
    let exam_dir = state.cfg.uploads_subdir("exam");
    let mut deleted_files = 0_i64;
    for url in file_urls {
        if url.contains("/uploads/") {
            if let Some(name) = basename(&url) {
                if tokio::fs::remove_file(exam_dir.join(&name)).await.is_ok() {
                    deleted_files += 1;
                }
            }
        }
    }

    // 删 submissions + exam
    sqlx::query("DELETE FROM submissions WHERE exam_id = ?")
        .bind(&id)
        .execute(&state.pools.exam)
        .await?;
    sqlx::query("DELETE FROM exams WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.exam)
        .await?;

    Ok(Json(json!({ "success": true, "deletedFiles": deleted_files })).into_response())
}
