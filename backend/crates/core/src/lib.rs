//! haruhi-core：统一配置、错误类型、响应封装。
//! 被所有其它后端 crate 复用。

pub mod config;
pub mod error;
pub mod parse;

pub use config::{Config, MailConfig};
pub use error::{AppError, AppResult};
