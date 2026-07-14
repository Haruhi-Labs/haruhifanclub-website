// @haruhi/api-client 纯逻辑单元测试（vitest）。
// 覆盖被全部 6 个 app 依赖的共享层中、最易出错的两块：
// 上传 URL 拼接（曾出过双前缀 404）与前端 RBAC 镜像（hasScope 向上继承 / hasPerm 向下含子作用域）。
import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  resolveUploadUrl,
  hasScope,
  createAdminAuth,
  createAuth,
  setToken,
  getToken,
  clearToken,
} from './index.js'

describe('resolveUploadUrl', () => {
  it('空值 → 空串', () => {
    expect(resolveUploadUrl('')).toBe('')
    expect(resolveUploadUrl(null)).toBe('')
    expect(resolveUploadUrl(undefined)).toBe('')
  })
  it('绝对 URL / blob / data 原样返回', () => {
    expect(resolveUploadUrl('http://x/y.png')).toBe('http://x/y.png')
    expect(resolveUploadUrl('https://a.b/c.webp')).toBe('https://a.b/c.webp')
    expect(resolveUploadUrl('blob:abc')).toBe('blob:abc')
    expect(resolveUploadUrl('data:image/png;base64,xx')).toBe('data:image/png;base64,xx')
  })
  it('站内绝对路径原样返回', () => {
    expect(resolveUploadUrl('/uploads/a.webp')).toBe('/uploads/a.webp')
    expect(resolveUploadUrl('/other/p.png')).toBe('/other/p.png')
  })
  it('去掉前导 uploads/ 防 /uploads/uploads 双前缀', () => {
    expect(resolveUploadUrl('uploads/art/2025-11/x.webp')).toBe('/uploads/art/2025-11/x.webp')
  })
  it('相对路径拼成 /uploads/<rel>', () => {
    expect(resolveUploadUrl('novel/covers/001.png')).toBe('/uploads/novel/covers/001.png')
    expect(resolveUploadUrl('art/x.webp')).toBe('/uploads/art/x.webp')
  })
  it('自定义 base', () => {
    expect(resolveUploadUrl('a/b.png', '/cdn')).toBe('/cdn/a/b.png')
  })
})

describe('hasScope（拥有作用域或其任一父级或超管即有权）', () => {
  it('null / 无 apps → false', () => {
    expect(hasScope(null, 'news')).toBe(false)
    expect(hasScope(undefined, 'news')).toBe(false)
    expect(hasScope({}, 'news')).toBe(false)
    expect(hasScope({ apps: {} }, 'news')).toBe(false)
  })
  it('超管恒 true', () => {
    expect(hasScope({ isSuperAdmin: true }, 'anything')).toBe(true)
    expect(hasScope({ isSuperAdmin: true, apps: {} }, 'news.activity')).toBe(true)
  })
  it('精确命中', () => {
    expect(hasScope({ apps: { news: {} } }, 'news')).toBe(true)
  })
  it('父作用域继承到子（持有 news → 对 news.activity 有效）', () => {
    expect(hasScope({ apps: { news: {} } }, 'news.activity')).toBe(true)
    expect(hasScope({ apps: { 'a.b': {} } }, 'a.b.c')).toBe(true)
  })
  it('子不授父、兄弟隔离', () => {
    expect(hasScope({ apps: { 'news.activity': {} } }, 'news')).toBe(false)
    expect(hasScope({ apps: { 'news.activity': {} } }, 'news.blog')).toBe(false)
  })
})

describe('createAdminAuth(app).hasPerm（拥有该 app、其任一子作用域或超管即可进后台）', () => {
  const auth = createAdminAuth('news')
  it('null → false', () => {
    expect(auth.hasPerm(null)).toBe(false)
    expect(auth.hasPerm({})).toBe(false)
  })
  it('超管 → true', () => {
    expect(auth.hasPerm({ isSuperAdmin: true })).toBe(true)
  })
  it('精确 app → true', () => {
    expect(auth.hasPerm({ apps: { news: {} } })).toBe(true)
  })
  it('持有任一子作用域 → true（news.activity 可进 news 后台）', () => {
    expect(auth.hasPerm({ apps: { 'news.activity': {} } })).toBe(true)
  })
  it('无关 app → false', () => {
    expect(auth.hasPerm({ apps: { art: {} } })).toBe(false)
  })
})

describe('createAdminAuth(app).buildHeaders', () => {
  const origDocument = globalThis.document

  afterEach(() => {
    if (origDocument === undefined) {
      delete globalThis.document
    } else {
      globalThis.document = origDocument
    }
    clearToken()
  })

  it('无 CSRF cookie 时只保留传入请求头', () => {
    globalThis.document = { cookie: '' }
    expect(createAdminAuth('shop').buildHeaders({ Accept: 'application/json' })).toEqual({
      Accept: 'application/json',
    })
  })

  it('有 CSRF cookie 时给后台裸 fetch 补齐 X-CSRF-Token', () => {
    globalThis.document = { cookie: 'foo=bar; haruhi_csrf=csrf-123; theme=light' }
    expect(createAdminAuth('shop').buildHeaders({ 'Content-Type': 'application/json' })).toEqual({
      'Content-Type': 'application/json',
      'X-CSRF-Token': 'csrf-123',
    })
  })
})

describe('createAdminAuth：无本模块权限不应吊销全站会话', () => {
  const origFetch = globalThis.fetch
  const noNewsPermUser = {
    id: 1,
    username: 'normal',
    displayName: null,
    isSuperAdmin: false,
    apps: { art: { role: 'editor', roleName: '画廊编辑', level: 10 } },
  }

  afterEach(() => {
    globalThis.fetch = origFetch
  })

  it('恢复会话发现无权限时，只返回 null，不调用登出接口', async () => {
    const calls = []
    globalThis.fetch = async (url) => {
      calls.push(String(url))
      if (String(url).endsWith('/auth/me')) {
        return {
          ok: true,
          status: 200,
          text: async () => JSON.stringify(noNewsPermUser),
        }
      }
      if (String(url).endsWith('/auth/logout')) {
        return {
          ok: true,
          status: 200,
          text: async () => '',
        }
      }
      throw new Error(`意外请求：${url}`)
    }

    await expect(createAdminAuth('news').restore()).resolves.toBeNull()
    expect(calls).toEqual(['/api/auth/me'])
  })

  it('后台登录成功但无权限时，只拒绝进入后台，不调用登出接口', async () => {
    const calls = []
    globalThis.fetch = async (url) => {
      calls.push(String(url))
      if (String(url).endsWith('/auth/login')) {
        return {
          ok: true,
          status: 200,
          text: async () => JSON.stringify({ user: noNewsPermUser, token: 'compat-token' }),
        }
      }
      if (String(url).endsWith('/auth/logout')) {
        return {
          ok: true,
          status: 200,
          text: async () => '',
        }
      }
      throw new Error(`意外请求：${url}`)
    }

    await expect(createAdminAuth('news').login('normal', 'pw')).resolves.toEqual({
      ok: false,
      error: '该账号无本模块管理权限',
    })
    expect(calls).toEqual(['/api/auth/login'])
  })
})

describe('createAuth：端用户登录态只走 cookie，不在 localStorage 留兼容 JWT', () => {
  let store
  const origLS = globalThis.localStorage
  const origFetch = globalThis.fetch

  beforeEach(() => {
    store = new Map()
    globalThis.localStorage = {
      getItem: (k) => (store.has(k) ? store.get(k) : null),
      setItem: (k, v) => store.set(k, String(v)),
      removeItem: (k) => store.delete(k),
    }
  })
  afterEach(() => {
    globalThis.localStorage = origLS
    globalThis.fetch = origFetch
  })

  // 模拟后端登录响应（仍带兼容 token，但前端端用户路径不应持久化它）
  function stubAuthResponse(user, token = 'compat-jwt-should-not-be-stored') {
    globalThis.fetch = async () => ({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({ token, user }),
    })
  }

  it('登录成功后不写入兼容 token，且返回用户档案', async () => {
    stubAuthResponse({ id: 1, nickname: '凉宫' })
    const user = await createAuth('/api').login('a@b.com', 'pw12345678')
    expect(user).toEqual({ id: 1, nickname: '凉宫' })
    expect(getToken()).toBe('')
    expect(store.has('haruhi_admin_token')).toBe(false)
  })

  it('登录会清掉历史版本遗留的本地 token（关闭已登录用户的暴露窗口）', async () => {
    setToken('stale-jwt-from-old-version')
    expect(getToken()).toBe('stale-jwt-from-old-version')
    stubAuthResponse({ id: 1, nickname: '凉宫' })
    await createAuth('/api').login('a@b.com', 'pw12345678')
    expect(getToken()).toBe('')
  })

  it('注册与通行密钥登录同样不留 token', async () => {
    setToken('x')
    stubAuthResponse({ id: 2, nickname: '阿虚' })
    const user = await createAuth('/api').register({
      email: 'x@y.com',
      password: 'pw12345678',
      nickname: '阿虚',
    })
    expect(user).toEqual({ id: 2, nickname: '阿虚' })
    expect(getToken()).toBe('')
  })
})
