import { defineStore } from 'pinia'
import { api } from '../services/api.js'

const SECTION_KEYS = ['recommended', 'popular', 'latest', 'personal']

function sectionParams(section, seed) {
  const params = { status: 'approved', page: 1, pageSize: 8 }
  if (section === 'recommended') return { ...params, sort: 'recommended', seed }
  if (section === 'popular') return { ...params, sort: 'popular', order: 'desc', range: 'week' }
  if (section === 'personal') {
    return { ...params, sort: 'time', order: 'desc', source_type: 'personal' }
  }
  return { ...params, sort: 'time', order: 'desc' }
}

function hasPopularityStats(items) {
  return (items || []).some(item => (
    item?.popularity_score !== undefined
    || item?.popularity?.score !== undefined
  ))
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
    sections: {
      recommended: [],
      popular: [],
      latest: [],
      personal: [],
    },
    sectionsLoading: {
      recommended: false,
      popular: false,
      latest: false,
      personal: false,
    },
    sectionsError: '',
    creatorExhibits: [],
    creatorExhibitsLoading: false,
    recommendationBatchId: '',
    recommendationsPersonalized: false,
    sectionReqIds: {
      recommended: 0,
      popular: 0,
      latest: 0,
      personal: 0,
    },
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

    async loadSection(section) {
      if (!SECTION_KEYS.includes(section)) return
      const currentReqId = ++this.sectionReqIds[section]
      this.sectionsLoading[section] = true
      this.sectionsError = ''

      try {
        let out = section === 'recommended'
          ? await api.recommendations(8)
          : await api.artworksList(sectionParams(section, this.randomSeed))
        if (section === 'popular' && !hasPopularityStats(out.data)) {
          out = await api.artworksList({
            ...sectionParams(section, this.randomSeed),
            sort: 'likes',
            range: undefined,
          })
        }
        if (this.sectionReqIds[section] !== currentReqId) return
        this.sections[section] = (out.data || []).slice(0, 8)
        if (section === 'recommended') {
          this.recommendationBatchId = out.batchId || ''
          this.recommendationsPersonalized = Boolean(out.personalized)
        }
      } catch (error) {
        if (this.sectionReqIds[section] !== currentReqId) return
        this.sections[section] = []
        if (section === 'recommended') {
          this.recommendationBatchId = ''
          this.recommendationsPersonalized = false
        }
        this.sectionsError = '作品加载失败，请刷新后重试'
        console.warn(`[Gallery] ${section} 区块加载失败：`, error)
      } finally {
        if (this.sectionReqIds[section] === currentReqId) {
          this.sectionsLoading[section] = false
        }
      }
    },

    async loadSections() {
      await Promise.all([
        ...SECTION_KEYS.map(section => this.loadSection(section)),
        this.loadCreatorExhibits(),
      ])
    },

    async loadCreatorExhibits() {
      this.creatorExhibitsLoading = true
      try {
        const out = await api.creatorExhibits()
        this.creatorExhibits = out.data || []
      } catch (error) {
        this.creatorExhibits = []
        this.sectionsError = '创作者展位加载失败，请刷新后重试'
        console.warn('[Gallery] 创作者展位加载失败：', error)
      } finally {
        this.creatorExhibitsLoading = false
      }
    },

    async refreshRecommendations() {
      this.randomSeed = Math.floor(Math.random() * 2147483647)
      await this.loadSection('recommended')
    },

    async likeArtwork(item) {
      if (!item) return
      const id = Number(item.id)
      const oldVal = item.like_total
      item.like_total = Number(oldVal || 0) + 1

      try {
        const out = await api.likeArtwork(id)
        if (out && out.totalLikes !== undefined) {
          item.like_total = Number(out.totalLikes)
        }
      } catch (error) {
        item.like_total = oldVal
        console.error('Like failed:', error)
      }
    },

    async fetchArtworkById(id) {
      if (!id) return null

      const loadedItems = [
        ...this.list,
        ...Object.values(this.sections).flat(),
        ...this.creatorExhibits.flatMap(group => group.items || []),
      ]
      const existing = loadedItems.find(item => String(item.id) === String(id))

      try {
        const response = await api.getArtwork(id)
        if (response.ok && response.data) {
          if (existing) Object.assign(existing, response.data)
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
