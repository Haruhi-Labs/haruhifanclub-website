export const UI = {
  theme: {
    // 主色：青绿（你可以随时改这里）
    accentHue: 172
  },
  sfx: {
    enabled: true,
    volume: 0.35,
    /**
     * 自定义点击音效（可选）：
     * 1) 把音效文件放到 /public/sfx/click.mp3
     * 2) 把这里改成 '/sfx/click.mp3'
     * 留空则使用内置“轻量合成点击声”（不需要任何资源文件）
     */
    clickUrl: ''
  }
}
