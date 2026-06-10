import { describe, it, expect } from 'vitest'
import { escapeHtml, parseInlineStyles, renderBlockMarkdown } from './inlineMarkdown.js'

describe('inlineMarkdown', () => {
  it('escapeHtml 转义全部特殊字符', () => {
    expect(escapeHtml('<a href="x">&\'')).toBe('&lt;a href=&quot;x&quot;&gt;&amp;&#39;')
  })

  it('parseInlineStyles 渲染内联标记', () => {
    expect(parseInlineStyles('**粗** *斜* __下__ ~~删~~')).toBe(
      '<b>粗</b> <i>斜</i> <u>下</u> <s class="inline-strikethrough">删</s>',
    )
    expect(parseInlineStyles('[名](u)')).toBe(
      '<a href="u" target="_blank" class="inline-link" onclick="event.stopPropagation()">名</a>',
    )
  })

  it('renderBlockMarkdown 先转义再叠加标记，空值安全', () => {
    expect(renderBlockMarkdown('')).toBe('')
    expect(renderBlockMarkdown(null)).toBe('')
    // 用户输入的尖括号被转义，星号标记仍生效
    expect(renderBlockMarkdown('<x> **b**')).toBe('&lt;x&gt; <b>b</b>')
  })
})
