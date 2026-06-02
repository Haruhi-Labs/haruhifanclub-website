//! 图片处理：解码任意常见格式并编码为 WebP（替代旧项目的 sharp）。

/// 把图片字节编码为 WebP，quality 0-100（与 sharp 的 webp({quality}) 对齐）。
pub fn encode_webp(input: &[u8], quality: f32) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(input).map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;
    let encoder =
        webp::Encoder::from_image(&img).map_err(|e| anyhow::anyhow!("WebP 编码器创建失败: {e}"))?;
    let encoded = encoder.encode(quality);
    Ok(encoded.to_vec())
}
