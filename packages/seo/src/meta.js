// 页面 meta 的 DOM 管理核心（纯浏览器端）。
//
// 三方标签协同约定（与后端 seo 模块、各 app index.html 的 <!-- seo:meta --> 标记块配套）：
// - index.html 静态兜底标签带 data-seo="static"；
// - 后端 HTML 注入（整块替换标记块）产出的标签带 data-seo="ssr"；
// - 本模块插入的标签带 data-seo="client"。
// 应用某类标签前，先把同类的 static/ssr 节点摘下暂存，组件卸载时原样放回，
// 保证「详情页 → 无自定义 meta 的页面」回退路径上静态兜底不丢失。
// <title> 不做增删（浏览器只认第一个），只赋值 document.title。

const CLIENT = 'client'
const ROUTE_ROBOTS = 'client-robots'

function metaNamed(name, content) {
  const el = document.createElement('meta')
  el.setAttribute('name', name)
  el.setAttribute('content', content)
  return el
}

function metaProperty(property, content) {
  const el = document.createElement('meta')
  el.setAttribute('property', property)
  el.setAttribute('content', content)
  return el
}

// 每类可管理标签：DOM 选择器 + 节点工厂。og:title/og:description 由 title/description 派生，
// og:url 由 canonical 派生（见 applyPageMeta），故这里不区分来源、只按标签种类管理。
const KINDS = {
  description: {
    selector: 'meta[name="description"]',
    create: (v) => metaNamed('description', v),
  },
  robots: {
    selector: `meta[name="robots"]:not([data-seo="${ROUTE_ROBOTS}"])`,
    create: (v) => metaNamed('robots', v),
  },
  canonical: {
    selector: 'link[rel="canonical"]',
    create: (v) => {
      const el = document.createElement('link')
      el.setAttribute('rel', 'canonical')
      el.setAttribute('href', v)
      return el
    },
  },
  'og:title': { selector: 'meta[property="og:title"]', create: (v) => metaProperty('og:title', v) },
  'og:description': {
    selector: 'meta[property="og:description"]',
    create: (v) => metaProperty('og:description', v),
  },
  'og:type': { selector: 'meta[property="og:type"]', create: (v) => metaProperty('og:type', v) },
  'og:image': { selector: 'meta[property="og:image"]', create: (v) => metaProperty('og:image', v) },
  'og:url': { selector: 'meta[property="og:url"]', create: (v) => metaProperty('og:url', v) },
  jsonld: {
    selector: 'script[type="application/ld+json"]',
    create: (v) => {
      const el = document.createElement('script')
      el.setAttribute('type', 'application/ld+json')
      // JSON-LD 是脚本内容：转义 < 防止数据中出现 </script> 逃逸
      el.textContent = JSON.stringify(v).replaceAll('<', '\\u003c')
      return el
    },
  },
}

// kind -> 被摘下的 static/ssr 原始节点（卸载时放回）。模块级单例：
// 同一时刻页面上至多一个详情视图在管理 meta，无需按组件区分归属。
const stashed = new Map()

function upsertKind(kind, value) {
  const def = KINDS[kind]
  // 首次接管该类标签：摘下 static/ssr 兜底节点并暂存
  if (!stashed.has(kind)) {
    const originals = [...document.head.querySelectorAll(def.selector)].filter(
      (el) => el.dataset.seo !== CLIENT,
    )
    originals.forEach((el) => el.remove())
    stashed.set(kind, originals)
  }
  document.head
    .querySelectorAll(`${def.selector}[data-seo="${CLIENT}"]`)
    .forEach((el) => el.remove())
  const el = def.create(value)
  el.dataset.seo = CLIENT
  document.head.appendChild(el)
}

/**
 * 应用一组页面 meta。只处理有值的字段，未提供的字段保留现有（static/ssr）标签，
 * 这样后端已注入的详情页在前端数据未就绪前不会出现 meta 真空。
 */
export function applyPageMeta(desc) {
  if (!desc) return
  const { title, description, canonical, ogType, ogImage, jsonLd, robots } = desc
  if (title) {
    document.title = title
    upsertKind('og:title', title)
  }
  if (description) {
    upsertKind('description', description)
    upsertKind('og:description', description)
  }
  if (canonical) {
    upsertKind('canonical', canonical)
    upsertKind('og:url', canonical)
  }
  if (ogType) upsertKind('og:type', ogType)
  if (ogImage) upsertKind('og:image', ogImage)
  if (jsonLd) upsertKind('jsonld', jsonLd)
  if (robots) upsertKind('robots', robots)
}

/** 移除本模块插入的 client 标签，并放回先前摘下的 static/ssr 兜底节点。不回滚 document.title（交给路由钩子）。 */
export function resetPageMeta() {
  document.head
    .querySelectorAll(`[data-seo="${CLIENT}"]`)
    .forEach((el) => el.remove())
  for (const originals of stashed.values()) {
    originals.forEach((el) => document.head.appendChild(el))
  }
  stashed.clear()
}

/** 路由级 noindex：独立标签、独立开关，不与 applyPageMeta 的 robots 互相干扰。 */
export function setRouteNoindex(on) {
  const existing = document.head.querySelector(`meta[name="robots"][data-seo="${ROUTE_ROBOTS}"]`)
  if (on && !existing) {
    const el = metaNamed('robots', 'noindex')
    el.dataset.seo = ROUTE_ROBOTS
    document.head.appendChild(el)
  } else if (!on && existing) {
    existing.remove()
  }
}
