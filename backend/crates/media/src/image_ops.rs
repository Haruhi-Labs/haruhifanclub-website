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

/// 头像解码时的单边像素上限：覆盖真实相机/手机大图（数千万像素），
/// 但能在解码前据图头尺寸拒掉「小文件解出超大画布」的解压炸弹。
const AVATAR_MAX_DECODE_DIM: u32 = 8_000;
/// 头像解码时的内存分配上限（字节），与尺寸上限互为兜底。
const AVATAR_MAX_DECODE_ALLOC: u64 = 384 * 1024 * 1024;

/// 头像专用：把任意图片居中裁成正方形并缩放到 `size`×`size`，编码为 WebP。
///
/// 前端已做交互式裁切，但服务端仍统一兜底裁切 + 限尺寸 + 转 WebP——既杜绝
/// 客户端绕过直传超大原图，又保证库里头像格式/尺寸可控。`resize_to_fill`
/// 保持比例缩放后居中裁剪到精确边长，等价于成熟产品「填满正方形」的语义。
///
/// 解码走带 `Limits` 的 `ImageReader`：仅靠字节上限挡不住高压缩比图片/解压炸弹
/// （几十 KB 可解出上亿像素），这里在解码前按图头尺寸与分配额提前拒绝，避免
/// 在阻塞线程池上吃满 CPU/内存。
pub fn square_avatar_webp(input: &[u8], size: u32, quality: f32) -> anyhow::Result<Vec<u8>> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(AVATAR_MAX_DECODE_DIM);
    limits.max_image_height = Some(AVATAR_MAX_DECODE_DIM);
    limits.max_alloc = Some(AVATAR_MAX_DECODE_ALLOC);

    let mut reader = image::ImageReader::new(std::io::Cursor::new(input))
        .with_guessed_format()
        .map_err(|e| anyhow::anyhow!("图片格式识别失败: {e}"))?;
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;

    let square = img.resize_to_fill(size, size, image::imageops::FilterType::Lanczos3);
    let encoder = webp::Encoder::from_image(&square)
        .map_err(|e| anyhow::anyhow!("WebP 编码器创建失败: {e}"))?;
    Ok(encoder.encode(quality).to_vec())
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
    let mut cmd = tokio::process::Command::new("vips");
    cmd.arg("thumbnail")
        .arg(src)
        .arg(&dst_spec)
        .arg(max_w.to_string())
        // 仅缩小不放大：小图原样输出，避免把低清图拉糊还占体积
        .arg("--size")
        .arg("down")
        // 请求取消 / 超时（future 被 drop）时连带杀掉子进程，避免孤儿 vips 堆积
        .kill_on_drop(true);
    // 硬超时：病态图或 IO 阻塞导致 vips 卡死时，不让它无期限占住调用方的并发许可
    // （art 侧 THUMB_GATE 仅 2 个，卡死即拖垮整个缩略图服务）。30s 远超正常生成（毫秒级）。
    let status = tokio::time::timeout(std::time::Duration::from_secs(30), cmd.status())
        .await
        .map_err(|_| anyhow::anyhow!("vips 执行超时（>30s），已中止"))?
        .map_err(|e| anyhow::anyhow!("启动 vips 失败（是否已安装 libvips-tools?）: {e}"))?;
    if !status.success() {
        anyhow::bail!("vips 退出码非零: {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 非正方形输入应被居中裁切并缩放为精确正方形 WebP。
    #[test]
    fn square_avatar_outputs_exact_square_webp() {
        // 造一张 200×100 的纯色 PNG 作为输入
        let buf = image::RgbImage::from_pixel(200, 100, image::Rgb([200, 30, 40]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(buf)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();

        let out = square_avatar_webp(png.get_ref(), 128, 80.0).unwrap();
        // WebP 魔数：RIFF....WEBP
        assert_eq!(&out[0..4], b"RIFF");
        assert_eq!(&out[8..12], b"WEBP");
        // 解码回来必须是精确 128×128
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (128, 128));
    }

    #[test]
    fn square_avatar_rejects_non_image() {
        assert!(square_avatar_webp(b"not an image", 128, 80.0).is_err());
    }
}
