//! dist/index.html 模板：按 mtime 缓存加载 + 标记块注入。
//!
//! 模板来自 `{apps_dir}/{app}/dist/index.html`（Nginx 平时直接静态服务同一文件，
//! 仅内容详情页路径转发到这里）。mtime 变化即自动重读——部署新 dist 无需重启后端。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

use crate::state::AppState;

/// app 名（目录名，如 "fiction"）→ (dist mtime, 模板内容)。
pub type SeoTemplates = Arc<RwLock<HashMap<&'static str, (SystemTime, Arc<String>)>>>;

pub fn new_cache() -> SeoTemplates {
    Arc::new(RwLock::new(HashMap::new()))
}

const MARK_START: &str = "<!-- seo:meta -->";
const MARK_END: &str = "<!-- /seo:meta -->";

/// 读取某 app 的模板；读不到（dev 未 build / 部署路径错）返回 None，
/// 调用方回 500 由 Nginx error_page 兜底静态壳。
pub async fn load(state: &AppState, app: &'static str) -> Option<Arc<String>> {
    let path = state.cfg.apps_dir.join(app).join("dist/index.html");
    let mtime = match tokio::fs::metadata(&path).await.and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(app, path = %path.display(), error = %e, "SEO 注入：读不到 dist/index.html");
            return None;
        }
    };
    {
        let cache = state.seo_templates.read().await;
        if let Some((t, html)) = cache.get(app) {
            if *t == mtime {
                return Some(html.clone());
            }
        }
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(html) => {
            let html = Arc::new(html);
            state
                .seo_templates
                .write()
                .await
                .insert(app, (mtime, html.clone()));
            Some(html)
        }
        Err(e) => {
            tracing::warn!(app, path = %path.display(), error = %e, "SEO 注入：模板读取失败");
            None
        }
    }
}

/// 标记块整块替换为 head_block；旧 dist 无标记块时在 </head> 前兜底插入
/// （此时会出现双 title，Google 取首个，属部署顺序问题的降级路径）。
pub fn inject(template: &str, head_block: &str) -> String {
    if let (Some(s), Some(e)) = (template.find(MARK_START), template.find(MARK_END)) {
        if s < e {
            let mut out = String::with_capacity(template.len() + head_block.len());
            out.push_str(&template[..s]);
            out.push_str(head_block);
            out.push_str(&template[e + MARK_END.len()..]);
            return out;
        }
    }
    tracing::warn!("SEO 注入：模板缺少 seo:meta 标记块，走 </head> 前兜底插入");
    let lower = template.to_ascii_lowercase();
    if let Some(pos) = lower.find("</head>") {
        let mut out = String::with_capacity(template.len() + head_block.len());
        out.push_str(&template[..pos]);
        out.push_str(head_block);
        out.push('\n');
        out.push_str(&template[pos..]);
        return out;
    }
    template.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_replaces_marker_block() {
        let tpl = "<head>\n<!-- seo:meta -->\n<title>旧</title>\n<!-- /seo:meta -->\n</head>";
        let out = inject(
            tpl,
            "<!-- seo:meta -->\n<title>新</title>\n<!-- /seo:meta -->",
        );
        assert!(out.contains("<title>新</title>"));
        assert!(!out.contains("<title>旧</title>"));
        assert_eq!(out.matches("seo:meta").count(), 2);
    }

    #[test]
    fn inject_falls_back_before_head_close() {
        let tpl = "<head><title>旧</title></head>";
        let out = inject(tpl, "<title>新</title>");
        assert!(
            out.contains("<title>新</title></head>") || out.contains("<title>新</title>\n</head>")
        );
        // 兜底路径不移除旧 title（双 title 降级可接受）
        assert!(out.contains("<title>旧</title>"));
    }
}
