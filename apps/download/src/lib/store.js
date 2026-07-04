// 全站共享的资源索引：整棵树只从后端拉一次，各页面共用同一份响应式状态。
import { reactive } from 'vue'
import { getIndex } from '@/api'

const state = reactive({
  loading: false,
  loaded: false,
  error: '',
  /** DownloadIndex：{ title, description, sourceUrl, syncedAt, contentUpdatedAt, stats, links, tree } */
  data: null,
})

let inflight = null

/** 首次调用触发拉取；重复调用复用同一请求 / 已加载则直接返回。 */
export function ensureIndex() {
  if (state.loaded) return Promise.resolve()
  if (inflight) return inflight
  state.loading = true
  state.error = ''
  inflight = getIndex()
    .then((d) => {
      state.data = d
      state.loaded = true
    })
    .catch((e) => {
      state.error = e?.message || '资源索引加载失败'
    })
    .finally(() => {
      state.loading = false
      inflight = null
    })
  return inflight
}

/** 强制重新拉取（如手动刷新）。 */
export function reloadIndex() {
  state.loaded = false
  inflight = null
  return ensureIndex()
}

export function useIndex() {
  return state
}
