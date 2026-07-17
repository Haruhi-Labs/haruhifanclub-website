import { defineStore } from 'pinia'
import { api } from '../services/api.js'
import { seedArtworks, seedComments } from '../mock/seedData.js'

const USE_DEV_SEED_ONLY = import.meta.env.DEV && import.meta.env.VITE_ART_USE_BACKEND !== '1'
const SECTION_KEYS = ['recommended', 'popular', 'latest', 'personal']

function norm(s) { return String(s || '').toLowerCase() }
function includesWord(hay, q) { return norm(hay).includes(norm(q)) }

function filterLocal(items, { content = 'mix', sourceMode = 'all', q = '', searchField = 'all' }) {
  let arr = items.filter(x => x.status === 'approved')

  if (content === 'haruhi' || content === 'other') {
    arr = arr.filter(x => x.content_type === content)
  }
  // 'mix' -> no filter (show all)

  if (sourceMode === 'personal' || sourceMode === 'network') {
    arr = arr.filter(x => x.source_type === sourceMode)
  }

  const qq = String(q || '').trim()
  if (qq) {
    arr = arr.filter(x => {
      const tags = Array.isArray(x.tags) ? x.tags.join(' ') : ''
      const title = x.title || ''
      const desc = x.description || ''
      const uid = x.uploader_uid || ''
      const name = x.uploader_name || ''

      if (searchField === 'title') return includesWord(title, qq)
      if (searchField === 'uid') return includesWord(uid, qq) || includesWord(name, qq)
      if (searchField === 'tag') return includesWord(tags, qq)

      return (
        includesWord(title, qq) ||
        includesWord(desc, qq) ||
        includesWord(tags, qq) ||
        includesWord(uid, qq) ||
        includesWord(name, qq)
      )
    })
  }

  return arr
}

function paginate(arr, page, pageSize) {
  const total = arr.length
  const start = (page - 1) * pageSize
  return { total, data: arr.slice(start, start + pageSize) }
}

// 本地稳定随机排序（fallback 用）
function stableRandKey(id, seed) {
  // XOR then two rounds of multiply — matches backend SQL hash
  const xor = ((id | 0) + (seed | 0) - 2 * ((id | 0) & (seed | 0))) >>> 0
  const h1 = Math.imul(xor, 2654435761) >>> 0
  return (Math.imul((h1 % 2147483647) + 1, 1103515245) >>> 0) % 2147483647
}

function wilsonLowerBound(successes, trials) {
  if (trials <= 0) return 0
  const z = 1.96
  const z2 = z * z
  const p = Math.min(Math.max(successes / trials, 0), 1)
  return Math.max(0, (
    p + z2 / (2 * trials)
    - z * Math.sqrt((p * (1 - p) + z2 / (4 * trials)) / trials)
  ) / (1 + z2 / trials))
}

function localPopularity(item, commentCounts, range) {
  const likes = Math.max(0, Number(item?.like_total || 0))
  const comments = Math.max(0, Number(commentCounts.get(String(item?.id)) || 0))
  const baseViews = Math.max(0, Number(item?.view_total || likes * 14 + comments * 32 + 24 + Math.abs(Number(item?.id || 0) % 73)))
  const views = range === 'week' ? Math.max(1, Math.round(baseViews * 0.36)) : baseViews
  const audience = Math.max(views, likes * 5, comments * 12, 1)
  const score = Math.round(100 * Math.log1p(audience) * (
    1 + 3 * wilsonLowerBound(likes, audience) + 6 * wilsonLowerBound(comments, audience)
  ))
  return { score, views, likes, comments, range }
}

function applyLocalSort(arr, sortMode, seed, timeRange = 'history') {
  const publicCommentCounts = seedComments.reduce((counts, comment) => {
    if (comment?.status && comment.status !== 'public') return counts
    const artworkId = String(comment?.artwork_id ?? '')
    counts.set(artworkId, (counts.get(artworkId) || 0) + 1)
    return counts
  }, new Map())
  const out = (Array.isArray(arr) ? arr : []).map(item => {
    const popularity = localPopularity(item, publicCommentCounts, timeRange)
    return { ...item, popularity, popularity_score: popularity.score }
  })
  if (sortMode === 'popular' || sortMode === 'likes') {
    out.sort((a, b) => {
      const scoreA = Number(a?.popularity_score || 0)
      const scoreB = Number(b?.popularity_score || 0)
      if (scoreB !== scoreA) return scoreB - scoreA
      const ta = String(a?.reviewed_at || a?.created_at || '')
      const tb = String(b?.reviewed_at || b?.created_at || '')
      if (tb !== ta) return tb.localeCompare(ta)
      return Number(b?.id || 0) - Number(a?.id || 0)
    })
    return out
  }
  if (sortMode === 'time') {
    out.sort((a, b) => {
      const ta = String(a?.reviewed_at || a?.created_at || '')
      const tb = String(b?.reviewed_at || b?.created_at || '')
      if (tb !== ta) return tb.localeCompare(ta)
      return Number(b?.id || 0) - Number(a?.id || 0)
    })
    return out
  }
  // 推荐算法接入前使用稳定随机批次；同一种子翻页不会重复洗牌。
  out.sort((a, b) => stableRandKey(a?.id || 0, seed) - stableRandKey(b?.id || 0, seed))
  return out
}

function sectionParams(section, seed) {
  const params = { status: 'approved', page: 1, pageSize: 8 }
  if (section === 'recommended') return { ...params, sort: 'recommended', seed }
  if (section === 'popular') return { ...params, sort: 'popular', order: 'desc', range: 'week' }
  if (section === 'personal') {
    return { ...params, sort: 'time', order: 'desc', source_type: 'personal' }
  }
  return { ...params, sort: 'time', order: 'desc' }
}

function localSectionItems(section, seed) {
  const sourceMode = section === 'personal' ? 'personal' : 'all'
  const filtered = filterLocal(seedArtworks, { sourceMode })
  const sortMode = section === 'latest' || section === 'personal' ? 'time' : section
  return applyLocalSort(filtered, sortMode, seed, section === 'popular' ? 'week' : 'history').slice(0, 8)
}

function localCreatorExhibits(items, seed) {
  const personal = filterLocal(items, { sourceMode: 'personal' })
  const ranked = applyLocalSort(personal, 'popular', seed, 'history')
  const groups = new Map()
  for (const item of ranked) {
    const uid = item.uploader_uid || item.uploader_name || 'gallery'
    if (!groups.has(uid)) {
      groups.set(uid, {
        uid,
        name: item.uploader_display_name || item.uploader_name || uid,
        avatar: item.uploader_avatar || '',
        items: [],
      })
    }
    if (groups.get(uid).items.length < 3) groups.get(uid).items.push(item)
  }
  return [...groups.values()]
}

function hasPopularityStats(items) {
  return (items || []).some(item => (
    item?.popularity_score !== undefined
    || item?.popularity?.score !== undefined
  ))
}

async function fetchLegacyCreatorExhibits(seed) {
  const pageSize = 60
  const items = []
  let page = 1
  let total = Number.POSITIVE_INFINITY

  while (items.length < total && page <= 20) {
    const out = await api.artworksList({
      status: 'approved',
      source_type: 'personal',
      sort: 'likes',
      order: 'desc',
      page,
      pageSize,
    })
    const pageItems = out.data || []
    items.push(...pageItems)
    total = Number(out.total || items.length)
    if (pageItems.length < pageSize) break
    page += 1
  }

  return localCreatorExhibits(items, seed)
}

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    content: 'mix',
    sourceMode: 'all', // all | personal | network （balanced 若存在会按 all 处理）

    sortMode: 'recommended', // recommended | popular | time
    timeRange: 'history', // week | year | history
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

    usingSeed: false,

    sections: {
      recommended: [],
      popular: [],
      latest: [],
      personal: []
    },
    sectionsLoading: {
      recommended: false,
      popular: false,
      latest: false,
      personal: false
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
      personal: 0
    },

    reqId: 0
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
      this.usingSeed = false

      // 统一走后端排序（跨页）
      const params = {
        status: 'approved',
        q: this.q,
        searchField: this.searchField,
        page: this.page,
        pageSize: this.limit
      }
      if (this.content !== 'mix') params.content_type = this.content
      // balanced 若存在按 all 处理：不传 source_type，保证全局排序正确
      if (this.sourceMode === 'personal' || this.sourceMode === 'network') params.source_type = this.sourceMode
      if (this.sortMode === 'popular' || this.sortMode === 'likes') {
        params.sort = 'popular'
        params.order = 'desc'
        params.range = this.timeRange
      }
      else if (this.sortMode === 'time') { params.sort = 'time'; params.order = 'desc' }
      else { params.sort = 'recommended'; params.seed = this.randomSeed }

      const seedFallback = () => {
        // 仅本地开发（无后端）才用 seed 假数据占位；生产绝不展示占位作品
        const localAll = filterLocal(seedArtworks, {
          content: this.content, sourceMode: this.sourceMode, q: this.q, searchField: this.searchField
        })
        const pg = paginate(
          applyLocalSort(localAll, this.sortMode, this.randomSeed, this.timeRange),
          this.page,
          this.limit,
        )
        this.list = pg.data
        this.total = pg.total
        this.hasMore = (this.page * this.limit) < this.total
        this.usingSeed = this.list.length > 0
      }

      // 失败时静默重试一次，缓解移动端首屏偶发的瞬时网络失败（避免一抖动就掉到占位）
      if (USE_DEV_SEED_ONLY) {
        seedFallback()
        this.loading = false
        return
      }

      let out = null
      const maxAttempts = import.meta.env.DEV ? 1 : 2
      for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
          out = await api.artworksList(params)
          if (params.sort === 'popular' && !hasPopularityStats(out.data)) {
            out = await api.artworksList({ ...params, sort: 'likes', range: undefined })
          }
          break
        } catch (e) {
          if (this.reqId !== currentReqId) return
          if (import.meta.env.DEV) {
            seedFallback()
            this.loading = false
            return
          }
          if (attempt === 0) { await new Promise(r => setTimeout(r, 500)); continue }
          // 两次都失败
          console.warn('[Gallery] 作品加载失败（已重试）：', e)
          if (import.meta.env.DEV) {
            seedFallback()
          } else {
            this.list = []; this.total = 0; this.hasMore = false
            this.error = '作品加载失败，请刷新后重试'
          }
          this.loading = false
          return
        }
      }

      if (this.reqId !== currentReqId) return

      this.list = out.data || []
      this.total = Number(out.total || 0)
      this.hasMore = (this.page * this.limit) < this.total

      // 接口成功但确无已通过作品：仅本地开发用 seed 占位，生产显示真实空态
      if (import.meta.env.DEV && this.list.length === 0 && this.page === 1 && !String(this.q || '').trim() && this.total === 0) {
        seedFallback()
      }

      this.loading = false
    },

    async loadSection(section) {
      if (!SECTION_KEYS.includes(section)) return
      const currentReqId = ++this.sectionReqIds[section]
      this.sectionsLoading[section] = true
      this.sectionsError = ''

      if (USE_DEV_SEED_ONLY) {
        this.sections[section] = localSectionItems(section, this.randomSeed)
        if (section === 'recommended') {
          this.recommendationBatchId = `dev-${this.randomSeed}`
          this.recommendationsPersonalized = false
        }
        this.sectionsLoading[section] = false
        return
      }

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
        if (import.meta.env.DEV) {
          try {
            const fallback = await api.artworksList({
              ...sectionParams(section, this.randomSeed),
              sort: section === 'recommended' ? 'random' : sectionParams(section, this.randomSeed).sort,
            })
            this.sections[section] = (fallback.data || []).slice(0, 8)
            if (section === 'recommended') {
              this.recommendationBatchId = `legacy-${this.randomSeed}`
              this.recommendationsPersonalized = false
            }
          } catch {
            this.sections[section] = localSectionItems(section, this.randomSeed)
            if (section === 'recommended') {
              this.recommendationBatchId = `dev-${this.randomSeed}`
              this.recommendationsPersonalized = false
            }
          }
        } else {
          this.sections[section] = []
          if (section === 'recommended') this.recommendationBatchId = ''
          this.sectionsError = '作品加载失败，请刷新后重试'
          console.warn(`[Gallery] ${section} 区块加载失败：`, error)
        }
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
      if (USE_DEV_SEED_ONLY) {
        this.creatorExhibits = localCreatorExhibits(seedArtworks, this.randomSeed)
        this.creatorExhibitsLoading = false
        return
      }
      try {
        const out = await api.creatorExhibits()
        this.creatorExhibits = out.data || []
        if (import.meta.env.DEV && !this.creatorExhibits.length) {
          this.creatorExhibits = await fetchLegacyCreatorExhibits(this.randomSeed)
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          try {
            this.creatorExhibits = await fetchLegacyCreatorExhibits(this.randomSeed)
          } catch {
            this.creatorExhibits = localCreatorExhibits(seedArtworks, this.randomSeed)
          }
        } else {
          this.creatorExhibits = []
          console.warn('[Gallery] 创作者展位加载失败：', error)
        }
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
      } catch (e) {
        item.like_total = oldVal
        console.error('Like failed:', e)
      }
    },

    async fetchArtworkById(id) {
      if (!id) return null

      // 1. Try every already-loaded gallery surface before requesting detail.
      // Read-only live-data preview blocks the stateful detail GET, so this also
      // keeps modal viewing usable without changing production analytics.
      const loadedItems = [
        ...this.list,
        ...Object.values(this.sections).flat(),
        ...this.creatorExhibits.flatMap(group => group.items || []),
      ]
      const existing = loadedItems.find(i => String(i.id) === String(id))

      if (USE_DEV_SEED_ONLY) {
        return existing || seedArtworks.find(i => String(i.id) === String(id)) || null
      }

      // 2. Fetch from API. Opening detail must hit the backend so guild browse quests can record progress.
      try {
        const res = await api.getArtwork(id)
        if (res.ok && res.data) {
          // Update the list item if it exists, so the grid also gets updated info if needed
          if (existing) {
            Object.assign(existing, res.data)
          }
          return res.data
        }
      } catch (e) {
        console.error('Fetch specific artwork failed:', e)
      }

      // Fallback: if API failed but we have existing (incomplete) item, return that
      return existing || null
    }
  }
})
