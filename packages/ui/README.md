# packages/ui

预留目录，当前未启用。

现状：

- 没有 `package.json`。
- 任何 app 都不应 import 本目录。
- 没有共享组件、主题 token 或样式入口。

保留原因：

- 现有 app 的视觉和技术栈差异较大：原生 CSS、CSS 变量、SCSS、Tailwind 都在使用。
- 当前复用的是鉴权、请求和上传路径，已经放在 `packages/api-client`。
- 只有出现跨 app 高复用且形态稳定的组件时，再从具体 app 中抽取。

启用前需要先补：

```text
package.json
src/
exports
README 使用说明
```

在启用前，请不要添加 `"@haruhi/ui": "workspace:*"` 依赖。
