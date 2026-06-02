//! 文件落盘助手：保存上传字节、Base64 图片（MD5 去重，供 news 复用）。

use std::path::{Path, PathBuf};

use base64::Engine;
use md5::{Digest, Md5};

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
