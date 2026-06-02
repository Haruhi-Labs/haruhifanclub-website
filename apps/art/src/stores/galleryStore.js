import { defineStore } from 'pinia'
import { api } from '../services/api.js'
import { seedArtworks } from '../mock/seedData.js'

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

function applyLocalSort(arr, sortMode, seed) {
  const out = Array.isArray(arr) ? [...arr] : []
  if (sortMode === 'likes') {
    out.sort((a, b) => {
      const la = Number(a?.like_total || 0)
      const lb = Number(b?.like_total || 0)
      if (lb !== la) return lb - la
      const ta = String(a?.created_at || '')
      const tb = String(b?.created_at || '')
      if (tb !== ta) return tb.localeCompare(ta)
      return Number(b?.id || 0) - Number(a?.id || 0)
    })
    return out
  }
  if (sortMode === 'time') {
    out.sort((a, b) => {
      const ta = String(a?.created_at || '')
      const tb = String(b?.created_at || '')
      if (tb !== ta) return tb.localeCompare(ta)
      return Number(b?.id || 0) - Number(a?.id || 0)
    })
    return out
  }
  // random
  out.sort((a, b) => stableRandKey(a?.id || 0, seed) - stableRandKey(b?.id || 0, seed))
  return out
}

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    content: 'mix',
    sourceMode: 'all', // all | personal | network （balanced 若存在会按 all 处理）

    sortMode: 'time', // random | likes | time
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

    reqId: 0
  }),

  actions: {
    setFilters(patch) {
      if (patch.content !== undefined) this.content = patch.content
      if (patch.sourceMode !== undefined) this.sourceMode = patch.sourceMode
      if (patch.sortMode !== undefined) {
        this.sortMode = patch.sortMode
        if (patch.sortMode === 'random') {
          this.randomSeed = Math.floor(Math.random() * 2147483647)
        }
      }
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

      try {
        // ✅ 统一走后端排序（跨页）
        const params = {
          status: 'approved',
          // content_type: this.content, // moved to logic below
          q: this.q,
          searchField: this.searchField,
          page: this.page,
          pageSize: this.limit
        }

        if (this.content !== 'mix') {
          params.content_type = this.content
        }

        console.log('[GalleryStore] loading...', { sort: this.sortMode, seed: this.randomSeed, reqId: currentReqId })

        // balanced 若存在，后端会当 all；这里也直接不传 source_type，保证全局排序正确
        if (this.sourceMode === 'personal' || this.sourceMode === 'network') {
          params.source_type = this.sourceMode
        }

        // ✅ sort 映射到后端：time | likes | random
        if (this.sortMode === 'likes') {
          params.sort = 'likes'
          params.order = 'desc'
        } else if (this.sortMode === 'time') {
          params.sort = 'time'
          params.order = 'desc'
        } else {
          params.sort = 'random'
          params.seed = this.randomSeed
        }

        const out = await api.artworksList(params)

        if (this.reqId !== currentReqId) return

        this.list = out.data || []
        this.total = Number(out.total || 0)
        this.hasMore = (this.page * this.limit) < this.total

        // Seed 兜底（同样支持跨页/排序）
        if (this.list.length === 0 && this.page === 1 && !String(this.q || '').trim() && this.total === 0) {
          const localAll = filterLocal(seedArtworks, {
            content: this.content,
            sourceMode: this.sourceMode,
            q: this.q,
            searchField: this.searchField
          })

          const sortedLocal = applyLocalSort(localAll, this.sortMode, this.randomSeed)
          const pg = paginate(sortedLocal, this.page, this.limit)

          this.list = pg.data
          this.total = pg.total
          this.hasMore = (this.page * this.limit) < this.total
          if (this.list.length > 0) this.usingSeed = true
        }

      } catch (e) {
        if (this.reqId !== currentReqId) return

        console.warn('API Error, falling back to seed data:', e)
        this.error = ''

        const localAll = filterLocal(seedArtworks, {
          content: this.content,
          sourceMode: this.sourceMode,
          q: this.q,
          searchField: this.searchField
        })

        const sortedLocal = applyLocalSort(localAll, this.sortMode, this.randomSeed)
        const pg = paginate(sortedLocal, this.page, this.limit)

        this.list = pg.data
        this.total = pg.total
        this.hasMore = (this.page * this.limit) < this.total
        this.usingSeed = true

      } finally {
        if (this.reqId === currentReqId) this.loading = false
      }
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

      // 1. Try to find in current list first
      let existing = this.list.find(i => String(i.id) === String(id))

      // If we found it, but it looks "incomplete" (missing images array for a multi-image post),
      // or we just want to be sure, we might want to fetch details.
      // For now, let's say if `images` is missing or empty, we fetch. 
      // (Note: simple posts might strictly have no images array if the backend doesn't send it, 
      // but usually the backend should send it if we ask for detail).
      if (existing) {
        // If it already has images populated, return it directly
        if (Array.isArray(existing.images) && existing.images.length > 0) {
          return existing
        }
        // If it's a seed data item (negative ID usually or just local), we might not be able to fetch
        // But for real items, we should fetch.
      }

      // 2. Fetch from API
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
