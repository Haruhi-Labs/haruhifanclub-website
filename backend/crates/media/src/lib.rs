//! haruhi-media：图片 WebP 压缩、EPUB 元数据/封面提取、文件落盘、Base64 图片。
//! 被 novel（封面）/ art / news（图片）/ exam（缩略图）等模块复用。

use std::path::{Path, PathBuf};

pub mod audio;
pub mod epub_ops;
pub mod image_ops;
pub mod upload;

pub use epub_ops::{read_epub, EpubInfo};
pub use image_ops::encode_webp;
pub use upload::{
    check_image, check_media, is_audio_ext, is_image_ext, save_base64_image, save_file,
    UploadReject, MAX_AUDIO_BYTES, MAX_IMAGE_BYTES,
};

/// 确保目录存在。
pub async fn ensure_dir(dir: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dir).await
}

/// 由原始文件名取扩展名（小写，不含点），缺省返回 fallback。
pub fn ext_of(name: &str, fallback: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| fallback.to_string())
}

/// 拼接 uploads 子路径（统一用正斜杠存库）。
pub fn rel_join(module: &str, sub: &str, filename: &str) -> String {
    format!("{module}/{sub}/{filename}")
}

/// 给定 uploads 根 + 库中相对路径，得到磁盘绝对路径。
pub fn abs_upload(uploads_root: &Path, rel: &str) -> PathBuf {
    uploads_root.join(rel)
}
