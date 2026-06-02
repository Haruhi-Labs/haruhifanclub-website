//! EPUB 元数据与封面提取（替代旧项目的 Node `epub` 库）。

use std::path::Path;

pub struct EpubInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    /// (图片字节, mime)
    pub cover: Option<(Vec<u8>, String)>,
}

/// 解析 EPUB，提取标题/作者/封面。解析失败由调用方降级处理。
pub fn read_epub(path: &Path) -> anyhow::Result<EpubInfo> {
    let mut doc =
        epub::doc::EpubDoc::new(path).map_err(|e| anyhow::anyhow!("EPUB 打开失败: {e}"))?;

    let title = doc.mdata("title").map(|m| m.value.clone());
    let author = doc.mdata("creator").map(|m| m.value.clone());
    let cover = doc.get_cover();

    Ok(EpubInfo {
        title,
        author,
        cover,
    })
}
