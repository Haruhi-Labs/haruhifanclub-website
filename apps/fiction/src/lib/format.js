// 展示层工具：分类映射、字数、阅读时长、时间格式化。

// 与后端 fiction.rs CATEGORIES 保持一致
export const CATEGORIES = [
  { slug: 'daily', label: '日常' },
  { slug: 'romance', label: '恋爱' },
  { slug: 'adventure', label: '冒险' },
  { slug: 'parallel', label: '平行世界' },
  { slug: 'fantasy', label: '幻想' },
  { slug: 'mystery', label: '推理' },
  { slug: 'comedy', label: '欢乐向' },
]

export function categoryLabel(slug) {
  return CATEGORIES.find((c) => c.slug === slug)?.label || '日常'
}

/** 大数字紧凑显示：12345 → 1.2万 */
export function compact(n) {
  const num = Number(n) || 0
  if (num >= 10000) return `${(num / 10000).toFixed(num % 10000 === 0 ? 0 : 1)}万`
  return String(num)
}

/** 中文字数按 ~400 字/分钟估算阅读时长（至少 1 分钟）。 */
export function readingMinutes(words) {
  return Math.max(1, Math.round((Number(words) || 0) / 400))
}

/** 字数友好展示：万字以上取「x.x 万字」。 */
export function wordLabel(words) {
  const n = Number(words) || 0
  if (n >= 10000) return `${(n / 10000).toFixed(1)} 万字`
  return `${n} 字`
}

/** ISO → YYYY-MM-DD。 */
export function fmtDate(iso) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const p = (x) => String(x).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`
}

/** 相对时间：刚刚 / x 分钟前 / x 小时前 / x 天前 / 日期。 */
export function fmtRelative(iso) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const diff = Date.now() - d.getTime()
  const min = Math.floor(diff / 60000)
  if (min < 1) return '刚刚'
  if (min < 60) return `${min} 分钟前`
  const h = Math.floor(min / 60)
  if (h < 24) return `${h} 小时前`
  const day = Math.floor(h / 24)
  if (day < 30) return `${day} 天前`
  return fmtDate(iso)
}
