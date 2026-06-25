// 无头登录态组合式：全站共享单例（类似 pinia store），各 app 自带 UI。
// 用法：
//   import { useSession } from '@haruhi/auth-ui'
//   const session = useSession('/api')
//   onMounted(() => session.refresh())   // 首次拉取当前用户（带 cookie）
//   session.isLoggedIn / session.isVerified / session.state.user
//   await session.login(account, password) / session.register({email,password,nickname}) / session.logout()

import { reactive, computed } from 'vue'
import { createAuth } from '@haruhi/api-client'

// 单例状态：全 app 共享一份登录态，避免各组件各拉一次
const state = reactive({
  user: null, // 当前用户档案（含 emailVerified / nickname / avatar / apps 权限矩阵）
  ready: false, // 是否已完成首次 refresh（守卫据此避免闪烁）
  loading: false,
})

let auth = null
function ensureAuth(apiBase) {
  if (!auth) auth = createAuth(apiBase || '/api')
  return auth
}

export function useSession(apiBase = '/api') {
  const a = ensureAuth(apiBase)

  const isLoggedIn = computed(() => !!state.user)
  const isVerified = computed(() => !!(state.user && state.user.emailVerified))
  const isSuperAdmin = computed(() => !!(state.user && state.user.isSuperAdmin))

  // 拉取当前用户（带 cookie）。失败即视为未登录。幂等，可多次调用。
  async function refresh() {
    state.loading = true
    try {
      state.user = await a.me()
    } catch {
      state.user = null
    } finally {
      state.loading = false
      state.ready = true
    }
    return state.user
  }

  async function login(account, password) {
    const r = await a.login(account, password)
    // 需要两步验证：不写登录态，原样返回供 UI 跳二次验证
    if (r && r.twoFactorRequired) return r
    state.user = r
    state.ready = true
    return r
  }

  // 两步验证二次校验：成功写回登录态
  async function login2fa(pendingToken, code, backup = false) {
    state.user = await a.login2fa(pendingToken, code, backup)
    state.ready = true
    return state.user
  }

  async function register(payload) {
    state.user = await a.register(payload)
    state.ready = true
    return state.user
  }

  async function logout() {
    try {
      await a.logout()
    } finally {
      state.user = null
    }
  }

  async function updateProfile(patch) {
    state.user = await a.updateProfile(patch)
    return state.user
  }

  // 通行密钥登录：成功后写回登录态
  async function loginPasskey(opts) {
    state.user = await a.loginPasskey(opts)
    state.ready = true
    return state.user
  }

  return {
    state,
    isLoggedIn,
    isVerified,
    isSuperAdmin,
    refresh,
    login,
    login2fa,
    register,
    logout,
    updateProfile,
    loginPasskey,
    // 直通后端账号端点
    forgotPassword: (email) => a.forgotPassword(email),
    resetPassword: (token, password) => a.resetPassword(token, password),
    verifyEmail: (token) => a.verifyEmail(token),
    resendVerification: () => a.resendVerification(),
    changePassword: (oldPassword, newPassword) => a.changePassword(oldPassword, newPassword),
    listSessions: () => a.listSessions(),
    revokeSession: (id) => a.revokeSession(id),
    // 通行密钥管理
    isPasskeySupported: () => a.isPasskeySupported(),
    isConditionalUiAvailable: () => a.isConditionalUiAvailable(),
    listPasskeys: () => a.listPasskeys(),
    addPasskey: (name) => a.addPasskey(name),
    deletePasskey: (id) => a.deletePasskey(id),
    renamePasskey: (id, name) => a.renamePasskey(id, name),
    // 两步验证管理
    setup2fa: () => a.setup2fa(),
    enable2fa: (code) => a.enable2fa(code),
    disable2fa: (password) => a.disable2fa(password),
  }
}
