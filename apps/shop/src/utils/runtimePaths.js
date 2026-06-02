const normalizeBasePath = (value) => {
    const raw = String(value || '/').trim()
    if (!raw || raw === '/') return '/'

    let normalized = raw.startsWith('/') ? raw : `/${raw}`
    normalized = normalized.replace(/\/{2,}/g, '/')
    if (!normalized.endsWith('/')) normalized += '/'
    return normalized
}

const normalizeApiPrefix = (value) => {
    const raw = String(value || '/api/shop').trim()
    if (!raw) return '/api/shop'
    const normalized = raw.startsWith('/') ? raw : `/${raw}`
    return normalized.replace(/\/+$/, '') || '/api/shop'
}

const APP_BASE_URL = normalizeBasePath(import.meta.env.BASE_URL || '/')
const APP_BASE_PATH = APP_BASE_URL === '/' ? '' : APP_BASE_URL.replace(/\/+$/, '')
// 统一后端约定：模块 API 前缀固定为 /api/shop（旧 /shop-api → /api/shop）
const API_BASE_PATH = normalizeApiPrefix('/api/shop')

export const resolveAppPath = (path = '') => {
    const clean = String(path || '').replace(/^\/+/, '')
    if (!clean) return APP_BASE_URL
    return APP_BASE_PATH ? `${APP_BASE_PATH}/${clean}` : `/${clean}`
}

export const resolveApiPath = (path = '') => {
    const clean = String(path || '').replace(/^\/+/, '')
    if (!clean) return API_BASE_PATH
    return `${API_BASE_PATH}/${clean}`
}

export const isAdminPagePath = (pathname = '') => {
    const current = String(pathname || '')
    const adminRoot = resolveAppPath('admin')
    return current === adminRoot || current.startsWith(`${adminRoot}/`)
}

export const appBaseUrl = APP_BASE_URL
export const apiBasePath = API_BASE_PATH
