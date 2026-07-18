import { getCsrfToken, getToken, resolveUploadUrl } from '@haruhi/api-client'

// 统一后端约定：
// - 模块 API 统一前缀 /api/art（旧的 /api/xxx → /api/art/xxx）
// - 静态图片统一走 /uploads/<path>
// - 管理员鉴权改为统一 JWT：请求自动带 Authorization: Bearer <jwt>（来自 @haruhi/api-client 的 token）
// - 前台匿名互动（点赞/评论）：后端用签名 Cookie 维持匿名身份，故所有请求带 credentials: 'include'
const API_PREFIX = '/api/art'
// 静态资源根：后端库里存的是相对 uploads 根的路径（如 art/2025-11/x.webp）
const ASSET_BASE = '/uploads'

function buildUrl(path, params) {
  const url = new URL(path, window.location.origin)
  if (params && typeof params === 'object') {
    for (const [k, v] of Object.entries(params)) {
      if (v !== undefined && v !== null) url.searchParams.set(k, String(v))
    }
  }
  return url.pathname + url.search
}

async function request(method, path, { params, body, isForm, headers, keepalive = false } = {}) {
  const url = buildUrl(path, params)

  // 统一 JWT：若已登录则自动带上 Bearer token（替换旧的 x-admin-password 头）
  const token = getToken()
  const authHeaders = token ? { Authorization: `Bearer ${token}` } : {}
  const csrf = method !== 'GET' && method !== 'HEAD' ? getCsrfToken() : ''
  const csrfHeaders = csrf ? { 'X-CSRF-Token': csrf } : {}

  const init = {
    method,
    // 匿名点赞/评论依赖后端签名 Cookie，必须带上凭证
    credentials: 'include',
    headers: {
      Accept: 'application/json',
      ...authHeaders,
      ...csrfHeaders,
      ...(headers || {}),
    },
    keepalive,
  }

  if (method !== 'GET' && method !== 'HEAD') {
    if (isForm) {
      // For FormData, do not set Content-Type header manually; browser does it
      init.body = body
    } else if (body !== undefined) {
      init.headers['Content-Type'] = 'application/json'
      init.body = JSON.stringify(body)
    }
  }

  const res = await fetch(url, init)
  let data
  try {
    data = await res.json()
  } catch {
    throw new Error(`HTTP ${res.status}`)
  }

  if (!res.ok || data?.ok === false) {
    throw new Error(data?.message || data?.error || `HTTP ${res.status}`)
  }
  return data
}

// 辅助函数：修复图片路径——委托给共享的 resolveUploadUrl（语义与原 fixPath 完全一致）。
// 后端返回的图片字段（image_url/original_url/avatar_url 等）为相对 uploads 根的路径，
// 统一拼成 /uploads/<path>；已是绝对地址或站内绝对路径则原样保留。
const fixPath = (p) => resolveUploadUrl(p, ASSET_BASE)

// 站内 art 图片转缩略图 URL。直接指向静态缓存路径
//   /uploads/art/.thumbs/<w>/<sub>.<ext>.webp
// 让 nginx 命中即静态直出（零后端开销）；未命中时 nginx @fallback 回源后端
// /api/art/thumb 现场生成并落盘，下次即静态命中。缓存路径布局须与后端
// thumb_cache_path 完全一致（art.rs）。外链、非 art 路径、gif/svg 原样返回。
export function thumbUrl(url, w = 640) {
  if (!url || typeof url !== 'string') return url
  if (!url.startsWith(`${ASSET_BASE}/art/`)) return url
  const rel = url.slice(ASSET_BASE.length + 1) // 例 art/2026-06/x.webp
  if (/\.(gif|svg)$/i.test(rel)) return url
  const sub = rel.slice('art/'.length) // 2026-06/x.webp
  return `${ASSET_BASE}/art/.thumbs/${w}/${sub}.webp`
}

function transformArtwork(a) {
  if (!a) return a

  // 修复单图路径
  a.image_url = fixPath(a.image_url)
  a.original_url = fixPath(a.original_url)
  a.uploader_avatar = fixPath(a.uploader_avatar)

  // 修复多图数组路径
  if (Array.isArray(a.images)) {
    a.images = a.images.map(img => ({
      ...img,
      image_url: fixPath(img.image_url),
      original_url: fixPath(img.original_url)
    }))
  }

  return a
}

function transformSocialPayload(data) {
  if (!data) return data
  if (data.profile) data.profile.avatar_url = fixPath(data.profile.avatar_url)
  if (Array.isArray(data.artworks)) data.artworks = data.artworks.map(transformArtwork)
  if (Array.isArray(data.favorites)) data.favorites = data.favorites.map(transformArtwork)
  for (const key of ['followers', 'following']) {
    if (Array.isArray(data.social?.[key])) {
      data.social[key] = data.social[key].map(item => ({
        ...item,
        avatar_url: fixPath(item.avatar_url),
      }))
    }
  }
  return data
}

function transformGuildReward(reward) {
  if (!reward) return reward
  reward.imageUrl = fixPath(reward.imageUrl)
  reward.categoryId = reward.categoryId ? Number(reward.categoryId) : null
  return reward
}

function transformGuildRewardCategory(category) {
  if (!category) return category
  return {
    ...category,
    id: Number(category.id),
    sortOrder: Number(category.sortOrder || 0),
  }
}

export const api = {
  health: () => request('GET', `${API_PREFIX}/health`),

  // Public
  recordVisitor: () => request('POST', `${API_PREFIX}/visitors`, { body: {} }),
  artworksList: async (params) => {
    const data = await request('GET', `${API_PREFIX}/artworks`, { params })
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  recommendations: async (limit = 8) => {
    const data = await request('GET', `${API_PREFIX}/recommendations`, { params: { limit } })
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  creatorExhibits: async () => {
    const data = await request('GET', `${API_PREFIX}/creator-exhibits`)
    if (data.data) {
      data.data = data.data.map(group => ({
        ...group,
        avatar: fixPath(group.avatar),
        items: (group.items || []).map(transformArtwork),
      }))
    }
    return data
  },
  recordRecommendationEvents: (events, sessionId, keepalive = false) =>
    request('POST', `${API_PREFIX}/recommendation-events`, {
      body: { session_id: sessionId, events },
      keepalive,
    }),
  getArtwork: async (id) => {
    const data = await request('GET', `${API_PREFIX}/artworks/${id}`)
    if (data.data) data.data = transformArtwork(data.data)
    return data
  },
  relatedArtworks: async (id, limit = 8) => {
    const data = await request('GET', `${API_PREFIX}/artworks/${id}/related`, { params: { limit } })
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  creatorNeighbors: async (current) => {
    if (!current?.id || !current?.uploader_uid) return { ok: true, data: [] }
    const response = await request('GET', `${API_PREFIX}/artworks/${current.id}/creator-neighbors`)
    return {
      ...response,
      data: (response.data || []).map(transformArtwork),
    }
  },
  creatorProfile: async (uid) => {
    const data = await request('GET', `${API_PREFIX}/creators/${encodeURIComponent(uid)}`)
    if (data?.creator) data.creator.avatar_url = fixPath(data.creator.avatar_url)
    if (data?.data) data.data.avatar_url = fixPath(data.data.avatar_url)
    return data
  },
  creatorWorks: async (params) => {
    const data = await request('GET', `${API_PREFIX}/artworks`, {
      params: { ...params, status: 'approved', source_type: 'personal', uploader_uid: params.uid }
    })
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  verifyCreator: async (uid) => {
    const data = await request('GET', `${API_PREFIX}/creators/verify`, { params: { uid } })
    if (data?.creator) data.creator.avatar_url = fixPath(data.creator.avatar_url)
    return data
  },
  uploadArtwork: (formData) => request('POST', `${API_PREFIX}/artworks`, { body: formData, isForm: true }),

  // Interaction
  likeArtwork: (id) => request('POST', `${API_PREFIX}/likes/artwork/${id}`, { body: {} }),
  toggleArtworkFavorite: (id) =>
    request('POST', `${API_PREFIX}/artworks/${id}/favorite`, { body: {} }),
  likeComment: (id) => request('POST', `${API_PREFIX}/likes/comment/${id}`, { body: {} }),
  listComments: (artworkId) =>
    request('GET', `${API_PREFIX}/comments`, { params: { artwork_id: artworkId } }),
  postComment: (body) => request('POST', `${API_PREFIX}/comments`, { body }),

  // Announcements（公告：公开只读 + 后台 CRUD）
  announcements: () => request('GET', `${API_PREFIX}/announcements`),
  adminAnnouncements: () => request('GET', `${API_PREFIX}/admin/announcements`),
  adminCreateAnnouncement: (body) => request('POST', `${API_PREFIX}/admin/announcements`, { body }),
  adminUpdateAnnouncement: (id, body) =>
    request('POST', `${API_PREFIX}/admin/announcements/${id}/update`, { body }),
  adminDeleteAnnouncement: (id) => request('DELETE', `${API_PREFIX}/admin/announcements/${id}`),

  // Admin - Artworks
  adminPendingArtworks: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/pending-artworks`)
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  adminAuditHistory: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/audit-history`)
    if (data.data) data.data = data.data.map(transformArtwork)
    return data
  },
  adminApproveArtwork: (id, note) => request('POST', `${API_PREFIX}/admin/artworks/${id}/approve`, { body: { note } }),
  adminRejectArtwork: (id, note) => request('POST', `${API_PREFIX}/admin/artworks/${id}/reject`, { body: { note } }),
  adminUpdateArtworkStatus: (id, status) => request('POST', `${API_PREFIX}/admin/artworks/${id}/status`, { body: { status } }),
  adminUpdateArtworkDetails: (id, data) => request('POST', `${API_PREFIX}/admin/artworks/${id}/update`, { body: data }),
  adminDeleteArtwork: (id) => request('DELETE', `${API_PREFIX}/admin/artworks/${id}`),

  // Admin - Comments
  adminListComments: (status) => request('GET', `${API_PREFIX}/admin/comments`, { params: { status } }),
  adminUpdateCommentStatus: (id, status) => request('POST', `${API_PREFIX}/admin/comments/${id}/status`, { body: { status } }),
  adminDeleteComment: (id) => request('DELETE', `${API_PREFIX}/admin/comments/${id}`),

  // Admin - Creators & Points
  adminPointsLedger: (params) => request('GET', `${API_PREFIX}/admin/points-ledger`, { params }),
  adminCreators: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/creators`)
    if (data.data) {
      data.data = data.data.map(c => {
        c.avatar_url = fixPath(c.avatar_url)
        return c
      })
    }
    return data
  },
  adminAddCreator: (uid) => request('POST', `${API_PREFIX}/admin/creators`, { body: { uid } }),

  // 更新创作者信息（支持传 FormData 包含头像文件）
  adminUpdateCreator: (uid, formData) => request('POST', `${API_PREFIX}/admin/creators/${encodeURIComponent(uid)}/update`, { body: formData, isForm: true }),

  // 删除创作者
  adminDeleteCreator: (uid) => request('DELETE', `${API_PREFIX}/admin/creators/${encodeURIComponent(uid)}`),

  adminGrantPoints: (body) => request('POST', `${API_PREFIX}/admin/points/grant`, { body }),
  adminPenalizePoints: (body) => request('POST', `${API_PREFIX}/admin/points/penalize`, { body }),

  // Points & Leaderboard
  pointsLeaderboard: async (page = 1) => {
    const data = await request('GET', `${API_PREFIX}/points/leaderboard`, { params: { page } })
    if (data.data) {
      data.data = data.data.map(c => {
        c.avatar_url = fixPath(c.avatar_url)
        return c
      })
    }
    return data
  },
  pointsHistory: (uid) => request('GET', `${API_PREFIX}/points/history`, { params: { uid } }),
  // Guild / Adventurer system
  guildMe: () => request('GET', `${API_PREFIX}/guild/me`),
  guildTerminal: async () => {
    const data = await request('GET', `${API_PREFIX}/guild/terminal`)
    return transformSocialPayload(data)
  },
  guildProfile: async (uid) => {
    const data = await request('GET', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}`)
    return transformSocialPayload(data)
  },
  guildProfileArtworks: async (uid, params = {}) => {
    const data = await request('GET', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/artworks`, { params })
    if (Array.isArray(data.data)) data.data = data.data.map(transformArtwork)
    return data
  },
  guildProfileMessages: (uid, params = {}) =>
    request('GET', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/messages`, { params }),
  postGuildProfileMessage: (uid, body) =>
    request('POST', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/messages`, {
      body: { body },
    }),
  toggleGuildFollow: (uid) =>
    request('POST', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/follow`, { body: {} }),
  guildProfileConnections: async (uid, params = {}) => {
    const data = await request(
      'GET',
      `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/connections`,
      { params },
    )
    if (Array.isArray(data.data)) {
      data.data = data.data.map(item => ({
        ...item,
        avatar_url: fixPath(item.avatar_url),
      }))
    }
    return data
  },
  guildProfileFavorites: async (uid, params = {}) => {
    const data = await request('GET', `${API_PREFIX}/guild/profile/${encodeURIComponent(uid)}/favorites`, { params })
    if (Array.isArray(data.data)) data.data = data.data.map(transformArtwork)
    return data
  },
  guildQuests: () => request('GET', `${API_PREFIX}/guild/quests`),
  guildClaimQuest: (id) => request('POST', `${API_PREFIX}/guild/quests/${id}/claim`, { body: {} }),
  guildQuestSubmissionArtworks: async (id) => {
    const data = await request('GET', `${API_PREFIX}/guild/quests/${id}/submission-artworks`)
    if (Array.isArray(data.data)) data.data = data.data.map(transformArtwork)
    return data
  },
  guildSubmitQuestArtworks: (id, artworkIds) =>
    request('POST', `${API_PREFIX}/guild/quests/${id}/submit-artworks`, { body: { artworkIds } }),
  guildLeaderboard: () => request('GET', `${API_PREFIX}/guild/leaderboard`),
  guildCoinHistory: () => request('GET', `${API_PREFIX}/guild/coins/history`),
  guildApplyRating: (body) => request('POST', `${API_PREFIX}/guild/rating/apply`, { body }),
  guildAccessSubmissionArtworks: async () => {
    const data = await request('GET', `${API_PREFIX}/guild/access/submission-artworks`)
    if (Array.isArray(data.data)) data.data = data.data.map(transformArtwork)
    return data
  },
  guildApplyAccess: (body) => request('POST', `${API_PREFIX}/guild/access/apply`, { body }),
  guildRewards: async () => {
    const data = await request('GET', `${API_PREFIX}/guild/rewards`)
    if (Array.isArray(data.data)) data.data = data.data.map(transformGuildReward)
    if (Array.isArray(data.categories))
      data.categories = data.categories.map(transformGuildRewardCategory)
    return data
  },
  guildRedeemReward: (id, body = {}) => request('POST', `${API_PREFIX}/guild/rewards/${id}/redeem`, { body }),
  guildMyRedemptions: () => request('GET', `${API_PREFIX}/guild/redemptions/me`),

  // Admin - Guild
  adminRewardSettings: () => request('GET', `${API_PREFIX}/admin/reward-settings`),
  adminUpdateRewardSettings: (body) => request('PUT', `${API_PREFIX}/admin/reward-settings`, { body }),
  adminGuildQuests: () => request('GET', `${API_PREFIX}/admin/guild/quests`),
  adminCreateGuildQuest: (body) => request('POST', `${API_PREFIX}/admin/guild/quests`, { body }),
  adminUpdateGuildQuest: (id, body) => request('PUT', `${API_PREFIX}/admin/guild/quests/${id}`, { body }),
  adminDeleteGuildQuest: (id) => request('DELETE', `${API_PREFIX}/admin/guild/quests/${id}`),
  adminUpdateGuildQuestStatus: (id, status) => request('POST', `${API_PREFIX}/admin/guild/quests/${id}/status`, { body: { status } }),
  adminGuildQuestClaims: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/guild/quest-claims`)
    if (Array.isArray(data.data)) {
      data.data = data.data.map(item => ({
        ...item,
        submittedArtworks: Array.isArray(item.submittedArtworks)
          ? item.submittedArtworks.map(transformArtwork)
          : [],
      }))
    }
    return data
  },
  adminApproveGuildQuestClaim: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/quest-claims/${id}/approve`, { body: { note } }),
  adminRejectGuildQuestClaim: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/quest-claims/${id}/reject`, { body: { note } }),
  adminGuildCreatorProductionStats: (params) => request('GET', `${API_PREFIX}/admin/guild/creator-production-stats`, { params }),
  adminGuildBudget: () => request('GET', `${API_PREFIX}/admin/guild/budget`),
  adminCreateGuildBudgetSupply: (body) => request('POST', `${API_PREFIX}/admin/guild/budget/supplies`, { body }),
  adminGuildRewards: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/guild/rewards`)
    if (Array.isArray(data.data)) data.data = data.data.map(transformGuildReward)
    if (Array.isArray(data.categories))
      data.categories = data.categories.map(transformGuildRewardCategory)
    return data
  },
  adminCreateGuildReward: (body) => request('POST', `${API_PREFIX}/admin/guild/rewards`, { body }),
  adminUpdateGuildReward: (id, body) => request('PUT', `${API_PREFIX}/admin/guild/rewards/${id}`, { body }),
  adminDeleteGuildReward: (id) => request('DELETE', `${API_PREFIX}/admin/guild/rewards/${id}`),
  adminUpdateGuildRewardStatus: (id, status) => request('POST', `${API_PREFIX}/admin/guild/rewards/${id}/status`, { body: { status } }),
  adminUpdateGuildRewardStock: (id, stock) => request('POST', `${API_PREFIX}/admin/guild/rewards/${id}/stock`, { body: { stock } }),
  adminGuildRewardCategories: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/guild/reward-categories`)
    if (Array.isArray(data.data)) data.data = data.data.map(transformGuildRewardCategory)
    return data
  },
  adminCreateGuildRewardCategory: (body) =>
    request('POST', `${API_PREFIX}/admin/guild/reward-categories`, { body }),
  adminUpdateGuildRewardCategory: (id, body) =>
    request('PUT', `${API_PREFIX}/admin/guild/reward-categories/${id}`, { body }),
  adminUpdateGuildRewardCategoryStatus: (id, status) =>
    request('POST', `${API_PREFIX}/admin/guild/reward-categories/${id}/status`, {
      body: { status },
    }),
  adminUploadGuildRewardImage: async (file) => {
    const formData = new FormData()
    formData.append('image', file)
    const data = await request('POST', `${API_PREFIX}/admin/guild/rewards/image`, { body: formData, isForm: true })
    data.url = fixPath(data.url || data.imageUrl)
    return data
  },
  adminGuildRedemptions: () => request('GET', `${API_PREFIX}/admin/guild/redemptions`),
  adminApproveGuildRedemption: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/redemptions/${id}/approve`, { body: { note } }),
  adminRejectGuildRedemption: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/redemptions/${id}/reject`, { body: { note } }),
  adminCancelGuildRedemption: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/redemptions/${id}/cancel`, { body: { note } }),
  adminFulfillGuildRedemption: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/redemptions/${id}/fulfilled`, { body: { note } }),
  adminGuildRatingApplications: () => request('GET', `${API_PREFIX}/admin/guild/rating-applications`),
  adminApproveGuildRating: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/rating-applications/${id}/approve`, { body: { note } }),
  adminRejectGuildRating: (id, note = '') => request('POST', `${API_PREFIX}/admin/guild/rating-applications/${id}/reject`, { body: { note } }),
  adminGuildAccessApplications: async () => {
    const data = await request('GET', `${API_PREFIX}/admin/guild/access-applications`)
    if (Array.isArray(data.data)) {
      data.data = data.data.map(item => ({
        ...item,
        submittedArtworks: Array.isArray(item.submittedArtworks)
          ? item.submittedArtworks.map(transformArtwork)
          : [],
      }))
    }
    return data
  },
  adminApproveGuildAccess: (id, note = '') =>
    request('POST', `${API_PREFIX}/admin/guild/access-applications/${id}/approve`, {
      body: { note },
    }),
  adminRejectGuildAccess: (id, note = '') =>
    request('POST', `${API_PREFIX}/admin/guild/access-applications/${id}/reject`, {
      body: { note },
    }),
  adminUpdateGuildProfileAccess: (uid, accessTier) => request('POST', `${API_PREFIX}/admin/guild/profiles/${encodeURIComponent(uid)}/access`, { body: { accessTier } }),
  searchCreators: async (q) => {
    const data = await request('GET', `${API_PREFIX}/creators/search`, { params: { q } })
    if (data.data) {
      data.data = data.data.map(c => {
        c.avatar_url = fixPath(c.avatar_url)
        return c
      })
    }
    return data
  }
}
