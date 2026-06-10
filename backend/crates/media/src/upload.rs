//! 文件落盘助手：保存上传字节、Base64 图片（MD5 去重，供 news 复用）。
//! 以及匿名上传的类型/大小白名单校验（art/exam 等公开上传口防滥用）。

use std::path::{Path, PathBuf};

use base64::Engine;
use md5::{Digest, Md5};

/// 允许上传的图片扩展名（小写，不含点）。
pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "svg", "bmp"];
/// 允许上传的音频扩展名（小写，不含点）。
pub const AUDIO_EXTS: &[&str] = &["mp3", "wav", "ogg", "m4a", "aac", "flac", "wma", "amr"];
/// 单张图片大小上限（字节）。原图可能较大，给到 32MB。
pub const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;
/// 单个音频大小上限（字节）。
pub const MAX_AUDIO_BYTES: usize = 64 * 1024 * 1024;

/// 上传校验失败原因（调用方转成 400）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadReject {
    /// 不支持的扩展名
    BadType(String),
    /// 超过大小上限（实际字节, 上限字节）
    TooLarge(usize, usize),
}

impl std::fmt::Display for UploadReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadReject::BadType(ext) => write!(f, "不支持的文件类型: .{ext}"),
            UploadReject::TooLarge(_, max) => {
                write!(f, "文件过大（上限 {} MB）", max / 1024 / 1024)
            }
        }
    }
}

pub fn is_image_ext(ext: &str) -> bool {
    IMAGE_EXTS.contains(&ext)
}

pub fn is_audio_ext(ext: &str) -> bool {
    AUDIO_EXTS.contains(&ext)
}

/// 校验图片上传：扩展名在白名单内、大小不超限。
pub fn check_image(ext: &str, size: usize) -> Result<(), UploadReject> {
    if !is_image_ext(ext) {
        return Err(UploadReject::BadType(ext.to_string()));
    }
    if size > MAX_IMAGE_BYTES {
        return Err(UploadReject::TooLarge(size, MAX_IMAGE_BYTES));
    }
    Ok(())
}

/// 校验媒体上传（图片或音频）：类型在白名单内、大小按类别不超限。
pub fn check_media(ext: &str, size: usize) -> Result<(), UploadReject> {
    if is_image_ext(ext) {
        if size > MAX_IMAGE_BYTES {
            return Err(UploadReject::TooLarge(size, MAX_IMAGE_BYTES));
        }
        Ok(())
    } else if is_audio_ext(ext) {
        if size > MAX_AUDIO_BYTES {
            return Err(UploadReject::TooLarge(size, MAX_AUDIO_BYTES));
        }
        Ok(())
    } else {
        Err(UploadReject::BadType(ext.to_string()))
    }
}

/// 把字节写入 dir/filename，返回绝对路径。
pub async fn save_file(dir: &Path, filename: &str, bytes: &[u8]) -> anyhow::Result<PathBuf> {
    tokio::fs::create_dir_all(dir).await?;
    let full = dir.join(filename);
    tokio::fs::write(&full, bytes).await?;
    Ok(full)
}

/// 解析 `data:image/png;base64,xxxx` 形式的 Base64 图片，按内容 MD5 去重落盘。
/// 返回相对 uploads 根的路径 `<module>/<md5>.<ext>`（与 harunews 行为一致）。
pub async fn save_base64_image(
    uploads_root: &Path,
    module: &str,
    data_url: &str,
) -> anyhow::Result<String> {
    let (mime, b64) = data_url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or_else(|| anyhow::anyhow!("非法的 Base64 图片"))?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| anyhow::anyhow!("Base64 解码失败: {e}"))?;

    let ext = match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        _ => "bin",
    };

    let mut hasher = Md5::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let filename = format!("{:x}.{ext}", hash);
    let rel = format!("{module}/{filename}");

    let dir = uploads_root.join(module);
    save_file(&dir, &filename, &bytes).await?;
    Ok(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_image_type_and_size() {
        assert!(check_image("png", 1024).is_ok());
        assert!(check_image("WEBP".to_lowercase().as_str(), 1024).is_ok());
        // 非图片类型拒绝
        assert_eq!(
            check_image("exe", 1024),
            Err(UploadReject::BadType("exe".into()))
        );
        // 超限拒绝
        assert!(matches!(
            check_image("png", MAX_IMAGE_BYTES + 1),
            Err(UploadReject::TooLarge(_, _))
        ));
    }

    #[test]
    fn check_media_accepts_image_and_audio_rejects_other() {
        assert!(check_media("jpg", 1024).is_ok());
        assert!(check_media("mp3", 1024).is_ok());
        // 可执行/脚本类一律拒绝
        assert!(matches!(
            check_media("html", 10),
            Err(UploadReject::BadType(_))
        ));
        assert!(matches!(
            check_media("js", 10),
            Err(UploadReject::BadType(_))
        ));
        // 音频用音频上限
        assert!(check_media("mp3", MAX_IMAGE_BYTES + 1).is_ok());
        assert!(matches!(
            check_media("mp3", MAX_AUDIO_BYTES + 1),
            Err(UploadReject::TooLarge(_, _))
        ));
    }
}
