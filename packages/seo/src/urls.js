/**
 * 由「app 内路由路径」构造规范 URL：origin + Vite base + path。
 * 例（fiction，base=/novel/）：canonicalUrl('/story/1') → https://haruyuki.cn/novel/story/1
 * dev 下 origin 是 localhost，无碍——canonical 只在生产环境被爬虫消费。
 */
export function canonicalUrl(path) {
  const base = (import.meta.env?.BASE_URL || '/').replace(/\/$/, '')
  return new URL(base + (path.startsWith('/') ? path : `/${path}`), window.location.origin).href
}

/** 相对地址绝对化（og:image 等要求绝对 URL 的场合）；已是绝对地址则原样返回。 */
export function absoluteUrl(url) {
  if (!url) return url
  return new URL(url, window.location.origin).href
}
