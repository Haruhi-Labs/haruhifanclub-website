// 段落内联富文本渲染助手（自 EditorView 原样抽出，便于复用与单测）。
// 顺序约定：先 escapeHtml 再 parseInlineStyles —— 用户文本先转义，再叠加受控标记。

/** 转义 HTML 特殊字符（含引号），防止用户文本破坏结构。 */
export const escapeHtml = (str) =>
  str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

/** 解析内联标记：**粗体** *斜体* __下划线__ ~~删除线~~ 与 [文字](链接)。 */
export const parseInlineStyles = (html) =>
  html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s class="inline-strikethrough">$1</s>')
    // 链接解析
    .replace(
      /\[([^\]]+)\]\(([^)]+)\)/g,
      '<a href="$2" target="_blank" class="inline-link" onclick="event.stopPropagation()">$1</a>',
    )

/** 段落 Markdown 渲染：先转义再叠加内联标记（与原 EditorView 行为一致）。 */
export const renderBlockMarkdown = (text) => parseInlineStyles(escapeHtml(text || ''))
