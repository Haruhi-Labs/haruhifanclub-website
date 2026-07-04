import { setRouteNoindex } from './meta.js'

/**
 * 路由级默认 title 与 noindex。
 *
 * - `to.meta.title`：字符串或 `(to) => string`，命中时 document.title = `${title} · ${siteName}`；
 *   未定义时回落 defaultTitle（通常与 index.html 静态 <title> 一致）。
 *   详情页在数据加载后用 usePageMeta 覆盖，比这里更晚生效，优先级天然正确。
 * - `to.matched` 任一层 `meta.noindex: true` 时插入路由级 noindex 标签，离开时移除。
 */
export function installRouterMeta(router, { siteName, defaultTitle } = {}) {
  const fallback = defaultTitle || (typeof document !== 'undefined' ? document.title : '')
  router.afterEach((to) => {
    const raw = to.meta?.title
    const title = typeof raw === 'function' ? raw(to) : raw
    document.title = title ? (siteName ? `${title} · ${siteName}` : title) : fallback
    setRouteNoindex(to.matched.some((r) => r.meta?.noindex))
  })
}
