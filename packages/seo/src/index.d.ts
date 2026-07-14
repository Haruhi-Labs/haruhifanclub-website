import type { Ref } from 'vue'
import type { Router } from 'vue-router'

export interface PageMetaDescriptor {
  /** 完整页面标题（不追加站名后缀，调用方自行拼接） */
  title?: string
  description?: string
  /** 规范 URL（绝对地址）；同时派生 og:url */
  canonical?: string
  /** og:type，如 'article' / 'book' / 'product'，默认不输出 */
  ogType?: string
  /** og:image，必须是绝对 URL */
  ogImage?: string
  /** JSON-LD 对象，序列化时自动防 </script> 逃逸 */
  jsonLd?: Record<string, unknown>
  /** robots 指令，如 'noindex' */
  robots?: string
}

export type PageMetaSource =
  | PageMetaDescriptor
  | null
  | undefined
  | Ref<PageMetaDescriptor | null | undefined>
  | (() => PageMetaDescriptor | null | undefined)

/** 声明当前视图的页面 meta；返回 watchEffect 的停止函数。 */
export function usePageMeta(_source: PageMetaSource): () => void

/** 快捷方式：当前视图 noindex。 */
export function useNoindex(): () => void

export interface RouterMetaOptions {
  /** 站名后缀，如 '春日文库'；route meta.title 会拼成 `${title} · ${siteName}` */
  siteName?: string
  /** 无 route title 时的兜底完整标题；缺省取安装时的 document.title */
  defaultTitle?: string
}

/** 安装路由级默认 title 与 meta.noindex 处理。 */
export function installRouterMeta(_router: Router, _options?: RouterMetaOptions): void

/** 由 app 内路由路径构造规范 URL（origin + Vite base + path）。 */
export function canonicalUrl(_path: string): string

/** 相对地址绝对化；已是绝对地址则原样返回。 */
export function absoluteUrl(_url: string | null | undefined): string | null | undefined

/** 直接应用/清除页面 meta（组件外场景用；组件内优先 usePageMeta）。 */
export function applyPageMeta(_desc: PageMetaDescriptor | null | undefined): void
export function resetPageMeta(): void
export function setRouteNoindex(_on: boolean): void
