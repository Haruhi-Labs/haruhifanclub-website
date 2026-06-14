//! 图片处理：encode_webp 进程内编码（小图，novel/news/exam 上传用）；
//! 缩略图走 libvips 子进程（thumbnail_webp_vips，流式、内存有界）。

use std::path::Path;

/// 把图片字节编码为 WebP，quality 0-100（与 sharp 的 webp({quality}) 对齐）。
pub fn encode_webp(input: &[u8], quality: f32) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(input).map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;
    let encoder =
        webp::Encoder::from_image(&img).map_err(|e| anyhow::anyhow!("WebP 编码器创建失败: {e}"))?;
    let encoded = encoder.encode(quality);
    Ok(encoded.to_vec())
}

/// 用系统 libvips 生成限宽 WebP 缩略图：`vips thumbnail src dst[Q=..,strip] W --size down`。
///
/// 相比进程内 `image` 全解码，libvips 采用 shrink-on-load + 流式管线，
/// 内存占用与源图尺寸基本无关（几十 MB 量级），从根上消除"巨图全解码进堆 +
/// glibc 滞留"导致的 RSS 膨胀/OOM 风险。vips thumbnail 默认按 EXIF Orientation
/// 自动旋转，无需手工矫正方向。失败（vips 不存在 / 非零退出）返回 Err，调用方据此降级。
pub async fn thumbnail_webp_vips(
    src: &Path,
    dst: &Path,
    max_w: u32,
    quality: u8,
) -> anyhow::Result<()> {
    // dst 扩展名为 .webp → vips 自动用 webpsave；Q=质量，strip=去元数据
    let dst_spec = format!("{}[Q={},strip]", dst.to_string_lossy(), quality);
    let status = tokio::process::Command::new("vips")
        .arg("thumbnail")
        .arg(src)
        .arg(&dst_spec)
        .arg(max_w.to_string())
        // 仅缩小不放大：小图原样输出，避免把低清图拉糊还占体积
        .arg("--size")
        .arg("down")
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("启动 vips 失败（是否已安装 libvips-tools?）: {e}"))?;
    if !status.success() {
        anyhow::bail!("vips 退出码非零: {status}");
    }
    Ok(())
}
