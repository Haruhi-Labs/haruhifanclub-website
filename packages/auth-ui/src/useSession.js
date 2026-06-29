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
let refreshPromise = null
let sessionVersion = 0

function ensureAuth(apiBase) {
  if (!auth) auth = createAuth(apiBase || '/api')
  return auth
}

export function useSession(apiBase = '/api') {
  const a = ensureAuth(apiBase)

  const isLoggedIn = computed(() => !!state.user)
  const isVerified = computed(() => !!(state.user && state.user.emailVerified))
  const isSuperAdmin = computed(() => !!(state.user && state.user.isSuperAdmin))

  function commitUser(user) {
    sessionVersion += 1
    refreshPromise = null
    state.user = user
    state.ready = true
    state.loading = false
    return user
  }

  // 拉取当前用户（带 cookie）。失败即视为未登录。幂等，可多次调用。
  async function refresh() {
    if (refreshPromise) return refreshPromise
    const version = sessionVersion
    state.loading = true
    const promise = (async () => {
      let user = null
      try {
        user = await a.me()
      } catch {
        user = null
      } finally {
        if (version === sessionVersion) {
          state.user = user
          state.ready = true
          state.loading = false
        }
        if (refreshPromise === promise) refreshPromise = null
      }
      return version === sessionVersion ? user : state.user
    })()
    refreshPromise = promise
    return refreshPromise
  }

  async function ensureReady() {
    if (state.ready) return state.user
    return refresh()
  }

  async function login(account, password) {
    const r = await a.login(account, password)
    // 需要两步验证：不写登录态，原样返回供 UI 跳二次验证
    if (r && r.twoFactorRequired) return r
    return commitUser(r)
  }

  // 两步验证二次校验：成功写回登录态
  async function login2fa(pendingToken, code, backup = false) {
    return commitUser(await a.login2fa(pendingToken, code, backup))
  }

  async function register(payload) {
    return commitUser(await a.register(payload))
  }

  async function logout() {
    try {
      await a.logout()
    } finally {
      commitUser(null)
    }
  }

  async function updateProfile(patch) {
    state.user = await a.updateProfile(patch)
    return state.user
  }

  // 上传头像（File/Blob）：成功后写回登录态，各处头像即时刷新
  async function uploadAvatar(file) {
    state.user = await a.uploadAvatar(file)
    return state.user
  }

  // 移除头像：成功后写回登录态
  async function removeAvatar() {
    state.user = await a.removeAvatar()
    return state.user
  }

  // 通行密钥登录：成功后写回登录态
  async function loginPasskey(opts) {
    return commitUser(await a.loginPasskey(opts))
  }

  return {
    state,
    isLoggedIn,
    isVerified,
    isSuperAdmin,
    refresh,
    ensureReady,
    login,
    login2fa,
    register,
    logout,
    updateProfile,
    uploadAvatar,
    removeAvatar,
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
