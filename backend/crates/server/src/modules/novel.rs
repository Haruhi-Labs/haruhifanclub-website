//! novel 模块：书库（EPUB 上传/解析/封面、浏览、后台管理）。
//! 对应旧 haruhi-novel-reader 后端，统一挂载于 /api/novel。

use std::path::{Path as FsPath, PathBuf};

use axum::extract::{Multipart, Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult};
use serde_json::{json, Value};

use crate::state::AppState;

type BookRow = (
    String,         // id
    String,         // title
    Option<String>, // author
    Option<String>, // cover_path
    Option<String>, // file_path
    Option<String>, // upload_date
    Option<String>, // category
    Option<f64>,    // sort_order
);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/books", get(list_books))
        .route("/books/{id}", get(get_book))
        .route("/admin/upload", post(upload))
        .route("/admin/books/{id}", patch(update_book).delete(delete_book))
}

fn book_to_json(b: BookRow) -> Value {
    let (id, title, author, cover_path, file_path, upload_date, category, order) = b;
    json!({
        "id": id, "title": title, "author": author,
        "cover_path": cover_path, "file_path": file_path,
        "upload_date": upload_date, "category": category,
        "order": order,
    })
}

const SELECT_COLS: &str =
    "SELECT id, title, author, cover_path, file_path, upload_date, category, sort_order FROM books";

// ---------- 公开接口 ----------

async fn list_books(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<BookRow> = sqlx::query_as(&format!("{SELECT_COLS} ORDER BY sort_order ASC"))
        .fetch_all(&state.pools.novel)
        .await?;
    let books: Vec<Value> = rows.into_iter().map(book_to_json).collect();
    Ok(Json(Value::Array(books)))
}

async fn get_book(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Value>> {
    let row: Option<BookRow> = sqlx::query_as(&format!("{SELECT_COLS} WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pools.novel)
        .await?;
    match row {
        Some(b) => Ok(Json(book_to_json(b))),
        None => Err(AppError::not_found("Book not found")),
    }
}

// ---------- 后台接口（需 novel 权限）----------

async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "novel", Action::Write).await?;

    // 读取 multipart 中的 file 字段
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_name = String::from("book.epub");
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("解析上传失败: {e}")))?
    {
        if field.name() == Some("file") {
            if let Some(n) = field.file_name() {
                original_name = n.to_string();
            }
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("读取文件失败: {e}")))?;
            file_bytes = Some(bytes.to_vec());
        }
    }
    let file_bytes = file_bytes.ok_or_else(|| AppError::bad_request("No file"))?;

    let ts = chrono::Utc::now().timestamp_millis();
    let id = ts.to_string();
    let ext = haruhi_media::ext_of(&original_name, "epub");
    let file_name = format!("{ts}.{ext}");

    let novel_root = state.cfg.uploads_subdir("novel");
    let files_dir = novel_root.join("files");
    let covers_dir = novel_root.join("covers");

    haruhi_media::save_file(&files_dir, &file_name, &file_bytes).await?;
    let file_rel = format!("novel/files/{file_name}");

    // 解析 EPUB（阻塞任务）
    let epub_path = files_dir.join(&file_name);
    let parsed = tokio::task::spawn_blocking(move || haruhi_media::read_epub(&epub_path))
        .await
        .map_err(|e| AppError::internal(format!("解析任务异常: {e}")))?;

    let (title, author, cover_rel) = match parsed {
        Ok(info) => {
            let title = info
                .title
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| default_title(&original_name));
            let author = info
                .author
                .filter(|a| !a.trim().is_empty())
                .unwrap_or_else(|| "佚名".to_string());
            let cover_rel = match info.cover {
                Some((data, mime)) => save_cover(&covers_dir, &id, data, &mime).await,
                None => None,
            };
            (title, author, cover_rel)
        }
        Err(e) => {
            tracing::warn!("EPUB 解析失败，按文件名降级入库: {e}");
            (default_title(&original_name), "解析失败".to_string(), None)
        }
    };

    let upload_iso = chrono::Utc::now().to_rfc3339();
    let sort_order = -(ts as f64);
    sqlx::query(
        "INSERT INTO books (id, title, author, cover_path, file_path, upload_date, category, sort_order) \
         VALUES (?, ?, ?, ?, ?, ?, NULL, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&author)
    .bind(&cover_rel)
    .bind(&file_rel)
    .bind(&upload_iso)
    .bind(sort_order)
    .execute(&state.pools.novel)
    .await?;

    Ok(Json(json!({ "success": true, "id": id, "title": title })))
}

async fn update_book(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "novel", Action::Write).await?;
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::bad_request("请求体须为对象"))?;

    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM books WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pools.novel)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("Not found"));
    }

    if let Some(v) = obj.get("title").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE books SET title = ? WHERE id = ?")
            .bind(v)
            .bind(&id)
            .execute(&state.pools.novel)
            .await?;
    }
    if let Some(v) = obj.get("author").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE books SET author = ? WHERE id = ?")
            .bind(v)
            .bind(&id)
            .execute(&state.pools.novel)
            .await?;
    }
    // category：键存在即更新（可为 null 或字符串），与旧逻辑一致
    if obj.contains_key("category") {
        let c = obj.get("category").and_then(|v| v.as_str());
        sqlx::query("UPDATE books SET category = ? WHERE id = ?")
            .bind(c)
            .bind(&id)
            .execute(&state.pools.novel)
            .await?;
    }
    if let Some(v) = obj.get("order").and_then(|v| v.as_f64()) {
        sqlx::query("UPDATE books SET sort_order = ? WHERE id = ?")
            .bind(v)
            .bind(&id)
            .execute(&state.pools.novel)
            .await?;
    }

    let updated: BookRow = sqlx::query_as(&format!("{SELECT_COLS} WHERE id = ?"))
        .bind(&id)
        .fetch_one(&state.pools.novel)
        .await?;
    Ok(Json(
        json!({ "success": true, "book": book_to_json(updated) }),
    ))
}

async fn delete_book(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "novel", Action::Manage).await?;

    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT file_path, cover_path FROM books WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.pools.novel)
            .await?;
    let (file_path, cover_path) = row.ok_or_else(|| AppError::not_found("Not found"))?;

    // 删除磁盘文件（尽力而为）
    let uploads_root = &state.cfg.uploads_dir;
    remove_quietly(uploads_root, file_path.as_deref()).await;
    remove_quietly(uploads_root, cover_path.as_deref()).await;

    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(&id)
        .execute(&state.pools.novel)
        .await?;
    Ok(Json(json!({ "success": true })))
}

// ---------- 辅助 ----------

fn default_title(name: &str) -> String {
    let stem = FsPath::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if stem.trim().is_empty() {
        "未命名书籍".to_string()
    } else {
        stem.to_string()
    }
}

/// 保存封面：优先转 WebP（quality 80），失败则按原 mime 落盘。返回相对 uploads 根的路径。
async fn save_cover(covers_dir: &FsPath, id: &str, data: Vec<u8>, mime: &str) -> Option<String> {
    let webp_name = format!("cover-{id}.webp");
    let data_for_webp = data.clone();
    let webp = tokio::task::spawn_blocking(move || haruhi_media::encode_webp(&data_for_webp, 80.0))
        .await
        .ok()
        .and_then(|r| r.ok());

    if let Some(bytes) = webp {
        if haruhi_media::save_file(covers_dir, &webp_name, &bytes)
            .await
            .is_ok()
        {
            return Some(format!("novel/covers/{webp_name}"));
        }
    }
    // 降级：原始格式
    let ext = mime.split('/').nth(1).unwrap_or("jpg");
    let name = format!("cover-{id}.{ext}");
    if haruhi_media::save_file(covers_dir, &name, &data)
        .await
        .is_ok()
    {
        Some(format!("novel/covers/{name}"))
    } else {
        None
    }
}

async fn remove_quietly(uploads_root: &FsPath, rel: Option<&str>) {
    if let Some(rel) = rel.filter(|r| !r.is_empty()) {
        let abs: PathBuf = uploads_root.join(rel);
        let _ = tokio::fs::remove_file(abs).await;
    }
}
