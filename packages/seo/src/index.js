import { getCurrentInstance, onUnmounted, unref, watchEffect } from 'vue'
import { applyPageMeta, resetPageMeta } from './meta.js'

export { installRouterMeta } from './router.js'
export { applyPageMeta, resetPageMeta, setRouteNoindex } from './meta.js'
export { canonicalUrl, absoluteUrl } from './urls.js'

/**
 * 声明当前视图的页面 meta（title/description/canonical/ogType/ogImage/jsonLd/robots）。
 *
 * source 可以是 plain object、ref，或返回对象的 getter（推荐：依赖变化自动重应用）。
 * 返回 null/undefined 时什么都不做——详情页应在数据加载完成后才返回对象，
 * 加载前保留后端注入（data-seo="ssr"）或静态兜底标签。
 * 组件卸载时移除 client 标签并放回兜底节点。
 */
export function usePageMeta(source) {
  const read = typeof source === 'function' ? source : () => unref(source)
  const stop = watchEffect(() => {
    applyPageMeta(read())
  })
  if (getCurrentInstance()) {
    onUnmounted(() => {
      stop()
      resetPageMeta()
    })
  }
  return stop
}

/** 快捷方式：当前视图 noindex（登录页、编辑器等不该被收录又不便挂在路由 meta 上时用）。 */
export function useNoindex() {
  return usePageMeta({ robots: 'noindex' })
}
