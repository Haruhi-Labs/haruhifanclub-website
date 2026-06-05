// 控制台共享层：API 客户端、登录会话、管理数据 store、轻量 toast。
import { reactive } from 'vue'
import { createApiClient, createAuth, type CurrentUser } from '@haruhi/api-client'

export const api = createApiClient('/api')
export const auth = createAuth('/api')

export interface AdminUser {
  id: number
  username: string
  displayName: string | null
  email: string | null
  isSuperAdmin: boolean
  status: string
  createdAt: string
  lastLoginAt: string | null
  roles: Record<string, string>
}
export interface RoleDef {
  key: string
  name: string
  level: number
}
export interface AuditItem {
  id: number
  userId: number | null
  app: string | null
  action: string
  target: string | null
  createdAt: string
}

// 模块中文名（含 news 的子作用域）
export const APP_NAMES: Record<string, string> = {
  news: '新闻',
  art: '画廊',
  exam: '考试',
  novel: '书库',
  shop: '商城',
  console: '控制台',
  'news.blog': '团报',
  'news.activity': '活动',
  'news.store': '积分商城',
  'news.points': '积分',
}
export function appName(a: string): string {
  return APP_NAMES[a] || a
}

// 有 AI 内容审核、会触发邮件通知的模块
export const AI_MODULES = ['art', 'exam']

// 登录会话
export const session = reactive<{ me: CurrentUser | null }>({ me: null })

// 管理数据共享 store（用户/角色/可分配 app），各页面共用
export const store = reactive({
  users: [] as AdminUser[],
  apps: [] as string[],
  roles: [] as RoleDef[],
  loaded: false,
  loading: false,
})

export async function refreshAdmin(): Promise<void> {
  store.loading = true
  try {
    const [u, r] = await Promise.all([api.get('/admin/users'), api.get('/admin/roles')])
    store.users = u.users
    store.apps = r.apps
    store.roles = r.roles
    store.loaded = true
  } finally {
    store.loading = false
  }
}

// 轻量 toast
export const toast = reactive<{ msg: string; kind: 'ok' | 'err' }>({ msg: '', kind: 'ok' })
let toastTimer: ReturnType<typeof setTimeout> | undefined
export function flash(msg: string, kind: 'ok' | 'err' = 'ok'): void {
  toast.msg = msg
  toast.kind = kind
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (toast.msg = ''), 2800)
}

export function errMsg(e: unknown): string {
  return (e as { message?: string })?.message || '操作失败'
}
