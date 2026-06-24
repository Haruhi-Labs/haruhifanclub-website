// 链接协议白名单：允许相对地址与 http(s) / mailto / tel，拦截 javascript: / data: 等可执行协议。
// 先剥离控制字符/空白再判定 scheme，避免 `java\tscript:` 之类的混淆绕过。
export function safeUrl(url?: string | null): string | undefined {
  if (url == null) return undefined
  let cleaned = ''
  for (const ch of String(url)) {
    const code = ch.charCodeAt(0)
    if (code <= 0x20 || code === 0x7f) continue
    cleaned += ch
  }
  if (!cleaned) return undefined
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(cleaned)
  if (scheme && !['http', 'https', 'mailto', 'tel'].includes(scheme[1].toLowerCase())) {
    return undefined
  }
  return cleaned
}
