// 个人中心数据访问层：统一封装各模块 /me/* 与跨库 /me/summary 端点。
// 全部走 @haruhi/api-client（credentials:'include' + 自动注入 CSRF），各 app 复用同一份。
//
// 用法：
//   const hub = useUserHub('/api')
//   const s = await hub.summary()
//   const { data, total } = await hub.art.artworks({ status: 'all', page: 1 })

import { createApiClient } from '@haruhi/api-client'

function qs(params) {
  if (!params) return ''
  const sp = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v != null && v !== '') sp.set(k, String(v))
  }
  const s = sp.toString()
  return s ? `?${s}` : ''
}

export function useUserHub(apiBase = '/api') {
  const api = createApiClient(apiBase)

  return {
    // 跨模块总览（个人控制台概览页）
    summary: () => api.get('/me/summary'),

    // 画廊（art）
    art: {
      artworks: (params) => api.get(`/art/me/artworks${qs(params)}`),
      comments: (params) => api.get(`/art/me/comments${qs(params)}`),
      points: () => api.get('/art/me/points'),
      updateArtwork: (id, patch) => api.patch(`/art/me/artworks/${id}`, patch),
      deleteArtwork: (id) => api.del(`/art/me/artworks/${id}`),
      deleteComment: (id) => api.del(`/art/me/comments/${id}`),
      claim: () => api.post('/art/claim', {}),
    },

    // 团报（news）
    news: {
      articles: () => api.get('/news/me/articles'),
      points: () => api.get('/news/me/points'),
      redemptions: () => api.get('/news/me/redemptions'),
      updateArticle: (id, patch) => api.put(`/news/me/articles/${id}`, patch),
      deleteArticle: (id) => api.del(`/news/me/articles/${id}`),
      prizes: () => api.get('/news/prizes'),
      redeem: (id) => api.post(`/news/prizes/${id}/redeem`, {}),
    },

    // 注：exam（试卷站）已从用户系统剥离，不在此聚合。
  }
}
