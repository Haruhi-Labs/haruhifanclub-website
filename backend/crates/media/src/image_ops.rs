//! 图片处理：解码任意常见格式并编码为 WebP（替代旧项目的 sharp）。

use image::DynamicImage;

/// 把图片字节编码为 WebP，quality 0-100（与 sharp 的 webp({quality}) 对齐）。
pub fn encode_webp(input: &[u8], quality: f32) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(input).map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;
    let encoder =
        webp::Encoder::from_image(&img).map_err(|e| anyhow::anyhow!("WebP 编码器创建失败: {e}"))?;
    let encoded = encoder.encode(quality);
    Ok(encoded.to_vec())
}

/// 生成限宽缩略图并编码为 WebP（等比缩放，不放大小图）。
/// JPEG 输入会先按 EXIF Orientation 矫正方向——浏览器显示原图时会自动矫正，
/// 缩略图必须保持一致，否则旧迁移的 JPG 会出现转向。
pub fn thumbnail_webp(input: &[u8], max_w: u32, quality: f32) -> anyhow::Result<Vec<u8>> {
    let mut img =
        image::load_from_memory(input).map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;
    if let Some(o) = jpeg_exif_orientation(input) {
        img = apply_orientation(img, o);
    }
    if img.width() > max_w {
        let h = ((max_w as u64 * img.height() as u64) / img.width() as u64).max(1) as u32;
        img = img.thumbnail(max_w, h);
    }
    // webp::Encoder 只接受 RGB8/RGBA8，其余色深（16bit PNG、灰度等）先转换
    let img = match img {
        DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_) => img,
        other => DynamicImage::ImageRgba8(other.to_rgba8()),
    };
    let encoder =
        webp::Encoder::from_image(&img).map_err(|e| anyhow::anyhow!("WebP 编码器创建失败: {e}"))?;
    Ok(encoder.encode(quality).to_vec())
}

/// 按 EXIF Orientation（1-8）矫正图像方向。
fn apply_orientation(img: DynamicImage, orientation: u16) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(), // transpose
        6 => img.rotate90(),
        7 => img.rotate270().fliph(), // transverse
        8 => img.rotate270(),
        _ => img,
    }
}

/// 从 JPEG 字节中提取 EXIF Orientation（1-8）。非 JPEG / 无 EXIF 返回 None。
/// 内联极简解析器（仅找 IFD0 的 0x0112 标签），避免引入 exif 依赖。
fn jpeg_exif_orientation(data: &[u8]) -> Option<u16> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None; // 非 JPEG（无 SOI）
    }
    let mut i = 2usize;
    while i + 4 <= data.len() {
        if data[i] != 0xFF {
            return None;
        }
        let marker = data[i + 1];
        if marker == 0xFF {
            i += 1; // 填充字节
            continue;
        }
        // SOS/EOI 之后不会再出现 APP1
        if marker == 0xDA || marker == 0xD9 {
            return None;
        }
        // standalone 标记（RST0-7 / TEM）无长度域
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if len < 2 || i + 2 + len > data.len() {
            return None;
        }
        if marker == 0xE1 {
            let seg = &data[i + 4..i + 2 + len];
            if seg.len() > 6 && &seg[..6] == b"Exif\0\0" {
                return tiff_orientation(&seg[6..]);
            }
        }
        i += 2 + len;
    }
    None
}

/// 在 TIFF 结构的 IFD0 里找 Orientation（0x0112）标签。
fn tiff_orientation(t: &[u8]) -> Option<u16> {
    if t.len() < 8 {
        return None;
    }
    let le = match &t[..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let rd16 = |b: &[u8]| {
        if le {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        }
    };
    let rd32 = |b: &[u8]| {
        if le {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        }
    };
    if rd16(&t[2..4]) != 42 {
        return None;
    }
    let ifd = rd32(&t[4..8]) as usize;
    if ifd + 2 > t.len() {
        return None;
    }
    let n = rd16(&t[ifd..ifd + 2]) as usize;
    for k in 0..n {
        let e = ifd + 2 + k * 12;
        if e + 12 > t.len() {
            return None;
        }
        if rd16(&t[e..e + 2]) == 0x0112 {
            // 类型 SHORT(3)、count 1：值内联在 value 域前 2 字节
            let v = rd16(&t[e + 8..e + 10]);
            return (1..=8).contains(&v).then_some(v);
        }
    }
    None
}
