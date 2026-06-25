// 个人控制台上下文：UserConsoleLayout 经 provide 下发，子页用 useConsoleContext 读取，
// 免去各 app 在每个子路由重复传 apiBase / site / basePath / loginPath。
import { inject } from 'vue'

export const CONSOLE_CTX = Symbol('hauth-console')

// 未在 layout 内时的兜底（子页也可独立挂载）。
const FALLBACK = {
  apiBase: '/api',
  site: undefined,
  basePath: '/account',
  loginPath: '/login',
  home: '/',
}

export function useConsoleContext() {
  return inject(CONSOLE_CTX, FALLBACK)
}
