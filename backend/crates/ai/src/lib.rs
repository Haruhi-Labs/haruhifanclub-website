//! haruhi-ai：DashScope（OpenAI 兼容）内容审核客户端。
//! 行为对齐旧 art/exam 的 ai.js：无 key 或出错时**放行**（fail-open），由业务层据 reason 决策。

use haruhi_core::Config;
use serde_json::{json, Value};

/// art 通用安全审核提示词（文本/图像）。
pub const ART_SYSTEM_PROMPT: &str = "\n你是一个严格的内容安全审核员。你的任务是检测内容中是否包含：色情、暴力、血腥、非法政治、违法犯罪或极度令人反感的内容。\n请以JSON格式返回结果，包含两个字段：\n1. \"safe\": boolean (true表示安全，false表示违规)\n2. \"reason\": string (如果违规，简短说明原因；如果安全，返回\"Pass\")\n不要返回多余的解释，只返回JSON。\n";

/// exam 试卷审核提示词。
pub const EXAM_SYSTEM_PROMPT: &str = "\n你是一个严格的内容审核员。审核用户提交的试卷内容，判断是否包含违禁信息或明显无效的低质量内容。只有在确信命中【安全合规/无意义垃圾/未修改模板/纯粹调试】时才判定不通过；包含具体完整题目即使标题简单也应通过。\n请输出严格 JSON（不要 Markdown 代码块）：{\"pass\": boolean, \"reason\": string}\n";

#[derive(Debug, Clone)]
pub struct Verdict {
    /// 是否放行（safe 或 pass）。
    pub ok: bool,
    pub reason: String,
}

impl Verdict {
    fn pass(reason: &str) -> Self {
        Self { ok: true, reason: reason.to_string() }
    }
}

#[derive(Clone)]
pub struct AiClient {
    api_key: Option<String>,
    base_url: String,
    text_model: String,
    image_model: String,
    http: reqwest::Client,
}

impl AiClient {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            api_key: cfg.dashscope_api_key.clone(),
            base_url: cfg.ai_api_url.trim_end_matches('/').to_string(),
            text_model: cfg.ai_text_model.clone(),
            image_model: cfg.ai_image_model.clone(),
            http: reqwest::Client::new(),
        }
    }

    pub fn is_online(&self) -> bool {
        self.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    /// 审核纯文本。system 传 ART_SYSTEM_PROMPT 或 EXAM_SYSTEM_PROMPT。
    pub async fn check_text(&self, system: &str, text: &str) -> Verdict {
        if !self.is_online() {
            return Verdict::pass("AI_OFFLINE");
        }
        let text = text.trim();
        if text.is_empty() {
            return Verdict::pass("EMPTY");
        }
        let body = json!({
            "model": self.text_model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": format!("请审核以下内容：\n\"{text}\"")}
            ],
            "response_format": {"type": "json_object"}
        });
        self.request(body).await
    }

    /// 审核图片（字节 + mime）。使用视觉模型。
    pub async fn check_image(&self, system: &str, image: &[u8], mime: &str) -> Verdict {
        if !self.is_online() {
            return Verdict::pass("AI_OFFLINE");
        }
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(image);
        let data_url = format!("data:{mime};base64,{b64}");
        let body = json!({
            "model": self.image_model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": [
                    {"type": "image_url", "image_url": {"url": data_url}},
                    {"type": "text", "text": "这张图片是否包含违规内容（色情、暴力、血腥等）？"}
                ]}
            ]
        });
        self.request(body).await
    }

    async fn request(&self, body: Value) -> Verdict {
        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(self.api_key.clone().unwrap_or_default())
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("AI 接口请求失败: {e}");
                return Verdict::pass("AI_API_ERROR");
            }
        };
        if !resp.status().is_success() {
            tracing::warn!("AI 接口返回 {}", resp.status());
            return Verdict::pass("AI_API_ERROR");
        }
        let data: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("AI 响应解析失败: {e}");
                return Verdict::pass("AI_API_ERROR");
            }
        };
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        parse_verdict(&content)
    }
}

/// 从模型返回文本中提取裁决：兼容 {safe} / {pass} 两种字段；解析失败 fail-open。
fn parse_verdict(raw: &str) -> Verdict {
    // 提取第一个 JSON 对象
    let json_str = raw
        .find('{')
        .and_then(|s| raw.rfind('}').map(|e| &raw[s..=e]))
        .unwrap_or(raw);
    match serde_json::from_str::<Value>(json_str) {
        Ok(v) => {
            let ok = v.get("safe").or_else(|| v.get("pass")).and_then(|b| b.as_bool());
            let reason = v
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string();
            match ok {
                Some(ok) => Verdict { ok, reason },
                None => Verdict::pass("AI_PARSE_ERROR"),
            }
        }
        Err(_) => {
            // 非结构化兜底：含安全词则放行，否则判违规（与旧图像逻辑一致）
            let lower = raw.to_lowercase();
            if lower.contains("true") || lower.contains("pass") || lower.contains("安全") {
                Verdict { ok: true, reason: "Pass (Text Analysis)".into() }
            } else if raw.trim().is_empty() {
                Verdict::pass("AI_PARSE_ERROR")
            } else {
                Verdict {
                    ok: false,
                    reason: format!("AI Review (Unstructured): {}", &raw.chars().take(50).collect::<String>()),
                }
            }
        }
    }
}
