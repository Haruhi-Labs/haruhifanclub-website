# haruhi-ai

后端 Cargo workspace 的内容审核 crate。封装 **DashScope（阿里云灵积，OpenAI 兼容接口）** 的文本/图像审核调用，供 `art`、`exam` 模块在发布内容前做安全把关。核心约定是 **fail-open（放行优先）**：无 API Key 或调用/解析出错时一律放行，由业务层依据返回的 `reason` 决策，绝不因审核服务不可用而阻断用户。

行为对齐旧站 `art`/`exam` 的 `ai.js`。

## 技术栈 / 关键依赖

- `reqwest` —— 调 `{AI_API_URL}/chat/completions`（OpenAI 兼容端点）。
- `serde` / `serde_json` —— 请求体构造与模型回包解析。
- `base64` —— 图像审核时把字节编码为 `data:<mime>;base64,...` 内联进 `image_url`。
- `tracing` —— 失败仅 `warn!` 记录，不向上抛错。
- `haruhi-core` —— 读取 `Config` 中的 AI 配置。

纯库 crate（仅 `src/lib.rs`，无二进制、无独立可执行）。

## 公开 API

- `AiClient::from_config(&Config)` —— 从配置构造客户端（持有可选 key、base_url、文本/图像模型名、一个复用的 `reqwest::Client`）。
- `AiClient::is_online()` —— key 非空即视为在线。
- `AiClient::check_text(system, text) -> Verdict` —— 审核纯文本，走 `text_model`，请求体带 `response_format: json_object`。空文本直接放行（`reason="EMPTY"`）。
- `AiClient::check_image(system, &[u8], mime) -> Verdict` —— 审核图片，走 `image_model`（视觉模型）。
- `Verdict { ok: bool, reason: String }` —— `ok=true` 表示放行。
- 常量提示词：`ART_SYSTEM_PROMPT`（色情/暴力/血腥/非法等通用安全审核，返回 `{safe, reason}`）、`EXAM_SYSTEM_PROMPT`（试卷违禁/低质审核，返回 `{pass, reason}`）。调用方按模块选用对应提示词。

裁决解析 `parse_verdict` 兼容两种字段：`{safe}`（art）与 `{pass}`（exam）；先从回包中截取第一个 `{...}` JSON 对象再解析；解析失败时走非结构化兜底（含 `true`/`pass`/「安全」等词则放行，空串放行，否则判违规）。

## 关键特性与约定（fail-open）

以下情形 `check_text`/`check_image` 均返回 `ok=true`，仅 `reason` 不同，便于业务层区分「真通过」与「未审核」：

- `AI_OFFLINE` —— 未配置 `DASHSCOPE_API_KEY`。
- `EMPTY` —— 文本去空白后为空。
- `AI_API_ERROR` —— 请求失败 / 非 2xx 状态 / 响应 JSON 解析失败。
- `AI_PARSE_ERROR` —— 回包缺少 `safe`/`pass` 字段，或空内容无法判定。
- `Pass` / 模型给出的 `reason` —— 模型正常裁决（`ok` 由模型决定，可能为 `false`）。

只有「模型明确判定不安全」或「非结构化回包不含安全词」时才返回 `ok=false`。

## 配置（经 haruhi-core 的 Config，环境变量映射）

| 环境变量 | 默认值 | 说明 |
| --- | --- | --- |
| `DASHSCOPE_API_KEY` | 无（离线放行） | API Key，缺省则全部 fail-open |
| `AI_API_URL` | `https://dashscope.aliyuncs.com/compatible-mode/v1` | OpenAI 兼容 base，会拼接 `/chat/completions` |
| `AI_TEXT_MODEL` | `qwen-plus` | 文本审核模型 |
| `AI_IMAGE_MODEL` | `qwen-vl-plus` | 图像审核（视觉）模型 |

## 本地开发

本 crate 随 `haruhi-server` 一起编译，不单独启动。

```bash
# 仅本 crate（含单测：parse_verdict 的字段兼容与兜底）
cargo test -p haruhi-ai

# 在整机后端中验证（:17777）
cargo run -p haruhi-server
```

无 key 时即可跑通：审核全部放行，方便本地开发联调。

## 与共享层 / 后端的关系

- 依赖 `haruhi-core::Config` 取配置。
- 被 `haruhi-server` 的两个模块消费：
  - `art`：发布作品/评论前 `check_text(ART_SYSTEM_PROMPT, ...)`，并对封面 WebP 调 `check_image(..., "image/webp")`，任一 `ok=false` 即拦截。
  - `exam`：试卷提交后 `tokio::spawn` 异步 `check_text(EXAM_SYSTEM_PROMPT, ...)`，据 `verdict.ok` 把状态置为 `published` 或 `locked`。

## 更多

- 根 README：[`../../../README.md`](../../../README.md)
- 贡献指南：[`../../../CONTRIBUTING.md`](../../../CONTRIBUTING.md)
- 新增模块：[`../../../docs/ADDING_MODULE.md`](../../../docs/ADDING_MODULE.md)
