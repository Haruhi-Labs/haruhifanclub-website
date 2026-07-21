import { defineStore } from 'pinia'
import { api } from '../services/api.js'

function hasPopularityStats(items) {
  return (items || []).some(item => (
    item?.popularity_score !== undefined
    || item?.popularity?.score !== undefined
  ))
}

function engagementSnapshot(item) {
  return {
    likeTotal: item?.like_total,
    popularityLikes: item?.popularity?.likes,
    liked: item?.liked,
    likesToday: item?.likes_today,
    likesRemainingToday: item?.likes_remaining_today,
    dailyLikeLimit: item?.daily_like_limit,
  }
}

function restoreEngagement(item, snapshot) {
  if (!item || !snapshot) return
  item.like_total = snapshot.likeTotal
  if (item.popularity) item.popularity.likes = snapshot.popularityLikes
  item.liked = snapshot.liked
  item.likes_today = snapshot.likesToday
  item.likes_remaining_today = snapshot.likesRemainingToday
  item.daily_like_limit = snapshot.dailyLikeLimit
}

function applyEngagement(item, payload = {}) {
  if (!item) return
  const total = payload.totalLikes ?? payload.total_likes ?? payload.like_total
  const likesToday = payload.likesToday ?? payload.likes_today
  const dailyLimit = payload.dailyLikeLimit ?? payload.daily_like_limit
  const remaining = payload.remainingLikesToday ?? payload.likes_remaining_today

  if (total !== undefined) {
    item.like_total = Number(total)
    if (item.popularity) item.popularity.likes = Number(total)
  }
  if (payload.liked !== undefined) item.liked = Boolean(payload.liked)
  if (likesToday !== undefined) item.likes_today = Math.max(0, Number(likesToday))
  if (dailyLimit !== undefined) item.daily_like_limit = Math.max(1, Number(dailyLimit))
  if (remaining !== undefined) {
    item.likes_remaining_today = Math.max(0, Number(remaining))
  } else if (likesToday !== undefined || dailyLimit !== undefined) {
    item.likes_remaining_today = Math.max(
      0,
      Number(item.daily_like_limit || 10) - Number(item.likes_today || 0),
    )
  }
}

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    content: 'mix',
    sourceMode: 'all',
    sortMode: 'recommended',
    timeRange: 'history',
    randomSeed: Math.floor(Math.random() * 2147483647),
    q: '',
    searchField: 'all',
    page: 1,
    limit: 12,
    loading: false,
    error: '',
    list: [],
    total: 0,
    hasMore: false,
    artworkCache: {},
    reqId: 0,
  }),

  actions: {
    setFilters(patch) {
      if (patch.content !== undefined) this.content = patch.content
      if (patch.sourceMode !== undefined) this.sourceMode = patch.sourceMode
      if (patch.sortMode !== undefined) {
        this.sortMode = patch.sortMode
        if (patch.sortMode === 'recommended' || patch.sortMode === 'random') {
          this.randomSeed = Math.floor(Math.random() * 2147483647)
        }
      }
      if (patch.timeRange !== undefined) this.timeRange = patch.timeRange
      if (patch.q !== undefined) this.q = patch.q
      if (patch.searchField !== undefined) this.searchField = patch.searchField
      if (patch.page !== undefined) this.page = patch.page
      if (patch.limit !== undefined) this.limit = patch.limit
    },

    async load() {
      const currentReqId = ++this.reqId
      this.loading = true
      this.error = ''

      const params = {
        status: 'approved',
        q: this.q,
        searchField: this.searchField,
        page: this.page,
        pageSize: this.limit,
      }
      if (this.content !== 'mix') params.content_type = this.content
      if (this.sourceMode === 'personal' || this.sourceMode === 'network') {
        params.source_type = this.sourceMode
      }
      if (this.sortMode === 'popular' || this.sortMode === 'likes') {
        params.sort = 'popular'
        params.order = 'desc'
        params.range = this.timeRange
      } else if (this.sortMode === 'time') {
        params.sort = 'time'
        params.order = 'desc'
      } else {
        params.sort = 'recommended'
        params.seed = this.randomSeed
      }

      let out = null
      let lastError = null
      for (let attempt = 0; attempt < 2; attempt += 1) {
        try {
          out = await api.artworksList(params)
          if (params.sort === 'popular' && !hasPopularityStats(out.data)) {
            out = await api.artworksList({ ...params, sort: 'likes', range: undefined })
          }
          break
        } catch (error) {
          if (this.reqId !== currentReqId) return
          lastError = error
          if (attempt === 0) await new Promise(resolve => setTimeout(resolve, 500))
        }
      }

      if (this.reqId !== currentReqId) return
      if (!out) {
        this.list = []
        this.total = 0
        this.hasMore = false
        this.error = '作品加载失败，请刷新后重试'
        this.loading = false
        console.warn('[Gallery] 作品加载失败（已重试）：', lastError)
        return
      }

      this.list = out.data || []
      this.total = Number(out.total || 0)
      this.hasMore = (this.page * this.limit) < this.total
      this.loading = false
    },

    async likeArtwork(item) {
      if (!item) return null
      const id = Number(item.id)
      if (!Number.isFinite(id)) return null

      const key = String(id)
      const targets = new Set([
        item,
        this.artworkCache[key],
        ...this.list.filter(value => String(value?.id) === key),
      ].filter(Boolean))
      const snapshots = new Map([...targets].map(target => [target, engagementSnapshot(target)]))
      const before = Number(item.like_total ?? item.popularity?.likes ?? 0)
      const used = Number(item.likes_today || 0) + 1
      const limit = Number(item.daily_like_limit || 10)
      const optimistic = {
        totalLikes: before + 1,
        liked: true,
        likesToday: used,
        remainingLikesToday: Math.max(0, limit - used),
        dailyLikeLimit: limit,
      }
      targets.forEach(target => applyEngagement(target, optimistic))

      try {
        const out = await api.likeArtwork(id)
        targets.forEach(target => applyEngagement(target, out))
        return out
      } catch (error) {
        if (error?.data?.dailyLikeLimit !== undefined) {
          targets.forEach(target => applyEngagement(target, error.data))
        } else {
          snapshots.forEach((snapshot, target) => restoreEngagement(target, snapshot))
        }
        console.error('[Gallery] 点赞失败：', error)
        throw error
      }
    },

    cachedArtworkById(id) {
      if (!id) return null

      const key = String(id)
      if (this.artworkCache[key]) return this.artworkCache[key]

      const loadedItems = [...this.list]
      const existing = loadedItems.find(item => String(item.id) === key) || null
      if (existing) this.artworkCache[key] = existing
      return existing
    },

    rememberArtwork(item) {
      if (!item?.id) return null
      const key = String(item.id)
      const existing = this.artworkCache[key]
      if (existing) {
        Object.assign(existing, item)
        return existing
      }
      this.artworkCache[key] = item
      return item
    },

    async fetchArtworkById(id) {
      if (!id) return null

      const existing = this.cachedArtworkById(id)

      try {
        const response = await api.getArtwork(id)
        if (response.ok && response.data) {
          if (existing) {
            Object.assign(existing, response.data)
            return existing
          }
          return response.data
        }
      } catch (error) {
        console.error('Fetch specific artwork failed:', error)
      }

      // 缓存只用于合并最新响应，不能在 404、撤审或网络失败时冒充权威详情。
      return null
    },
  },
})
