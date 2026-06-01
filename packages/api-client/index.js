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
