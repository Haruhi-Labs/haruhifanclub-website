//! 音频转码：调用系统 ffmpeg 把任意常见音频转 MP3（192k）。
//! 对齐旧 exam-platform/server/index.js 的 convertToMp3（fluent-ffmpeg → libmp3lame, 192k）。
//! 转码失败时由调用方降级保留原文件。

use std::path::Path;

/// 用系统 ffmpeg 把 input 转码为 192k MP3 写到 output。
/// 失败（ffmpeg 不存在 / 非零退出 / IO 错误）返回 Err，调用方据此降级。
pub async fn transcode_to_mp3(input: &Path, output: &Path) -> anyhow::Result<()> {
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-codec:a",
            "libmp3lame",
            "-b:a",
            "192k",
            &output.to_string_lossy(),
        ])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("启动 ffmpeg 失败: {e}"))?;

    if !status.success() {
        anyhow::bail!("ffmpeg 退出码非零: {status}");
    }
    Ok(())
}
