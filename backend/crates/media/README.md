# haruhi-media

媒体处理工具 crate：图片转 WebP、EPUB 元数据/封面提取、音频转 MP3、文件落盘与 Base64 图片去重保存。被多个业务模块（novel 封面、art/news 图片、exam 音频/缩略图等）复用，替代各旧站里的 sharp / Node `epub` / fluent-ffmpeg。

本 crate 只提供纯函数与异步 IO 助手，不含 HTTP 路由、数据库或鉴权——这些由调用方（各模块的 server 端代码）组装。

## 技术栈与关键依赖

- `image` 0.25：解码常见图片格式（jpeg/png/gif/bmp/webp）。
- `webp` 0.3：WebP 编码。
- `epub` 2：解析 EPUB 元数据与封面。
- `md-5` 0.10：Base64 图片按内容 MD5 命名去重。
- `base64` 0.22：解析 `data:` URL。
- `tokio`：异步文件读写与子进程（ffmpeg）调用。
- `haruhi-core`、`anyhow`、`tracing`：错误与日志的工作区共用约定。

音频转码依赖**系统已安装 `ffmpeg`**（运行时通过子进程调用，非编译期依赖）。

## 目录结构

```
src/
  lib.rs        路径/目录助手 + 各模块的 re-export
  upload.rs     文件落盘、Base64 图片去重保存
  image_ops.rs  图片 → WebP
  epub_ops.rs   EPUB 标题/作者/封面提取
  audio.rs      音频 → MP3（调用 ffmpeg）
```

## 公开 API

`lib.rs` 顶层助手：

- `ensure_dir(dir)` —— 递归建目录。
- `ext_of(name, fallback)` —— 取小写扩展名（不含点），缺省回退。
- `rel_join(module, sub, filename)` —— 拼 uploads 子路径，统一用正斜杠（入库格式）。
- `abs_upload(uploads_root, rel)` —— 由 uploads 根 + 库中相对路径还原磁盘绝对路径。

re-export 的模块函数：

- `encode_webp(input, quality)`（`image_ops`）—— 字节 → WebP，`quality` 0–100，对齐 sharp 的 `webp({quality})`。
- `read_epub(path) -> EpubInfo`（`epub_ops`）—— 提取 `title` / `author` / `cover`（封面为 `(字节, mime)`）；解析失败返回 `Err`，由调用方降级。
- `save_file(dir, filename, bytes)`（`upload`）—— 建目录并写入，返回绝对路径。
- `save_base64_image(uploads_root, module, data_url)`（`upload`）—— 解析 `data:image/...;base64,...`，按内容 MD5 命名落到 `<module>/<md5>.<ext>`，返回相对 uploads 根的路径；与 news 旧行为一致，天然去重。

未经 re-export、需按模块路径调用：

- `audio::transcode_to_mp3(input, output)` —— 调用系统 `ffmpeg` 转 192k MP3（`libmp3lame`）；ffmpeg 缺失 / 非零退出 / IO 错误均返回 `Err`，调用方据此降级保留原文件。

## 关键约定

- **路径分隔符**：入库的相对路径一律正斜杠（见 `rel_join`），便于跨平台与 URL 拼接；`abs_upload` 负责还原磁盘路径。
- **图片去重**：`save_base64_image` 以字节 MD5 作文件名，相同内容只存一份。仅识别 png/jpeg/webp/gif/svg，其余 mime 落 `.bin`。
- **降级策略**：EPUB 解析、音频转码失败都设计为不致命，由各模块的 server 代码捕获 `Err` 后降级（保留原文件 / 跳过封面）。

## 本地开发

随后端工作区一起构建，无独立二进制：

```bash
cargo build -p haruhi-media     # 仅构建本 crate
cargo test -p haruhi-media      # 跑本 crate 测试
```

音频相关功能本地验证需先装好 `ffmpeg`（`ffmpeg -version` 可用）。本 crate 不监听端口、不暴露 `/api` 或 `/uploads`——实际路由与上传目录（`uploads/<module>/`）由 `haruhi-server` 装配。

## 与后端其他层的关系

- 被 novel / art / news / exam 等模块的 server 端代码直接调用，集中处理媒体落盘与转码。
- uploads 根目录、`/uploads/<module>/*` 静态映射、优雅停机等由 `haruhi-server` 提供；本 crate 只负责字节进出磁盘与格式转换。
- 错误类型沿用 `anyhow`，由调用方转成 `haruhi-core` 的统一 `AppError` JSON。

## 更多

- 后端整体与模块约定见根 [README](../../../README.md)。
- 新增模块的端到端模板与媒体接入方式见 `docs/ADDING_MODULE.md`（以 novel 为参考）。
- 协作与提交规范见根 [CONTRIBUTING](../../../CONTRIBUTING.md)。
