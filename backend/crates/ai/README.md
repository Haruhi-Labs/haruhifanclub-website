# haruhi-ai

内容审核客户端。封装 DashScope/OpenAI 兼容 `/chat/completions` 调用，供 `art` 和 `exam` 模块使用。

## 行为

- 未配置 `DASHSCOPE_API_KEY` 时返回通过，`reason = "AI_OFFLINE"`。
- 请求失败、非 2xx、响应解析失败时返回通过，`reason = "AI_API_ERROR"`。
- 回包缺少 `safe` 或 `pass` 字段时返回通过，`reason = "AI_PARSE_ERROR"`。
- 模型明确返回不安全时才 `ok = false`。

这套策略会避免审核服务不可用时阻断用户操作。业务模块仍可根据 `reason` 做展示或记录。

## 主要 API

| API                                | 说明                    |
| ---------------------------------- | ----------------------- |
| `AiClient::from_config(&Config)`   | 从环境配置创建客户端    |
| `AiClient::is_online()`            | 是否配置了 API key      |
| `check_text(system, text)`         | 文本审核                |
| `check_image(system, bytes, mime)` | 图片审核                |
| `Verdict { ok, reason }`           | 审核结果                |
| `ART_SYSTEM_PROMPT`                | art 文本/图片审核提示词 |
| `EXAM_SYSTEM_PROMPT`               | exam 试卷审核提示词     |

`parse_verdict` 兼容 `{ "safe": boolean }` 和 `{ "pass": boolean }`，并会从模型返回文本中截取第一个 JSON 对象。

## 配置

| 变量                | 默认值                                              | 说明                                 |
| ------------------- | --------------------------------------------------- | ------------------------------------ |
| `DASHSCOPE_API_KEY` | 空                                                  | 空值时离线放行                       |
| `AI_API_URL`        | `https://dashscope.aliyuncs.com/compatible-mode/v1` | base URL，会拼接 `/chat/completions` |
| `AI_TEXT_MODEL`     | `qwen-plus`                                         | 文本模型                             |
| `AI_IMAGE_MODEL`    | `qwen-vl-plus`                                      | 图像模型                             |

## 开发

```bash
cargo test -p haruhi-ai
cargo build -p haruhi-ai
```

本 crate 没有 HTTP 路由，也不单独运行。实际调用点在 `haruhi-server` 的 art/exam 模块。
