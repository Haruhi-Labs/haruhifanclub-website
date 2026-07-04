// 展示层工具：时间格式化 + 树的扁平化 / 查找 / 搜索。

/** ISO → YYYY-MM-DD。 */
export function fmtDate(iso) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const p = (x) => String(x).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`
}

/** 相对时间：今天 / x 天前 / x 个月前 / 日期。用于「最近更新」「同步于」等弱提示。 */
export function fmtRelative(iso) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const day = Math.floor((Date.now() - d.getTime()) / 86400000)
  if (day <= 0) return '今天'
  if (day === 1) return '昨天'
  if (day < 30) return `${day} 天前`
  const mon = Math.floor(day / 30)
  if (mon < 12) return `${mon} 个月前`
  return fmtDate(iso)
}

/** 大数字紧凑显示：12345 → 1.2万。 */
export function compact(n) {
  const num = Number(n) || 0
  if (num >= 10000) return `${(num / 10000).toFixed(num % 10000 === 0 ? 0 : 1)}万`
  return String(num)
}

/**
 * 把资源树拍平为条目列表，每条带上所属分类路径。
 * @returns {Array<{...entry, path: string[], top: string}>}
 *   path = 从顶层到该条目所在分类的标题链；top = 顶层分类标题。
 */
export function flattenEntries(tree) {
  const out = []
  const walk = (nodes, path, top) => {
    for (const n of nodes || []) {
      if (n.kind === 'category') {
        walk(n.children, [...path, n.title], top ?? n.title)
      } else if (n.kind === 'entry') {
        out.push({ ...n, path, top: top ?? path[0] ?? '' })
      }
    }
  }
  walk(tree, [], null)
  return out
}

/** 深度优先查找某个分类节点（按 id）。 */
export function findCategory(tree, id) {
  for (const n of tree || []) {
    if (n.kind !== 'category') continue
    if (n.id === id) return n
    const hit = findCategory(n.children, id)
    if (hit) return hit
  }
  return null
}

/**
 * 在扁平条目上做搜索。空格分词，所有词都命中（标题 / 摘要 / 分类路径任一）才算命中；
 * 标题命中权重更高，按分数降序返回。
 */
export function searchEntries(entries, query) {
  const q = (query || '').trim().toLowerCase()
  if (!q) return []
  const tokens = q.split(/\s+/).filter(Boolean)
  const scored = []
  for (const e of entries) {
    const title = (e.title || '').toLowerCase()
    const desc = (e.description || '').toLowerCase()
    const path = (e.path || []).join(' ').toLowerCase()
    let score = 0
    let ok = true
    for (const t of tokens) {
      const inTitle = title.includes(t)
      const inDesc = desc.includes(t)
      const inPath = path.includes(t)
      if (!inTitle && !inDesc && !inPath) {
        ok = false
        break
      }
      if (inTitle) score += title === t ? 100 : title.startsWith(t) ? 40 : 20
      else if (inPath) score += 8
      else score += 4
    }
    if (ok) scored.push({ entry: e, score })
  }
  scored.sort((a, b) => b.score - a.score || (a.entry.title > b.entry.title ? 1 : -1))
  return scored.map((s) => s.entry)
}
