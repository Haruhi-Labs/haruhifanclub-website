// @haruhi/api-client —— 统一 fetch 封装。
// 鉴权已升级为「服务端会话 + httpOnly cookie」：请求一律带 credentials，写操作自动注入 CSRF 头。
// 端用户登录态只认 cookie，不再把兼容 JWT 存 localStorage（避免 XSS 暴露面）。
// localStorage Bearer（setToken/getToken）现仅供尚未切 cookie 的旧管理员路径（createAdminAuth）使用。
// 各 app 用 createApiClient('/api/<module>') 创建模块客户端；账号相关用 createAuth() 走 /api/auth。

import {
  createCredential,
  getCredential,
  isPasskeySupported,
  isConditionalUiAvailable,
} from './webauthn.js'

const TOKEN_KEY = 'haruhi_admin_token'
const CSRF_COOKIE = 'haruhi_csrf'
// 仅这些方法会改状态，需带 CSRF 双提交头
const UNSAFE_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE'])

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

/** 读取可读的 CSRF cookie（haruhi_csrf，非 httpOnly），用于回填 X-CSRF-Token。 */
export function getCsrfToken() {
  try {
    const m = document.cookie.match(/(?:^|;\s*)haruhi_csrf=([^;]+)/)
    return m ? decodeURIComponent(m[1]) : ''
  } catch {
    return ''
  }
}

/** 是否存在会话 cookie 的迹象（csrf cookie 可读 → 大概率已登录）。供同步路由守卫快速判断。 */
export function hasSessionCookie() {
  return !!getCsrfToken()
}

/**
 * 把后端返回的上传资源路径解析为可访问 URL（统一各 app 此前各写一份的
 * art `fixPath` / novel `${ASSET_BASE}/${path}` 拼接）。语义取自 art 的 fixPath，
 * novel 等更简单的 `/uploads/<path>` 拼接是其子集：
 * - 空值 → ''；
 * - 绝对 URL（http/https）、blob:、data: → 原样返回；
 * - 站内绝对路径（以 '/' 开头）→ 原样返回；
 * - 否则去掉可能的前导 'uploads/' 再拼 `<base>/<rel>`，避免 /uploads/uploads/... 双前缀。
 * @param {string} path 后端字段，如 'novel/covers/1.png' 或 'uploads/art/2024-01/x.webp'
 * @param {string} [base] 资源根，默认 '/uploads'
 * @returns {string}
 */
export function resolveUploadUrl(path, base = '/uploads') {
  if (!path) return ''
  if (path.startsWith('http') || path.startsWith('blob:') || path.startsWith('data:')) return path
  if (path.startsWith('/')) return path
  const rel = path.startsWith('uploads/') ? path.slice('uploads/'.length) : path
  return `${base}/${rel}`
}

/**
 * 创建一个 API 客户端。
 * @param {string} base 基础路径，例如 '/api' 或 '/api/novel'
 */
export function createApiClient(base = '/api') {
  async function request(method, path, { body, headers, isForm, signal } = {}) {
    // credentials:'include' → 浏览器自动带上 httpOnly 会话 cookie（同源/已配 CORS credentials 的跨域）
    const opts = { method, headers: { ...(headers || {}) }, signal, credentials: 'include' }
    const token = getToken()
    if (token) opts.headers['Authorization'] = `Bearer ${token}`
    // 写方法注入 CSRF 双提交头（与 haruhi_csrf cookie 比对）
    if (UNSAFE_METHODS.has(method)) {
      const csrf = getCsrfToken()
      if (csrf) opts.headers['X-CSRF-Token'] = csrf
    }
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
    postForm: (path, formData, opts) =>
      request('POST', path, { ...opts, body: formData, isForm: true }),
  }
}

/**
 * 统一账号助手（注册/登录/登出/邮箱验证/找回密码/资料/会话管理）。
 * 登录态完全以服务端会话 + httpOnly cookie 为准；端用户不再把兼容 JWT 存进
 * localStorage——那份 JS 可读、有 XSS 暴露面，且 cookie 路径已是各 app 主路径。
 * 路由守卫的「是否已登录」判断走可读的 csrf cookie（hasSessionCookie），不依赖 token。
 * @param {string} apiBase 默认 '/api'
 */
export function createAuth(apiBase = '/api') {
  const api = createApiClient(apiBase)
  // 取出用户档案，同时清掉历史版本可能遗留在 localStorage 的兼容 token，
  // 关闭已登录用户残留 JWT 的暴露窗口；后续端用户登录态只认 cookie。
  const takeUser = (r) => {
    clearToken()
    return r && r.user
  }
  return {
    // 终端用户注册：{ email, password, nickname? }
    async register(payload) {
      return takeUser(await api.post('/auth/register', payload))
    },
    // 登录：account 可为邮箱或用户名（兼容旧 login(username, password) 调用）。
    // 若账号启用了两步验证，后端返回 { twoFactorRequired, pendingToken }，此时原样返回，
    // 由调用方跳转二次验证；否则返回 CurrentUser。
    async login(account, password) {
      const r = await api.post('/auth/login', { account, password })
      if (r && r.twoFactorRequired) {
        return { twoFactorRequired: true, pendingToken: r.pendingToken }
      }
      return takeUser(r)
    },
    // 两步验证二次校验（凭 pendingToken）：成功返回 CurrentUser
    async login2fa(pendingToken, code, backup = false) {
      return takeUser(await api.post('/auth/2fa/login', { pendingToken, code, backup }))
    },
    // 2FA 管理（需登录）
    setup2fa() {
      return api.post('/auth/2fa/setup')
    },
    enable2fa(code) {
      return api.post('/auth/2fa/enable', { code })
    },
    disable2fa(password) {
      return api.post('/auth/2fa/disable', { password })
    },
    me() {
      return api.get('/auth/me')
    },
    // 登出：调服务端吊销会话 + 清本地兼容 token
    async logout() {
      try {
        await api.post('/auth/logout')
      } catch {
        /* 即便服务端失败也清本地 */
      }
      clearToken()
    },
    forgotPassword(email) {
      return api.post('/auth/forgot-password', { email })
    },
    resetPassword(token, password) {
      return api.post('/auth/reset-password', { token, password })
    },
    verifyEmail(token) {
      return api.post('/auth/verify-email', { token })
    },
    resendVerification() {
      return api.post('/auth/resend-verification')
    },
    updateProfile(patch) {
      return api.patch('/auth/profile', patch)
    },
    // 上传头像（File/Blob）：服务端裁正方形 + 转 WebP，返回更新后的用户档案。
    uploadAvatar(file) {
      const fd = new FormData()
      fd.append('avatar', file, (file && file.name) || 'avatar.webp')
      return api.postForm('/auth/avatar', fd)
    },
    // 移除头像：恢复为昵称首字默认展示，返回更新后的用户档案。
    removeAvatar() {
      return api.del('/auth/avatar')
    },
    changePassword(oldPassword, newPassword) {
      return api.post('/auth/change-password', { oldPassword, newPassword })
    },
    listSessions() {
      return api.get('/auth/sessions')
    },
    revokeSession(id) {
      return api.del(`/auth/sessions/${encodeURIComponent(id)}`)
    },

    // ---------- 通行密钥（Passkey / WebAuthn） ----------
    isPasskeySupported,
    isConditionalUiAvailable,
    listPasskeys() {
      return api.get('/auth/passkeys')
    },
    deletePasskey(id) {
      return api.del(`/auth/passkeys/${encodeURIComponent(id)}`)
    },
    renamePasskey(id, name) {
      return api.patch(`/auth/passkeys/${encodeURIComponent(id)}`, { name })
    },
    // 注册一把新通行密钥（需已登录）：start → 系统验证器 → finish
    async addPasskey(name) {
      const { flowId, options } = await api.post('/auth/passkey/register/start')
      const credential = await createCredential(options.publicKey)
      return api.post('/auth/passkey/register/finish', {
        flowId,
        name: name || undefined,
        credential,
      })
    },
    // 用通行密钥登录（无用户名 / discoverable）。conditional=true 走条件式自动填充。
    async loginPasskey({ conditional = false, signal } = {}) {
      const { flowId, options } = await api.post('/auth/passkey/login/start')
      const credential = await getCredential(options.publicKey, { conditional, signal })
      return takeUser(await api.post('/auth/passkey/login/finish', { flowId, credential }))
    },

    getToken,
    isLoggedIn: () => !!getToken() || hasSessionCookie(),
  }
}

/**
 * 前端作用域判定（镜像后端 authorize 的层级逻辑）：用于细粒度门控（如按 news.activity 显隐后台 tab）。
 * 拥有该作用域、其任一祖先(父级)、或超管，均视为有权。
 * @param {{isSuperAdmin?:boolean, apps?:Record<string,unknown>}|null} user
 * @param {string} scope 如 'news.activity'
 */
export function hasScope(user, scope) {
  if (!user) return false
  if (user.isSuperAdmin) return true
  if (!user.apps) return false
  let cur = scope
  while (cur) {
    if (user.apps[cur]) return true
    const i = cur.lastIndexOf('.')
    if (i < 0) break
    cur = cur.slice(0, i)
  }
  return false
}

/**
 * 前端权限级别判定（镜像后端 authorize 的层级比较）：scope 或其任一祖先的 level ≥ minLevel，
 * 或超管，即视为有权。级别：Read=1 / Write=2 / Moderate=3 / Manage=4。
 * @param {{isSuperAdmin?:boolean, apps?:Record<string,{level?:number}>}|null} user
 * @param {string} scope 如 'news.store'
 * @param {number} minLevel 所需最低级别
 */
export function hasLevel(user, scope, minLevel) {
  if (!user) return false
  if (user.isSuperAdmin) return true
  if (!user.apps) return false
  let cur = scope
  while (cur) {
    const r = user.apps[cur]
    if (r && typeof r.level === 'number' && r.level >= minLevel) return true
    const i = cur.lastIndexOf('.')
    if (i < 0) break
    cur = cur.slice(0, i)
  }
  return false
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
 * 统一后台管理员鉴权（会话 cookie 优先；兼容旧 localStorage token）。
 * 把"登录并校验本模块权限 / 会话恢复 / 同步守卫判断 / 登出 / 注入鉴权头"
 * 收敛到一处——各站只需 createAdminAuth('news')，不必各写一遍 hasXPerm/restore。
 * @param {string} app 模块名：news|art|exam|novel|shop|console
 * @param {string} apiBase 默认 '/api'
 */
export function createAdminAuth(app, apiBase = '/api') {
  const auth = createAuth(apiBase)

  // 是否可进入本模块后台：超管 / 被授予该 app 角色 / 被授予该 app 任一子作用域(如 news.activity)
  const hasPerm = (user) => {
    if (!user) return false
    if (user.isSuperAdmin) return true
    if (!user.apps) return false
    if (user.apps[app]) return true
    const prefix = app + '.'
    return Object.keys(user.apps).some((k) => k.startsWith(prefix))
  }

  // 同步快速判断（供路由守卫先放行，真正权限由 restore/login 异步校验）：
  // 有会话 cookie 迹象，或本地 token 未过期，都视为「可能已登录」。
  const hasValidToken = () => {
    if (hasSessionCookie()) return true
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
        await auth.logout()
        return { ok: false, error: '该账号无本模块管理权限' }
      }
      return { ok: true, user }
    } catch (e) {
      clearToken()
      return {
        ok: false,
        error: e && e.status === 401 ? '用户名或密码错误' : (e && e.message) || '登录失败',
      }
    }
  }

  // 会话恢复：以服务端 me() 为权威（cookie 可能有效即便本地 token 已过期）；失败则登出并返回 null
  const restore = async () => {
    try {
      const user = await auth.me()
      if (hasPerm(user)) return user
      await auth.logout()
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
    isLoggedIn: () => !!getToken() || hasSessionCookie(),
  }
}
