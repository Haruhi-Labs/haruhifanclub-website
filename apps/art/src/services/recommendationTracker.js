import { api } from './api.js'

const SESSION_KEY = 'haruhi-art-recommendation-session'
const FLUSH_DELAY_MS = 900
const MAX_QUEUE_SIZE = 40

const queue = []
let flushTimer = 0
let flushing = false
let activeView = null
const contextByArtwork = new Map()

function sessionId() {
  if (typeof window === 'undefined') return ''
  let value = window.sessionStorage.getItem(SESSION_KEY)
  if (!value) {
    value = window.crypto?.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`
    window.sessionStorage.setItem(SESSION_KEY, value)
  }
  return value
}

function normalizeContext(item, context = {}) {
  const recommendation = item?.recommendation || {}
  return {
    batch_id: context.batchId || recommendation.batch_id || undefined,
    source: context.source || 'gallery',
    position: Number.isFinite(context.position)
      ? context.position
      : Number.isFinite(recommendation.position) ? recommendation.position : undefined,
  }
}

function scheduleFlush() {
  if (typeof window === 'undefined' || flushTimer) return
  flushTimer = window.setTimeout(() => {
    flushTimer = 0
    flushRecommendationEvents()
  }, FLUSH_DELAY_MS)
}

export function queueRecommendationEvent(event) {
  const artworkId = Number(event?.artwork_id)
  if (!Number.isFinite(artworkId) || artworkId <= 0) return
  queue.push({ ...event, artwork_id: artworkId })
  if (queue.length >= MAX_QUEUE_SIZE) flushRecommendationEvents()
  else scheduleFlush()
}

export async function flushRecommendationEvents(keepalive = false) {
  if (flushing || queue.length === 0) return
  if (flushTimer && typeof window !== 'undefined') window.clearTimeout(flushTimer)
  flushTimer = 0
  const events = queue.splice(0, MAX_QUEUE_SIZE)
  flushing = true
  try {
    await api.recordRecommendationEvents(events, sessionId(), keepalive)
  } catch (error) {
    if (!keepalive) console.warn('[Gallery] 推荐行为上报失败：', error)
  } finally {
    flushing = false
    if (queue.length) scheduleFlush()
  }
}

export function trackArtworkImpression(item, context) {
  const normalized = normalizeContext(item, context)
  queueRecommendationEvent({
    artwork_id: item?.id,
    event_type: 'impression',
    ...normalized,
  })
}

export function trackArtworkOpen(item, context) {
  const normalized = normalizeContext(item, context)
  contextByArtwork.set(String(item?.id), normalized)
  queueRecommendationEvent({
    artwork_id: item?.id,
    event_type: 'open',
    ...normalized,
  })
}

function pauseActiveView() {
  if (!activeView?.runningSince) return
  activeView.elapsed += performance.now() - activeView.runningSince
  activeView.runningSince = 0
}

function resumeActiveView() {
  if (!activeView || activeView.runningSince || document.hidden) return
  activeView.runningSince = performance.now()
}

export function startArtworkView(item) {
  if (!item?.id) return
  if (activeView && String(activeView.artworkId) !== String(item.id)) finishArtworkView()
  if (activeView) return
  const context = contextByArtwork.get(String(item.id)) || normalizeContext(item, { source: 'direct' })
  activeView = {
    artworkId: Number(item.id),
    context,
    elapsed: 0,
    runningSince: typeof document !== 'undefined' && !document.hidden ? performance.now() : 0,
  }
}

export function finishArtworkView() {
  if (!activeView) return
  pauseActiveView()
  const view = activeView
  activeView = null
  const dwellMs = Math.round(Math.min(view.elapsed, 600_000))
  if (dwellMs < 1_000) return
  queueRecommendationEvent({
    artwork_id: view.artworkId,
    event_type: 'dwell',
    dwell_ms: dwellMs,
    ...view.context,
  })
}

if (typeof window !== 'undefined') {
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      pauseActiveView()
      flushRecommendationEvents(true)
    } else {
      resumeActiveView()
    }
  })
  window.addEventListener('pagehide', () => {
    finishArtworkView()
    flushRecommendationEvents(true)
  })
}
