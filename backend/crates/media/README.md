# haruhi-media

媒体处理库。供业务模块保存文件、处理图片、解析 EPUB、转码音频。

## 模块

```text
src/
  lib.rs        路径助手和 re-export
  upload.rs     上传字节保存、Base64 图片去重保存
  image_ops.rs  图片 -> WebP
  epub_ops.rs   EPUB 标题、作者、封面
  audio.rs      ffmpeg 音频 -> MP3
```

## 主要 API

顶层助手：

| API                               | 说明                                    |
| --------------------------------- | --------------------------------------- |
| `ensure_dir(dir)`                 | 创建目录                                |
| `ext_of(name, fallback)`          | 取小写扩展名                            |
| `rel_join(module, sub, filename)` | 拼 uploads 相对路径，使用正斜杠         |
| `abs_upload(uploads_root, rel)`   | 从 uploads 根目录和相对路径得到磁盘路径 |

re-export：

| API                                                 | 说明                                              |
| --------------------------------------------------- | ------------------------------------------------- |
| `save_file(dir, filename, bytes)`                   | 写入文件并返回绝对路径                            |
| `save_base64_image(uploads_root, module, data_url)` | 解析 `data:image/...;base64,...`，按 MD5 命名去重 |
| `encode_webp(input, quality)`                       | 图片字节转 WebP                                   |
| `read_epub(path)`                                   | 读取 EPUB 标题、作者和封面                        |

上传校验（供公开匿名上传口防滥用）：

| API                           | 说明                                                                 |
| ----------------------------- | -------------------------------------------------------------------- |
| `check_image(ext, size)`      | 扩展名在图片白名单且 ≤ `MAX_IMAGE_BYTES`（32MB），否则 `UploadReject` |
| `check_media(ext, size)`      | 图片或音频白名单；音频上限 `MAX_AUDIO_BYTES`（64MB）                  |
| `is_image_ext`/`is_audio_ext` | 扩展名是否在对应白名单                                                |

按模块路径调用：

| API                                      | 说明                        |
| ---------------------------------------- | --------------------------- |
| `audio::transcode_to_mp3(input, output)` | 调系统 `ffmpeg` 转 192k MP3 |
| `thumbnail_webp_vips(src, dst, max_w, q)` | 调系统 `vips` 流式生成限宽 WebP 缩略图（内存有界） |

## 约定

- 入库的上传路径使用正斜杠，例如 `novel/covers/a.webp`。
- `save_base64_image()` 返回相对 uploads 根的路径，例如 `news/<md5>.webp`。
- EPUB 解析失败、音频转码失败都由调用方决定如何降级。
- 音频转码在运行时调用系统 `ffmpeg`；缩略图生成调用系统 `vips`（`libvips-tools`）。两者均为子进程，缺失/失败由调用方降级。
- art/exam 等公开匿名上传口在落盘前调用 `check_image`/`check_media`，把 `UploadReject` 转 400；art 仅收图片，exam 仅收图片/音频。

## 使用位置

- novel：EPUB 上传、封面提取。
- art：投稿图片处理。
- news：Base64 图片保存。
- exam：图片和音频上传处理。

HTTP 路由、鉴权、数据库写入和 `/uploads` 静态映射都在 `haruhi-server`。

## 开发

```bash
cargo build -p haruhi-media
cargo test -p haruhi-media
ffmpeg -version
```
