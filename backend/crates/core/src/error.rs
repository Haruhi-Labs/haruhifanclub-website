//! 统一错误类型：所有 handler 返回 `AppResult<T>`，错误自动转成 JSON 响应。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),

    #[error("未授权")]
    Unauthorized,

    #[error("无权限")]
    Forbidden,

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            // sqlx 的 RowNotFound 映射为 404，其余为 500
            AppError::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        // 5xx 记录详情，但不向客户端暴露内部信息
        if status.is_server_error() {
            tracing::error!(error = %self, "服务端错误");
        }
        let message = match &self {
            AppError::Database(sqlx::Error::RowNotFound) => "资源不存在".to_string(),
            AppError::Database(_) => "服务器内部错误".to_string(),
            other => other.to_string(),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
