// @haruhi/api-client —— 统一 fetch 封装：自动注入 JWT、统一错误处理。
// 各 app 用 createApiClient('/api/<module>') 创建模块客户端；
// 登录/鉴权用 createAuth() 走 /api/auth。

const TOKEN_KEY = 'haruhi_admin_token'

export function getToken() {
  try {
    return localStorage.getItem(TOKEN_KEY) || ''
  } catch {
    return ''
  }
}

export function setToken(token) {
  try {
    if (token) localStorage.setItem(TOKEN_KEY, token)
    else localStorage.removeItem(TOKEN_KEY)
  } catch {
    /* ignore */
  }
}

export function clearToken() {
  setToken('')
}

/**
 * 创建一个 API 客户端。
 * @param {string} base 基础路径，例如 '/api' 或 '/api/novel'
 */
export function createApiClient(base = '/api') {
  async function request(method, path, { body, headers, isForm, signal } = {}) {
    const opts = { method, headers: { ...(headers || {}) }, signal }
    const token = getToken()
    if (token) opts.headers['Authorization'] = `Bearer ${token}`
    if (body !== undefined && body !== null) {
      if (isForm) {
        opts.body = body // FormData，浏览器自动设置 multipart 边界
      } else {
        opts.headers['Content-Type'] = 'application/json'
        opts.body = JSON.stringify(body)
      }
    }
    const res = await fetch(base + path, opts)
    const text = await res.text()
    let data = null
    if (text) {
      try {
        data = JSON.parse(text)
      } catch {
        data = text
      }
    }
    if (!res.ok) {
      const message = (data && data.error) || res.statusText || `HTTP ${res.status}`
      const err = new Error(message)
      err.status = res.status
      err.data = data
      if (res.status === 401) clearToken()
      throw err
    }
    return data
  }

  return {
    base,
    get: (path, opts) => request('GET', path, opts),
    post: (path, body, opts) => request('POST', path, { ...opts, body }),
    put: (path, body, opts) => request('PUT', path, { ...opts, body }),
    patch: (path, body, opts) => request('PATCH', path, { ...opts, body }),
    del: (path, opts) => request('DELETE', path, opts),
    postForm: (path, formData, opts) => request('POST', path, { ...opts, body: formData, isForm: true }),
  }
}

/**
 * 统一鉴权助手（登录/当前用户/登出）。
 * @param {string} apiBase 默认 '/api'
 */
export function createAuth(apiBase = '/api') {
  const api = createApiClient(apiBase)
  return {
    async login(username, password) {
      const r = await api.post('/auth/login', { username, password })
      setToken(r.token)
      return r.user
    },
    async me() {
      return api.get('/auth/me')
    },
    logout() {
      clearToken()
    },
    getToken,
    isLoggedIn: () => !!getToken(),
  }
}

// JWT 本地解码（仅用于同步校验 exp，不验签；验签由后端做）
function decodeJwtPayload(token) {
  if (!token || typeof token !== 'string') return null
  const parts = token.split('.')
  if (parts.length !== 3) return null
  try {
    const b64 = parts[1].replace(/-/g, '+').replace(/_/g, '/')
    const padded = b64 + '='.repeat((4 - (b64.length % 4)) % 4)
    return JSON.parse(atob(padded))
  } catch {
    return null
  }
}

/**
 * 统一后台管理员鉴权（单点 JWT + 按 app 权限校验）。
 * 把"登录并校验本模块权限 / 会话恢复 / 同步 token 校验 / 登出 / 注入鉴权头"
 * 收敛到一处——各站只需 createAdminAuth('news')，不必各写一遍 hasXPerm/restore。
 * @param {string} app 模块名：news|art|exam|novel|shop|console
 * @param {string} apiBase 默认 '/api'
 */
export function createAdminAuth(app, apiBase = '/api') {
  const auth = createAuth(apiBase)

  // 是否具备本模块管理权限：超管或被授予该 app 角色
  const hasPerm = (user) => !!user && (user.isSuperAdmin || (user.apps && user.apps[app]))

  // 同步：本地 token 存在且未过期（供路由守卫快速判断；真正权限由 restore/login 异步校验）
  const hasValidToken = () => {
    const token = getToken()
    if (!token) return false
    const payload = decodeJwtPayload(token)
    if (!payload) {
      clearToken()
      return false
    }
    if (payload.exp && Number(payload.exp) <= Math.floor(Date.now() / 1000)) {
      clearToken()
      return false
    }
    return true
  }

  // 构造带 JWT 的请求头（给仍用裸 fetch/axios 的调用点）
  const buildHeaders = (headers = {}) => {
    const token = getToken()
    return token ? { ...headers, Authorization: `Bearer ${token}` } : { ...headers }
  }

  // 登录 + 校验本模块权限。永不抛错，返回 { ok, user?, error? }
  const login = async (username, password) => {
    try {
      const user = await auth.login(username, password)
      if (!hasPerm(user)) {
        clearToken()
        return { ok: false, error: '该账号无本模块管理权限' }
      }
      return { ok: true, user }
    } catch (e) {
      clearToken()
      return { ok: false, error: e && e.status === 401 ? '用户名或密码错误' : (e && e.message) || '登录失败' }
    }
  }

  // 会话恢复：有有效 token 则拉 me() 并校验权限；任何失败都登出并返回 null
  const restore = async () => {
    if (!hasValidToken()) return null
    try {
      const user = await auth.me()
      if (hasPerm(user)) return user
      clearToken()
      return null
    } catch {
      clearToken()
      return null
    }
  }

  return {
    app,
    hasPerm,
    hasValidToken,
    buildHeaders,
    login,
    restore,
    me: () => auth.me(),
    logout: () => auth.logout(),
    getToken,
    isLoggedIn: () => !!getToken(),
  }
}
