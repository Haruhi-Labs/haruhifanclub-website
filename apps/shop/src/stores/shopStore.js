import { reactive, computed } from 'vue'
import { trackEvent } from '@/utils/analytics'
import {
    buildAdminAuthHeaders,
    clearAdminToken,
    getShopAdminUser,
    hasValidAdminToken,
    loginShopAdmin,
    logoutShopAdmin
} from '@/utils/adminAuth'
import { isAdminPagePath, resolveApiPath, resolveAppPath } from '@/utils/runtimePaths'

// 使用 Vite 代理路径（统一前缀 /api/shop，由 resolveApiPath 注入）
const API_URL = resolveApiPath('/products')
const ADMIN_PRODUCT_ADJUST_URL = resolveApiPath('/admin/products')
const ORDER_URL = resolveApiPath('/orders')
const UPLOAD_URL = resolveApiPath('/upload')
const SITE_CONFIG_URL = resolveApiPath('/site-config')
const ADMIN_SITE_CONFIG_URL = resolveApiPath('/admin/site-config')
const COUPON_PREVIEW_URL = resolveApiPath('/coupons/preview')
const ADMIN_COUPONS_URL = resolveApiPath('/admin/coupons')
const CONTACT_MESSAGES_URL = resolveApiPath('/contact/messages')
const ADMIN_CONTACT_MESSAGES_URL = resolveApiPath('/admin/contact-messages')
const FREE_SHIPPING_THRESHOLD = 150
const FREE_SHIPPING_THRESHOLD_CENTS = Math.round(FREE_SHIPPING_THRESHOLD * 100)
const PRESALE_MODES = Object.freeze({
    NONE: 'none',
    GOAL: 'goal',
    FIXED: 'fixed'
})
const PRESALE_FIXED_DATE_TYPES = Object.freeze({
    MONTH_START: 'month_start',
    MONTH_END: 'month_end',
    DATE: 'date'
})
const PRESALE_MODE_VALUES = new Set(Object.values(PRESALE_MODES))
const PRESALE_FIXED_DATE_TYPE_VALUES = new Set(Object.values(PRESALE_FIXED_DATE_TYPES))
const DEFAULT_SITE_CONFIG = Object.freeze({
    payment: {
        wechatQr: '',
        alipayQr: '',
        friendQr: ''
    }
})
const toCents = (value) => Math.round((Number(value) || 0) * 100)
const fromCents = (value) => Number((Number(value || 0) / 100).toFixed(2))

const normalizeDiscountPrice = (discountPrice, price) => {
    const basePrice = Number(price)
    const value = Number(discountPrice)
    if (!Number.isFinite(value) || value <= 0) return null
    if (Number.isFinite(basePrice) && basePrice > 0 && value >= basePrice) return null
    return Number(value.toFixed(2))
}

const hasProductDiscount = (product) => normalizeDiscountPrice(product?.discountPrice, product?.price) !== null
const resolveProductPrice = (product) => {
    const discount = normalizeDiscountPrice(product?.discountPrice, product?.price)
    if (discount !== null) return discount
    const basePrice = Number(product?.price)
    return Number.isFinite(basePrice) ? Number(basePrice.toFixed(2)) : 0
}

const normalizeProduct = (product = {}) => {
    const basePrice = Number(product.price)
    const presaleModeRaw = String(product.presaleMode || PRESALE_MODES.NONE).trim().toLowerCase()
    const presaleMode = PRESALE_MODE_VALUES.has(presaleModeRaw) ? presaleModeRaw : PRESALE_MODES.NONE
    const presaleGoalTargetRaw = Number.parseInt(product.presaleGoalTarget, 10)
    const presaleGoalTarget = Number.isInteger(presaleGoalTargetRaw) && presaleGoalTargetRaw > 0 ? presaleGoalTargetRaw : 0
    const presaleFixedDateTypeRaw = String(product.presaleFixedDateType || '').trim().toLowerCase()
    const presaleFixedDateType = PRESALE_FIXED_DATE_TYPE_VALUES.has(presaleFixedDateTypeRaw) ? presaleFixedDateTypeRaw : ''
    const presaleFixedDateValue = String(product.presaleFixedDateValue || '').trim()
    const presalePaidCountRaw = Number(product.presalePaidCount)
    const presalePaidCount = Number.isFinite(presalePaidCountRaw) && presalePaidCountRaw > 0
        ? Math.floor(presalePaidCountRaw)
        : 0
    const presalePaidCountBaseRaw = Number(product.presalePaidCountBase)
    const presalePaidCountBase = Number.isFinite(presalePaidCountBaseRaw) && presalePaidCountBaseRaw > 0
        ? Math.floor(presalePaidCountBaseRaw)
        : 0
    const presalePaidOffsetRaw = Number.parseInt(product.presalePaidOffset, 10)
    const presalePaidOffset = Number.isInteger(presalePaidOffsetRaw) ? presalePaidOffsetRaw : 0

    return {
        ...product,
        imageOriginal: typeof product.imageOriginal === 'string' ? product.imageOriginal : '',
        price: Number.isFinite(basePrice) ? Number(basePrice.toFixed(2)) : 0,
        discountPrice: normalizeDiscountPrice(product.discountPrice, product.price),
        shippingCost: Number.isFinite(Number(product.shippingCost)) ? Number(product.shippingCost) : 0,
        presaleMode,
        presaleGoalTarget: presaleMode === PRESALE_MODES.GOAL ? presaleGoalTarget : 0,
        presaleFixedDateType: presaleMode === PRESALE_MODES.FIXED ? presaleFixedDateType : '',
        presaleFixedDateValue: presaleMode === PRESALE_MODES.FIXED ? presaleFixedDateValue : '',
        presalePaidCountBase,
        presalePaidOffset,
        presalePaidCount
    }
}

const normalizeSiteConfig = (rawConfig = {}) => {
    const next = JSON.parse(JSON.stringify(DEFAULT_SITE_CONFIG))
    const payment = rawConfig && typeof rawConfig === 'object' ? rawConfig.payment || {} : {}

    next.payment.wechatQr = typeof payment.wechatQr === 'string' ? payment.wechatQr : ''
    next.payment.alipayQr = typeof payment.alipayQr === 'string' ? payment.alipayQr : ''
    next.payment.friendQr = typeof payment.friendQr === 'string' ? payment.friendQr : ''
    return next
}

const getFileBaseName = (filename = '') => String(filename).replace(/\.[^/.]+$/, '') || 'image'

const renameFileToWebp = (file) => {
    if (!(file instanceof File)) return file
    if (String(file.type || '').toLowerCase() !== 'image/webp') return file
    if (String(file.name || '').toLowerCase().endsWith('.webp')) return file
    return new File([file], `${getFileBaseName(file.name)}.webp`, {
        type: file.type || 'image/webp',
        lastModified: file.lastModified || Date.now()
    })
}

let webpEncodeSupportCache = null
const canEncodeWebp = () => {
    if (webpEncodeSupportCache !== null) return webpEncodeSupportCache
    try {
        const canvas = document.createElement('canvas')
        canvas.width = 1
        canvas.height = 1
        webpEncodeSupportCache = canvas.toDataURL('image/webp').startsWith('data:image/webp')
    } catch {
        webpEncodeSupportCache = false
    }
    return webpEncodeSupportCache
}

const clampNumber = (value, min, max, fallback) => {
    const num = Number(value)
    if (!Number.isFinite(num)) return fallback
    return Math.min(max, Math.max(min, num))
}

const loadImageFromObjectUrl = (objectUrl) => (
    new Promise((resolve, reject) => {
        const img = new Image()
        img.onload = () => resolve(img)
        img.onerror = () => reject(new Error('图片解码失败'))
        img.src = objectUrl
    })
)

const createCanvasFromImage = (image, { maxDimension }) => {
    const rawWidth = Number(image.naturalWidth || image.width || 0)
    const rawHeight = Number(image.naturalHeight || image.height || 0)
    if (!rawWidth || !rawHeight) return null

    let width = rawWidth
    let height = rawHeight
    const resolvedMaxDimensionRaw = Number(maxDimension)
    if (Number.isFinite(resolvedMaxDimensionRaw) && resolvedMaxDimensionRaw > 0) {
        const longestEdge = Math.max(width, height)
        const resolvedMaxDimension = clampNumber(resolvedMaxDimensionRaw, 200, 6000, 2200)
        if (longestEdge > resolvedMaxDimension) {
            const ratio = resolvedMaxDimension / longestEdge
            width = Math.max(1, Math.round(width * ratio))
            height = Math.max(1, Math.round(height * ratio))
        }
    }

    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const ctx = canvas.getContext('2d')
    if (!ctx) return null
    ctx.imageSmoothingEnabled = true
    ctx.imageSmoothingQuality = 'high'
    ctx.drawImage(image, 0, 0, width, height)
    return canvas
}

const encodeCanvasToWebp = (canvas, quality) => (
    new Promise((resolve) => {
        canvas.toBlob((value) => resolve(value), 'image/webp', quality)
    })
)

const convertImageFileToWebp = async (file, options = {}) => {
    if (!(file instanceof File)) return null
    if (!String(file.type || '').startsWith('image/')) return null
    if (String(file.type) === 'image/webp') return renameFileToWebp(file)
    if (!canEncodeWebp()) return null

    const quality = clampNumber(options?.quality, 0.5, 1, 0.84)

    const objectUrl = URL.createObjectURL(file)
    try {
        const image = await loadImageFromObjectUrl(objectUrl)
        const canvas = createCanvasFromImage(image, {
            maxDimension: options?.maxDimension
        })
        if (!canvas) throw new Error('画布初始化失败')

        const blob = await encodeCanvasToWebp(canvas, quality)
        if (!blob) throw new Error('WebP 编码失败')
        if (String(blob.type || '').toLowerCase() !== 'image/webp') throw new Error('WebP 编码失败')

        return new File([blob], `${getFileBaseName(file.name)}.webp`, {
            type: blob.type,
            lastModified: Date.now()
        })
    } catch {
        return null
    } finally {
        URL.revokeObjectURL(objectUrl)
    }
}

const state = reactive({
    cart: [],
    currentType: 'all',
    products: [], 
    currentOrder: null,
    notification: null,
    adminUser: null,
    adminOrders: [], // 确保后台订单列表状态存在
    adminOrdersMeta: { page: 1, pageSize: 20, total: 0, totalPages: 1 },
    adminCoupons: [],
    adminCouponsMeta: { page: 1, pageSize: 20, total: 0, totalPages: 1 },
    adminContactMessages: [],
    adminContactMessagesMeta: { page: 1, pageSize: 20, total: 0, totalPages: 1 },
    siteConfig: normalizeSiteConfig()
})

export const useShopStore = () => {
    const isPresaleProduct = (product) => {
        const mode = String(product?.presaleMode || '').trim().toLowerCase()
        return mode === PRESALE_MODES.GOAL || mode === PRESALE_MODES.FIXED
    }

    const getPresaleProgress = (product) => {
        const target = Number(product?.presaleGoalTarget)
        const paidCount = Number(product?.presalePaidCount) || 0
        if (!Number.isFinite(target) || target <= 0) {
            return {
                paidCount,
                target: 0,
                percent: 0,
                reached: false
            }
        }
        const percent = Math.min(100, Math.max(0, (paidCount / target) * 100))
        return {
            paidCount,
            target,
            percent: Number(percent.toFixed(2)),
            reached: paidCount >= target
        }
    }

    const formatFixedPresaleDate = (product) => {
        const type = String(product?.presaleFixedDateType || '').trim().toLowerCase()
        const value = String(product?.presaleFixedDateValue || '').trim()
        if (!value) return ''

        if (type === PRESALE_FIXED_DATE_TYPES.MONTH_START || type === PRESALE_FIXED_DATE_TYPES.MONTH_END) {
            const [year, month] = value.split('-')
            if (!year || !month) return ''
            return type === PRESALE_FIXED_DATE_TYPES.MONTH_START
                ? `${year}年${month}月初`
                : `${year}年${month}月底`
        }
        if (type === PRESALE_FIXED_DATE_TYPES.DATE) {
            const [year, month, day] = value.split('-')
            if (!year || !month || !day) return ''
            return `${year}年${month}月${day}日`
        }
        return ''
    }

    // --- 计算属性 ---
    const cartCount = computed(() => state.cart.reduce((acc, item) => acc + item.quantity, 0))
    const cartTotal = computed(() => {
        const totalCents = state.cart.reduce((acc, item) => acc + toCents(item.price) * item.quantity, 0)
        return fromCents(totalCents)
    })
    
    // 运费计算：按标签分组取最大值后累加，满额包邮
    const shippingFee = computed(() => {
        if (state.cart.length === 0) return 0

        const productsTotalCents = toCents(cartTotal.value)
        if (productsTotalCents >= FREE_SHIPPING_THRESHOLD_CENTS) return 0

        const groups = {}
        state.cart.forEach(item => {
            const tag = item.shippingTag || 'default'
            const cost = toCents(item.shippingCost)
            if (groups[tag] === undefined) groups[tag] = cost
            else if (cost > groups[tag]) groups[tag] = cost
        })
        const totalCents = Object.values(groups).reduce((sum, val) => sum + Number(val || 0), 0)
        return fromCents(totalCents)
    })

    const finalTotal = computed(() => fromCents(toCents(cartTotal.value) + toCents(shippingFee.value)))

    const setProductType = (type) => { state.currentType = type }

    // --- 通用方法 ---
    const showNotification = (msg) => {
        state.notification = msg
        setTimeout(() => state.notification = null, 3000)
    }

    const setOrder = (order) => state.currentOrder = order

    const clearAdminState = () => {
        clearAdminToken()
        state.adminUser = null
        state.adminOrders = []
        state.adminOrdersMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
        state.adminCoupons = []
        state.adminCouponsMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
        state.adminContactMessages = []
        state.adminContactMessagesMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
    }

    const handleAdminUnauthorized = () => {
        clearAdminState()
        showNotification('登录已失效，请重新登录')
        if (typeof window !== 'undefined' && isAdminPagePath(window.location.pathname)) {
            window.location.href = resolveAppPath('admin/login')
        }
    }

    const ensureAdminAuth = () => {
        if (hasValidAdminToken()) return true
        showNotification('请先登录后台')
        return false
    }

    // 统一 JWT 单点登录：用户名 + 密码 → /api/auth/login，
    // 并校验是否具备 shop 管理权限（超管或 apps.shop）。
    const adminLogin = async (username, password) => {
        const result = await loginShopAdmin(username, password)
        if (!result.ok) {
            showNotification(result.error || '登录失败')
            return false
        }
        state.adminUser = result.user || null
        showNotification('登录成功')
        return true
    }

    const loadAdminUser = async () => {
        if (!ensureAdminAuth()) return null
        const user = await getShopAdminUser()
        state.adminUser = user || null
        return state.adminUser
    }

    const adminLogout = async () => {
        await logoutShopAdmin()
        clearAdminState()
    }

    const submitContactMessage = async (payload) => {
        try {
            const res = await fetch(CONTACT_MESSAGES_URL, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload || {})
            })
            const data = await res.json().catch(() => ({}))
            if (!res.ok) {
                showNotification(data.error || '留言提交失败')
                return false
            }
            showNotification('留言提交成功，我们会尽快处理')
            return true
        } catch (err) {
            showNotification('留言提交失败，请稍后重试')
            return false
        }
    }

    // --- 商品 API ---
    const fetchProducts = async () => {
        try {
            const res = await fetch(API_URL)
            const data = await res.json().catch(() => [])
            if (!res.ok) {
                showNotification(data.error || '商品加载失败')
                return
            }
            state.products = Array.isArray(data) ? data.map(normalizeProduct) : []
        } catch (err) { console.error(err) }
    }

    const fetchSiteConfig = async (admin = false) => {
        const url = admin ? ADMIN_SITE_CONFIG_URL : SITE_CONFIG_URL
        const headers = admin ? buildAdminAuthHeaders() : undefined

        try {
            const res = await fetch(url, headers ? { headers } : undefined)
            const data = await res.json().catch(() => ({}))
            if (res.status === 401 && admin) {
                handleAdminUnauthorized()
                return null
            }
            if (!res.ok) {
                if (admin) showNotification(data.error || '站点配置加载失败')
                return null
            }
            state.siteConfig = normalizeSiteConfig(data)
            return state.siteConfig
        } catch (err) {
            if (admin) showNotification('站点配置加载失败')
            return null
        }
    }

    const updateSiteConfig = async (configPayload) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(ADMIN_SITE_CONFIG_URL, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(configPayload)
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '保存配置失败')
                return false
            }

            state.siteConfig = normalizeSiteConfig(data.config)
            showNotification('配置已保存')
            return true
        } catch (err) {
            showNotification('保存配置失败')
            return false
        }
    }

    const previewCoupon = async ({ code, orderAmount }) => {
        try {
            const res = await fetch(COUPON_PREVIEW_URL, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ code, orderAmount })
            })
            const data = await res.json().catch(() => ({}))
            if (!res.ok) return { ok: false, error: data.error || '优惠券不可用' }
            return { ok: true, data }
        } catch (err) {
            return { ok: false, error: '网络错误，请稍后重试' }
        }
    }

    const fetchAdminCoupons = async (filters = {}) => {
        if (!ensureAdminAuth()) {
            state.adminCoupons = []
            state.adminCouponsMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
            return []
        }

        const search = new URLSearchParams()
        if (filters.status !== undefined && filters.status !== null && String(filters.status) !== '') {
            search.set('status', String(filters.status))
        }
        if (filters.batchNo) search.set('batchNo', String(filters.batchNo).trim())
        if (filters.keyword) search.set('keyword', String(filters.keyword).trim())
        if (filters.sortBy) search.set('sortBy', String(filters.sortBy))
        if (filters.sortDir) search.set('sortDir', String(filters.sortDir))
        if (filters.page) search.set('page', String(filters.page))
        if (filters.pageSize) search.set('pageSize', String(filters.pageSize))

        const url = `${ADMIN_COUPONS_URL}${search.toString() ? `?${search.toString()}` : ''}`

        try {
            const res = await fetch(url, { headers: buildAdminAuthHeaders() })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return []
            }
            if (!res.ok) {
                showNotification(data.error || '优惠券列表加载失败')
                return []
            }
            if (Array.isArray(data)) {
                state.adminCoupons = data
                state.adminCouponsMeta = {
                    page: filters.page ? Number(filters.page) : 1,
                    pageSize: filters.pageSize ? Number(filters.pageSize) : data.length || 20,
                    total: data.length,
                    totalPages: 1
                }
            } else {
                state.adminCoupons = Array.isArray(data.items) ? data.items : []
                state.adminCouponsMeta = data.pagination || {
                    page: 1,
                    pageSize: state.adminCoupons.length || 20,
                    total: state.adminCoupons.length,
                    totalPages: 1
                }
            }
            return state.adminCoupons
        } catch (err) {
            showNotification('优惠券列表加载失败')
            return []
        }
    }

    const createCouponBatch = async (payload) => {
        if (!ensureAdminAuth()) return null
        try {
            const res = await fetch(`${ADMIN_COUPONS_URL}/batch`, {
                method: 'POST',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(payload)
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return null
            }
            if (!res.ok) {
                showNotification(data.error || '创建优惠券失败')
                return null
            }
            showNotification(`已创建 ${data.quantity || 0} 张优惠券`)
            return data
        } catch (err) {
            showNotification('创建优惠券失败')
            return null
        }
    }

    const updateCouponStatus = async (id, status) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${ADMIN_COUPONS_URL}/${id}/status`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ status })
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '状态更新失败')
                return false
            }
            return true
        } catch (err) {
            showNotification('状态更新失败')
            return false
        }
    }

    const deleteCoupon = async (id) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${ADMIN_COUPONS_URL}/${id}`, {
                method: 'DELETE',
                headers: buildAdminAuthHeaders()
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '删除优惠券失败')
                return false
            }
            showNotification('优惠券已删除')
            return true
        } catch (err) {
            showNotification('删除优惠券失败')
            return false
        }
    }

    const fetchAdminContactMessages = async (filters = {}) => {
        if (!ensureAdminAuth()) {
            state.adminContactMessages = []
            state.adminContactMessagesMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
            return []
        }

        const search = new URLSearchParams()
        if (filters.status !== undefined && filters.status !== null && String(filters.status) !== '') {
            search.set('status', String(filters.status))
        }
        if (filters.keyword) search.set('keyword', String(filters.keyword).trim())
        if (filters.sortBy) search.set('sortBy', String(filters.sortBy))
        if (filters.sortDir) search.set('sortDir', String(filters.sortDir))
        if (filters.page) search.set('page', String(filters.page))
        if (filters.pageSize) search.set('pageSize', String(filters.pageSize))

        const url = `${ADMIN_CONTACT_MESSAGES_URL}${search.toString() ? `?${search.toString()}` : ''}`

        try {
            const res = await fetch(url, { headers: buildAdminAuthHeaders() })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return []
            }
            if (!res.ok) {
                showNotification(data.error || '留言列表加载失败')
                return []
            }
            if (Array.isArray(data)) {
                state.adminContactMessages = data
                state.adminContactMessagesMeta = {
                    page: filters.page ? Number(filters.page) : 1,
                    pageSize: filters.pageSize ? Number(filters.pageSize) : data.length || 20,
                    total: data.length,
                    totalPages: 1
                }
            } else {
                state.adminContactMessages = Array.isArray(data.items) ? data.items : []
                state.adminContactMessagesMeta = data.pagination || {
                    page: 1,
                    pageSize: state.adminContactMessages.length || 20,
                    total: state.adminContactMessages.length,
                    totalPages: 1
                }
            }
            return state.adminContactMessages
        } catch (err) {
            showNotification('留言列表加载失败')
            return []
        }
    }

    const updateAdminContactMessageStatus = async (id, status) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${ADMIN_CONTACT_MESSAGES_URL}/${id}/status`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ status })
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '留言状态更新失败')
                return false
            }
            return true
        } catch (err) {
            showNotification('留言状态更新失败')
            return false
        }
    }

    const uploadImage = async (file, options = {}) => {
        if (!ensureAdminAuth()) return null
        const purpose = options?.purpose === 'qr'
            ? 'qr'
            : (options?.purpose === 'original' ? 'original' : 'general')
        const shouldConvertToWebp = options?.convertToWebp ?? purpose === 'general'
        const compressionOptions = {
            maxDimension: options?.maxDimension,
            quality: options?.quality
        }

        let uploadFile = file
        if (shouldConvertToWebp) {
            uploadFile = await convertImageFileToWebp(file, compressionOptions)
            if (!uploadFile) {
                showNotification('图片转 WebP 失败，请更换图片后重试')
                return null
            }
            uploadFile = renameFileToWebp(uploadFile)
        }

        const formData = new FormData()
        formData.append('file', uploadFile)
        formData.append('purpose', purpose)
        try {
            const res = await fetch(UPLOAD_URL, {
                method: 'POST',
                headers: buildAdminAuthHeaders(),
                body: formData
            })
            if (res.status === 401) {
                handleAdminUnauthorized()
                return null
            }
            const data = await res.json().catch(() => ({}))
            if (res.ok) return data.url
            showNotification(data.error || '图片上传失败')
            return null
        } catch (e) { 
            showNotification('图片上传失败')
            return null 
        }
    }

    const addProduct = async (product) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(API_URL, {
                method: 'POST',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(product)
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (res.ok) { await fetchProducts(); showNotification('商品添加成功'); return true }
            showNotification(data.error || '添加失败')
        } catch(e) { showNotification('添加失败'); }
        return false
    }

    const updateProduct = async (id, product) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${API_URL}/${id}`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(product)
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (res.ok) { await fetchProducts(); showNotification('商品更新成功'); return true }
            showNotification(data.error || '更新失败')
        } catch(e) { showNotification('更新失败'); }
        return false
    }

    const adjustProductMetrics = async (id, { stockDelta = 0, presalePaidDelta = 0 } = {}) => {
        if (!ensureAdminAuth()) return false
        const normalizeDelta = (value) => {
            if (value === '' || value === null || value === undefined) return 0
            const num = Number(value)
            if (!Number.isInteger(num)) return null
            return num
        }
        const cleanStockDelta = normalizeDelta(stockDelta)
        const cleanPresalePaidDelta = normalizeDelta(presalePaidDelta)
        if (cleanStockDelta === null || cleanPresalePaidDelta === null) {
            showNotification('调整数量必须是整数')
            return false
        }
        if (cleanStockDelta === 0 && cleanPresalePaidDelta === 0) {
            showNotification('请至少调整一项数量')
            return false
        }

        try {
            const res = await fetch(`${ADMIN_PRODUCT_ADJUST_URL}/${id}/adjust`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({
                    stockDelta: cleanStockDelta,
                    presalePaidDelta: cleanPresalePaidDelta
                })
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '调整失败')
                return false
            }

            await fetchProducts()
            showNotification('库存/预售进度已调整')
            return true
        } catch (e) {
            showNotification('调整失败')
            return false
        }
    }

    const deleteProduct = async (id) => {
        if (!ensureAdminAuth()) return
        if (!confirm('确定删除?')) return
        try {
            const res = await fetch(`${API_URL}/${id}`, {
                method: 'DELETE',
                headers: buildAdminAuthHeaders()
            })
            if (res.status === 401) {
                handleAdminUnauthorized()
                return
            }
            await fetchProducts()
            showNotification('已删除')
        } catch(e) { showNotification('删除失败'); }
    }

    const reorderProducts = async (order) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${API_URL}/reorder`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json', ...buildAdminAuthHeaders() },
                body: JSON.stringify({ order })
            })
            if (res.status === 401) { handleAdminUnauthorized(); return false }
            if (!res.ok) return false
            await fetchProducts()
            return true
        } catch (e) { return false }
    }

    // --- 购物车逻辑 ---
    const addToCart = (product, qty = 1) => {
        const count = Number(qty)
        if (!Number.isFinite(count) || count <= 0) return

        const salePrice = resolveProductPrice(product)
        const originalPrice = Number(product.price) || salePrice
        const existingItem = state.cart.find(item => item.id === product.id)
        if (existingItem) {
            existingItem.quantity += count
        } else {
            state.cart.push({
                ...product,
                quantity: count,
                price: salePrice,
                originalPrice,
                discountPrice: normalizeDiscountPrice(product.discountPrice, product.price),
                shippingTag: product.shippingTag,
                shippingCost: Number(product.shippingCost) || 0
            })
        }
        showNotification('已加入购物车')
        trackEvent('add_to_cart', { productId: product.id, quantity: count })
    }

    const removeFromCart = (index) => state.cart.splice(index, 1)
    const clearCart = () => state.cart = []

    // --- 订单 API (前台 & 后台) ---
    
    // 前台：创建订单
    const createOrderBackend = async (orderData) => {
        try {
            const res = await fetch(ORDER_URL, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(orderData) })
            const data = await res.json()
            if (!res.ok) throw new Error(data.error || '创建订单失败')
            const finalOrderId = data.orderId || orderData.id
            trackEvent('order_submitted', {
                orderId: finalOrderId,
                total: data.total || orderData.total,
                itemCount: (orderData.items || []).length
            })
            return data
        } catch (e) {
            showNotification(e.message)
            return null
        }
    }

    // 后台：获取订单列表
    const fetchAdminOrders = async (status = 'all', filters = {}) => {
        if (!ensureAdminAuth()) {
            state.adminOrders = []
            state.adminOrdersMeta = { page: 1, pageSize: 20, total: 0, totalPages: 1 }
            return
        }

        const search = new URLSearchParams()
        search.set('status', String(status))
        if (filters.keyword) search.set('keyword', String(filters.keyword).trim())
        if (filters.sortBy) search.set('sortBy', String(filters.sortBy))
        if (filters.sortDir) search.set('sortDir', String(filters.sortDir))
        if (filters.page) search.set('page', String(filters.page))
        if (filters.pageSize) search.set('pageSize', String(filters.pageSize))
        if (filters.hasPresale) search.set('hasPresale', '1')
        if (filters.hasSpot) search.set('hasSpot', '1')

        try {
            const res = await fetch(`${ORDER_URL}?${search.toString()}`, {
                headers: buildAdminAuthHeaders()
            })
            if (res.status === 401) {
                handleAdminUnauthorized()
                return
            }
            if (res.ok) {
                const data = await res.json().catch(() => ({}))
                if (Array.isArray(data)) {
                    state.adminOrders = data
                    state.adminOrdersMeta = {
                        page: filters.page ? Number(filters.page) : 1,
                        pageSize: filters.pageSize ? Number(filters.pageSize) : data.length || 20,
                        total: data.length,
                        totalPages: 1
                    }
                } else {
                    state.adminOrders = Array.isArray(data.items) ? data.items : []
                    state.adminOrdersMeta = data.pagination || {
                        page: 1,
                        pageSize: state.adminOrders.length || 20,
                        total: state.adminOrders.length,
                        totalPages: 1
                    }
                }
            } else {
                const data = await res.json().catch(() => ({}))
                showNotification(data.error || '订单列表加载失败')
            }
        } catch (e) { console.error("Fetch orders failed", e) }
    }

    // 前台：用户提交支付凭证 (待付款 -> 待确认)
    const submitOrderPayment = async (id) => {
        try {
            const res = await fetch(`${ORDER_URL}/${id}/payment`, { method: 'POST' })
            const data = await res.json().catch(() => ({}))
            if (!res.ok) {
                showNotification(data.error || '提交支付失败')
                return false
            }
            return true
        } catch (e) {
            showNotification('网络错误')
            return false
        }
    }

    // 后台：更新订单状态
    const updateOrderStatus = async (id, status, tracking = {}, statusFilter = null, listFilters = null) => {
        if (!ensureAdminAuth()) return false
        try {
            const payload = { status, ...tracking }
            const res = await fetch(`${ORDER_URL}/${id}/status`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(payload)
            })
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (res.ok) {
                if (statusFilter !== null) {
                    await fetchAdminOrders(statusFilter, listFilters || {}) // 仅后台页按当前筛选刷新列表
                }
                showNotification('订单状态已更新')
                return true
            } else {
                const data = await res.json().catch(() => ({}))
                showNotification(data.error || '状态更新失败')
                return false
            }
        } catch (e) {
            showNotification('网络错误')
            return false
        }
    }

    const updateAdminOrderContact = async (id, contact, statusFilter = null, listFilters = null) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${ORDER_URL}/${id}/contact`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ contact: contact || {} })
            })
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            const data = await res.json().catch(() => ({}))
            if (!res.ok) {
                showNotification(data.error || '收货信息更新失败')
                return false
            }
            if (statusFilter !== null) {
                await fetchAdminOrders(statusFilter, listFilters || {})
            }
            showNotification('收货信息已更新')
            return true
        } catch (e) {
            showNotification('网络错误')
            return false
        }
    }

    const deleteAdminOrder = async (id, statusFilter = null, listFilters = null) => {
        if (!ensureAdminAuth()) return false
        try {
            const res = await fetch(`${ORDER_URL}/${id}`, {
                method: 'DELETE',
                headers: buildAdminAuthHeaders()
            })
            const data = await res.json().catch(() => ({}))
            if (res.status === 401) {
                handleAdminUnauthorized()
                return false
            }
            if (!res.ok) {
                showNotification(data.error || '删除订单失败')
                return false
            }
            if (statusFilter !== null) {
                await fetchAdminOrders(statusFilter, listFilters || {})
            }
            showNotification('订单已删除')
            return true
        } catch (e) {
            showNotification('网络错误')
            return false
        }
    }

    const fetchOrderIds = async (status, exported, { notFullyExported } = {}) => {
        if (!ensureAdminAuth()) return []
        const search = new URLSearchParams()
        search.set('status', String(status))
        if (exported !== undefined && exported !== null) search.set('exported', String(exported))
        if (notFullyExported) search.set('notFullyExported', '1')
        try {
            const res = await fetch(`${ORDER_URL}/ids?${search.toString()}`, { headers: buildAdminAuthHeaders() })
            if (!res.ok) return []
            const data = await res.json()
            return Array.isArray(data.ids) ? data.ids : []
        } catch { return [] }
    }

    const markOrdersExported = async (ids) => {
        if (!ensureAdminAuth() || !ids.length) return false
        try {
            const res = await fetch(`${ORDER_URL}/mark-exported`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ ids })
            })
            return res.ok
        } catch { return false }
    }

    const markOrdersSpotExported = async (ids) => {
        if (!ensureAdminAuth() || !ids.length) return false
        try {
            const res = await fetch(`${ORDER_URL}/mark-spot-exported`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ ids })
            })
            return res.ok
        } catch { return false }
    }

    const markPresaleExported = async (ids, productIds) => {
        if (!ensureAdminAuth() || !ids.length || !productIds.length) return false
        try {
            const res = await fetch(`${ORDER_URL}/mark-presale-exported`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify({ ids, productIds })
            })
            return res.ok
        } catch { return false }
    }

    const fetchSpotExportData = async () => {
        if (!ensureAdminAuth()) return []
        try {
            const res = await fetch(`${ORDER_URL}/spot-export-data`, { headers: buildAdminAuthHeaders() })
            if (!res.ok) return []
            const data = await res.json()
            return Array.isArray(data.orders) ? data.orders : []
        } catch { return [] }
    }

    const shipSubOrder = async (orderId, subKey, tracking = {}) => {
        if (!ensureAdminAuth()) return null
        try {
            const res = await fetch(`${ORDER_URL}/${orderId}/sub-orders/${encodeURIComponent(subKey)}/ship`, {
                method: 'PUT',
                headers: buildAdminAuthHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(tracking)
            })
            const data = await res.json().catch(() => ({}))
            if (!res.ok) {
                showNotification(data.error || '子订单发货失败')
                return null
            }
            showNotification('子订单已发货')
            return data
        } catch {
            showNotification('子订单发货失败')
            return null
        }
    }

    const importTracking = async (file) => {
        if (!ensureAdminAuth() || !file) return null
        try {
            const form = new FormData()
            form.append('file', file)
            const res = await fetch(`${ORDER_URL}/import-tracking`, {
                method: 'POST',
                headers: buildAdminAuthHeaders(),
                body: form
            })
            return await res.json()
        } catch { return null }
    }

    return {
        state, cartCount, cartTotal, shippingFee, finalTotal,
        freeShippingThreshold: FREE_SHIPPING_THRESHOLD,
        setProductType, addToCart, removeFromCart, clearCart, showNotification, setOrder,
        fetchProducts, addProduct, updateProduct, adjustProductMetrics, deleteProduct, reorderProducts, uploadImage,
        createOrderBackend, submitOrderPayment,
        fetchAdminOrders, updateOrderStatus, updateAdminOrderContact, deleteAdminOrder,
        fetchOrderIds, markOrdersExported, markOrdersSpotExported, markPresaleExported, fetchSpotExportData, shipSubOrder, importTracking,
        previewCoupon, fetchAdminCoupons, createCouponBatch, updateCouponStatus, deleteCoupon,
        submitContactMessage, fetchAdminContactMessages, updateAdminContactMessageStatus,
        fetchSiteConfig, updateSiteConfig,
        resolveProductPrice, hasProductDiscount, isPresaleProduct, getPresaleProgress, formatFixedPresaleDate,
        adminLogin, loadAdminUser, adminLogout
    }
}
