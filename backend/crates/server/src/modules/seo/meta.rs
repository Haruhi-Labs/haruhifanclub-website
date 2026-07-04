//! PageMeta：一个内容页要注入的元数据，渲染为 head 标记块 HTML。
//!
//! 渲染产物整块替换 index.html 中的 `<!-- seo:meta --> … <!-- /seo:meta -->`；
//! 所有标签带 `data-seo="ssr"`，前端 @haruhi/seo 在数据就绪后按该属性接管替换。

use super::esc;

pub struct PageMeta {
    /// 完整标题（含站名后缀）。404 壳也要给默认标题——标记块被整体替换，title 不能缺。
    pub title: String,
    pub description: Option<String>,
    /// 规范 URL（绝对地址）；同时派生 og:url。
    pub canonical: Option<String>,
    /// og:type：article / book / product / website。
    pub og_type: &'static str,
    /// og:image（绝对 URL）。
    pub og_image: Option<String>,
    pub json_ld: Option<serde_json::Value>,
    /// Some("noindex")：内容不存在/不可见的 404 壳。
    pub robots: Option<&'static str>,
}

impl PageMeta {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            canonical: None,
            og_type: "website",
            og_image: None,
            json_ld: None,
            robots: None,
        }
    }

    /// 内容不存在/不可见时的 404 壳 meta：默认标题 + noindex。
    pub fn not_found(default_title: &str) -> Self {
        Self {
            robots: Some("noindex"),
            ..Self::new(default_title)
        }
    }
}

fn tag_meta(out: &mut String, attr: &str, key: &str, content: &str) {
    out.push_str(&format!(
        "    <meta {attr}=\"{key}\" content=\"{}\" data-seo=\"ssr\" />\n",
        esc(content)
    ));
}

/// 渲染标记块（含起止注释本身）。
pub fn render_head_block(m: &PageMeta) -> String {
    let mut out = String::from("<!-- seo:meta -->\n");
    out.push_str(&format!("    <title>{}</title>\n", esc(&m.title)));
    tag_meta(&mut out, "property", "og:site_name", "凉宫春日应援团");
    tag_meta(&mut out, "property", "og:type", m.og_type);
    tag_meta(&mut out, "property", "og:title", &m.title);
    if let Some(d) = &m.description {
        tag_meta(&mut out, "name", "description", d);
        tag_meta(&mut out, "property", "og:description", d);
    }
    if let Some(c) = &m.canonical {
        out.push_str(&format!(
            "    <link rel=\"canonical\" href=\"{}\" data-seo=\"ssr\" />\n",
            esc(c)
        ));
        tag_meta(&mut out, "property", "og:url", c);
    }
    if let Some(i) = &m.og_image {
        tag_meta(&mut out, "property", "og:image", i);
    }
    if let Some(r) = m.robots {
        tag_meta(&mut out, "name", "robots", r);
    }
    if let Some(j) = &m.json_ld {
        // JSON-LD 在 <script> 内：转义 < 防数据中出现 </script> 逃逸
        let json = j.to_string().replace('<', "\\u003c");
        out.push_str(&format!(
            "    <script type=\"application/ld+json\" data-seo=\"ssr\">{json}</script>\n"
        ));
    }
    out.push_str("    <!-- /seo:meta -->");
    out
}

/// og:image 等相对地址绝对化：http(s) 原样；`/` 开头拼站点基址；
/// 其余视为 uploads 相对路径（如 fiction/covers/x.webp）。
pub fn absolutize(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if let Some(rest) = url.strip_prefix('/') {
        format!("{base}/{rest}")
    } else {
        format!("{base}/uploads/{url}")
    }
}

/// 富文本 → 摘要：剥标签、解常见实体、折叠空白、按字符数截断。
/// 输入是 ammonia 白名单净化过的 content_html，无需完整 HTML 解析器。
pub fn excerpt(html: &str, max_chars: usize) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    truncate_chars(
        &text.split_whitespace().collect::<Vec<_>>().join(" "),
        max_chars,
    )
}

/// 按字符数截断（多字节安全），超长加省略号。
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// URL 路径段 percent-encode（uid 等自由文本进 canonical/og:url 前用）：
/// 保留 RFC 3986 unreserved 字符，其余按 UTF-8 字节编码。
pub fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
