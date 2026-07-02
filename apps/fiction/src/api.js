// 同人文库前端统一数据层：封装 /api/fiction 客户端、会话与上传 URL。
import { createApiClient, resolveUploadUrl } from '@haruhi/api-client'
import { useSession } from '@haruhi/auth-ui'

// 站内业务接口
export const api = createApiClient('/api/fiction')
// 跨模块个人总览（/api/me/summary 等）
export const meApi = createApiClient('/api/me')
// 统一账号会话（与 AccountMenu / 路由守卫共享同一实例）
export const session = useSession('/api')

/** 上传相对路径 → 可访问 URL；空值返回 null。 */
export function coverUrl(path) {
  return path ? resolveUploadUrl(path) : null
}

// ---- 公开读 ----
export const getSpotlight = () => api.get('/spotlight')
export const getCategories = () => api.get('/categories')
export const getTags = (limit = 40) => api.get(`/tags?limit=${limit}`)
export const listStories = (params = {}) => api.get(`/stories?${qs(params)}`)
export const getStory = (id) => api.get(`/stories/${id}`)
export const getChapter = (id, cid) => api.get(`/stories/${id}/chapters/${cid}`)
export const bumpView = (id) => api.post(`/stories/${id}/views`)
export const listComments = (id, params = {}) => api.get(`/stories/${id}/comments?${qs(params)}`)

// ---- 互动（需登录）----
export const toggleLike = (id) => api.post(`/stories/${id}/like`)
export const toggleBookmark = (id) => api.post(`/stories/${id}/bookmark`)
export const postComment = (id, body) => api.post(`/stories/${id}/comments`, body)
export const deleteMyComment = (cid) => api.del(`/me/comments/${cid}`)
export const getProgress = (id) => api.get(`/me/progress/${id}`)
export const saveProgress = (id, body) => api.put(`/me/progress/${id}`, body)

// ---- 个人中心 / 创作 ----
export const myStories = () => api.get('/me/stories')
export const myStory = (id) => api.get(`/me/stories/${id}`)
export const myChapter = (id, cid) => api.get(`/me/stories/${id}/chapters/${cid}`)
export const myBookmarks = (params = {}) => api.get(`/me/bookmarks?${qs(params)}`)
export const myComments = (params = {}) => api.get(`/me/comments?${qs(params)}`)
export const myStats = () => api.get('/me/stats')
export const createStory = (body) => api.post('/me/stories', body)
export const updateStory = (id, body) => api.patch(`/me/stories/${id}`, body)
export const deleteStory = (id) => api.del(`/me/stories/${id}`)
export const publishStory = (id) => api.post(`/me/stories/${id}/publish`)
export const unpublishStory = (id) => api.post(`/me/stories/${id}/unpublish`)
export const createChapter = (id, body) => api.post(`/me/stories/${id}/chapters`, body)
export const updateChapter = (id, cid, body) => api.patch(`/me/stories/${id}/chapters/${cid}`, body)
export const deleteChapter = (id, cid) => api.del(`/me/stories/${id}/chapters/${cid}`)
export const reorderChapters = (id, order) => api.post(`/me/stories/${id}/chapters/reorder`, { order })

/** 上传封面（File），返回 { path }。 */
export function uploadCover(file) {
  const fd = new FormData()
  fd.append('file', file, (file && file.name) || 'cover.png')
  return api.postForm('/me/covers', fd)
}

function qs(params) {
  const s = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== '') s.set(k, v)
  }
  return s.toString()
}
