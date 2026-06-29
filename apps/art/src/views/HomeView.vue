<template>
  <section
    ref="artHomeRef"
    class="container-card art-home"
    :class="{
      'is-light-gear-ready': lightGearReady,
      'is-route-suspended': homeRouteSuspended,
      'is-home-visuals-visible': homeVisualsVisible,
      'is-home-backdrop-visible': homeBackdropVisible,
      'is-home-gears-visible': homeGearsVisible,
    }"
  >
    <div class="home-arrival-veil" aria-hidden="true"></div>

    <div v-if="renderHaruhiTextField && haruhiTextCorpus" class="haruhi-text-field" aria-hidden="true">
      <div class="haruhi-text-flow" :style="{ '--haruhi-text-duration': `${HARUHI_TEXT_DRIFT_SECONDS}s` }">
        <div v-for="copy in haruhiTextCopies" :key="copy" class="haruhi-text-sheet">
          <div
            v-for="row in haruhiTextRows"
            :key="`${copy}-${row.id}`"
            class="haruhi-text-row"
            :style="{
              '--row-y': `${row.y}px`,
              '--row-offset': `${row.offset}px`,
              '--row-alpha': row.alpha,
            }"
          >
            <span>{{ row.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="shouldRenderLightsOutUi" class="home-lights-out-ui">
      <div class="home-light-gear-ui">
        <section
          class="light-gear light-gear--top"
          :class="{ 'is-dragging': lightGearDragging && lightGearDragRow === 'top' }"
          aria-label="上层画作齿轮轨道"
          @pointerdown="onLightGearPointerDown('top', $event)"
          @pointermove="onLightGearDrag"
          @pointerup="onLightGearGrabEnd"
          @pointercancel="onLightGearGrabCancel"
        >
          <div v-if="renderHomeGears" class="light-gear__belt">
            <figure v-for="item in lightGearTopItems" :key="item.key" class="light-gear__tile" role="link" :aria-label="item.title" :data-artwork-id="item.id">
              <img :src="item.imageUrl" :alt="item.title" loading="eager" decoding="async" draggable="false" @dragstart.prevent />
              <figcaption>{{ item.title }}</figcaption>
            </figure>
          </div>
        </section>

        <section class="day-hero-panel">
          <p class="day-kicker">Haruhi Fanclub Gallery</p>
          <p class="day-visitor">画廊的第 <strong>{{ visitorNumberText }}</strong> 位访问者，你好</p>
        </section>

        <section
          class="light-gear light-gear--bottom"
          :class="{ 'is-dragging': lightGearDragging && lightGearDragRow === 'bottom' }"
          aria-label="下层画作齿轮轨道"
          @pointerdown="onLightGearPointerDown('bottom', $event)"
          @pointermove="onLightGearDrag"
          @pointerup="onLightGearGrabEnd"
          @pointercancel="onLightGearGrabCancel"
        >
          <div v-if="renderHomeGears" class="light-gear__belt">
            <figure v-for="item in lightGearBottomItems" :key="item.key" class="light-gear__tile" role="link" :aria-label="item.title" :data-artwork-id="item.id">
              <img :src="item.imageUrl" :alt="item.title" loading="eager" decoding="async" draggable="false" @dragstart.prevent />
              <figcaption>{{ item.title }}</figcaption>
            </figure>
          </div>
        </section>
      </div>

    </div>

    <div v-if="shouldRenderLightsOnUi" class="home-lights-on-ui home-light-gear-ui">
      <section
        class="light-gear light-gear--top"
        :class="{ 'is-dragging': lightGearDragging && lightGearDragRow === 'top' }"
        aria-label="上层画作齿轮轨道"
        @pointerdown="onLightGearPointerDown('top', $event)"
        @pointermove="onLightGearDrag"
        @pointerup="onLightGearGrabEnd"
        @pointercancel="onLightGearGrabCancel"
      >
        <div v-if="renderHomeGears" class="light-gear__belt">
          <figure v-for="item in lightGearTopItems" :key="item.key" class="light-gear__tile" role="link" :aria-label="item.title" :data-artwork-id="item.id">
            <img :src="item.imageUrl" :alt="item.title" loading="eager" decoding="async" draggable="false" @dragstart.prevent />
            <figcaption>{{ item.title }}</figcaption>
          </figure>
        </div>
      </section>

      <section class="day-hero-panel">
        <p class="day-kicker">Haruhi Fanclub Gallery</p>
        <p class="day-visitor">画廊的第 <strong>{{ visitorNumberText }}</strong> 位访问者，你好</p>
      </section>

      <section
        class="light-gear light-gear--bottom"
        :class="{ 'is-dragging': lightGearDragging && lightGearDragRow === 'bottom' }"
        aria-label="下层画作齿轮轨道"
        @pointerdown="onLightGearPointerDown('bottom', $event)"
        @pointermove="onLightGearDrag"
        @pointerup="onLightGearGrabEnd"
        @pointercancel="onLightGearGrabCancel"
      >
        <div v-if="renderHomeGears" class="light-gear__belt">
          <figure v-for="item in lightGearBottomItems" :key="item.key" class="light-gear__tile" role="link" :aria-label="item.title" :data-artwork-id="item.id">
            <img :src="item.imageUrl" :alt="item.title" loading="eager" decoding="async" draggable="false" @dragstart.prevent />
            <figcaption>{{ item.title }}</figcaption>
          </figure>
        </div>
      </section>
    </div>
  </section>
</template>

<script>
let cachedHaruhiBackgroundText = ''
let pendingHaruhiBackgroundText = null
</script>

<script setup>
import { computed, nextTick, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { seedArtworks } from '../mock/seedData'
import { api, thumbUrl } from '../services/api'

defineOptions({ name: 'HomeView' })
const LIGHT_GEAR_COPY_COUNT = 4
const LIGHT_GEAR_TILE_STEP = 344
const LIGHT_GEAR_MAX_VELOCITY = 3.2
const LIGHT_GEAR_THROW_BOOST = 1.35
const LIGHT_GEAR_INERTIA_DECAY = 0.982
const LIGHT_GEAR_STOP_VELOCITY = 0.0035
const LIGHT_GEAR_AUTO_SPEED = 15
const HARUHI_TEXT_DRIFT_SECONDS = 900
const HARUHI_TEXT_ROW_COUNT = 27
const HARUHI_TEXT_ROW_GAP = 58
const HARUHI_TEXT_ROW_STAGGER = 104
const HARUHI_TEXT_LINE_LENGTH = 320
const HARUHI_TEXT_SOURCE_URLS = [
  `${import.meta.env.BASE_URL}haruhi-background.txt`,
  `${import.meta.env.BASE_URL}haruhi-background.local`,
]
const HARUHI_TEXT_CACHE_KEY = 'haruhi-art-background-text'
const HARUHI_TEXT_FALLBACK = '凉宫春日 '.repeat(64)
const haruhiTextCopies = [0, 1]
const haruhiTextSource = ref(getInitialHaruhiTextSource())
const haruhiTextCorpus = computed(() => normalizeHaruhiText(haruhiTextSource.value))
const haruhiTextRows = computed(() => {
  const corpus = haruhiTextCorpus.value
  return Array.from({ length: HARUHI_TEXT_ROW_COUNT }, (_, index) => ({
    id: index,
    y: (index - Math.floor(HARUHI_TEXT_ROW_COUNT / 2)) * HARUHI_TEXT_ROW_GAP,
    offset: -((index % 8) * HARUHI_TEXT_ROW_STAGGER),
    alpha: 0.18 + (index % 4) * 0.035,
    text: getHaruhiTextLine(corpus, index),
  }))
})

function normalizeHaruhiText(value) {
  return String(value || '')
    .replace(/［＃[^］]*］/g, ' ')
    .replace(/｜/g, '')
    .replace(/《[^》]*》/g, '')
    .replace(/\s+/g, ' ')
    .replace(/　+/g, ' ')
    .trim()
}

function getHaruhiTextLine(corpus, index) {
  const source = corpus || ''
  if (!source) return ''
  if (source.length <= HARUHI_TEXT_LINE_LENGTH) {
    return `${source} `.repeat(Math.ceil(HARUHI_TEXT_LINE_LENGTH / Math.max(source.length, 1)) + 1)
  }

  const start = (index * 337) % source.length
  const doubled = `${source} ${source}`
  return doubled.slice(start, start + HARUHI_TEXT_LINE_LENGTH)
}

function readCachedHaruhiText() {
  const memoryText = normalizeHaruhiText(cachedHaruhiBackgroundText)
  if (memoryText) return cachedHaruhiBackgroundText

  if (typeof window === 'undefined') return ''
  const storedText = window.sessionStorage.getItem(HARUHI_TEXT_CACHE_KEY) || ''
  if (normalizeHaruhiText(storedText)) {
    cachedHaruhiBackgroundText = storedText
    return storedText
  }
  return ''
}

function getInitialHaruhiTextSource() {
  return readCachedHaruhiText()
}

function rememberHaruhiText(text) {
  if (!normalizeHaruhiText(text)) return ''
  cachedHaruhiBackgroundText = text
  if (typeof window !== 'undefined') {
    window.sessionStorage.setItem(HARUHI_TEXT_CACHE_KEY, text)
  }
  return text
}

async function loadHaruhiBackgroundText() {
  if (typeof window === 'undefined') return

  const cachedText = readCachedHaruhiText()
  if (cachedText) {
    haruhiTextSource.value = cachedText
    return
  }

  try {
    if (!pendingHaruhiBackgroundText) {
      pendingHaruhiBackgroundText = (async () => {
        for (const sourceUrl of HARUHI_TEXT_SOURCE_URLS) {
          const response = await window.fetch(sourceUrl, { cache: 'force-cache' })
          if (!response.ok) continue
          if (response.headers.get('content-type')?.includes('text/html')) continue

          const text = await response.text()
          if (normalizeHaruhiText(text)) return rememberHaruhiText(text)
        }
        return ''
      })().finally(() => {
        pendingHaruhiBackgroundText = null
      })
    }

    const text = await pendingHaruhiBackgroundText
    if (homeViewMounted && normalizeHaruhiText(text)) {
      haruhiTextSource.value = text
      return
    }

    if (homeViewMounted && !normalizeHaruhiText(haruhiTextSource.value)) {
      haruhiTextSource.value = HARUHI_TEXT_FALLBACK
    }
  } catch {
    if (homeViewMounted && !normalizeHaruhiText(haruhiTextSource.value)) {
      haruhiTextSource.value = HARUHI_TEXT_FALLBACK
    }
  }
}


const visitorNumber = ref(null)

// 访客数字 odometer：加载时数字快速乱跳（翻牌感），数据到达后缓动减速、定格到真实数字
const visitorDisplay = ref(null)
const visitorRollPhase = ref('idle') // idle | scramble | settle | done
let visitorRollFrame = 0
let visitorScrambleStart = 0
let visitorScrambleLast = 0
let visitorSettleStart = 0
let visitorSettleFrom = 0

const visitorNumberText = computed(() => {
  if (visitorRollPhase.value === 'done' && visitorNumber.value) {
    return visitorNumber.value.toLocaleString('zh-CN')
  }
  if (visitorDisplay.value != null) {
    return visitorDisplay.value.toLocaleString('zh-CN')
  }
  return visitorNumber.value ? visitorNumber.value.toLocaleString('zh-CN') : '····'
})

function stopVisitorRoll() {
  if (visitorRollFrame) {
    cancelAnimationFrame(visitorRollFrame)
    visitorRollFrame = 0
  }
}

function tickVisitorRoll(time) {
  if (visitorRollPhase.value === 'scramble') {
    if (!visitorScrambleStart) visitorScrambleStart = time
    // 每 ~60ms 翻一个 4 位随机数，营造老虎机式乱跳
    if (!visitorScrambleLast || time - visitorScrambleLast > 60) {
      visitorDisplay.value = 1000 + Math.floor(Math.random() * 9000)
      visitorScrambleLast = time
    }
    // 数据已到且乱跳已持续足够久 → 转入减速定格（保证“乱跳”动画可见，即便接口秒回）
    if (visitorNumber.value && time - visitorScrambleStart >= 700) {
      visitorSettleFrom = visitorDisplay.value || 0
      visitorSettleStart = time
      visitorRollPhase.value = 'settle'
    }
  } else if (visitorRollPhase.value === 'settle') {
    const t = Math.min(1, (time - visitorSettleStart) / 1100)
    // easeOutQuart：先快后极慢，像转盘减速停下
    const eased = 1 - Math.pow(1 - t, 4)
    visitorDisplay.value = Math.round(visitorSettleFrom + (visitorNumber.value - visitorSettleFrom) * eased)
    if (t >= 1) {
      visitorDisplay.value = visitorNumber.value
      visitorRollPhase.value = 'done'
      visitorRollFrame = 0
      return
    }
  } else {
    visitorRollFrame = 0
    return
  }
  visitorRollFrame = requestAnimationFrame(tickVisitorRoll)
}

function startVisitorRoll() {
  if (visitorRollPhase.value === 'done' || visitorRollFrame) return
  if (prefersReducedMotion()) {
    if (visitorNumber.value) {
      visitorDisplay.value = visitorNumber.value
      visitorRollPhase.value = 'done'
    }
    return
  }
  visitorScrambleStart = 0
  visitorScrambleLast = 0
  visitorSettleStart = 0
  visitorRollPhase.value = 'scramble'
  visitorRollFrame = requestAnimationFrame(tickVisitorRoll)
}

// 数据到达：若动画循环未在跑（reduced-motion 或已停）则兜底；否则交由 tick 自行 scramble→settle
watch(visitorNumber, (val) => {
  if (!val) return
  if (prefersReducedMotion()) {
    visitorDisplay.value = val
    visitorRollPhase.value = 'done'
    return
  }
  if (!visitorRollFrame && visitorRollPhase.value !== 'done') startVisitorRoll()
})

let visitorNumberPromise = null

function loadVisitorNumber() {
  if (visitorNumberPromise) return visitorNumberPromise
  visitorNumberPromise = api.recordVisitor()
    .then((result) => {
      const total = Number(result?.total || 0)
      visitorNumber.value = Number.isFinite(total) && total > 0 ? total : null
    })
    .catch((error) => {
      visitorNumberPromise = null
      console.warn('访客统计同步失败:', error)
    })
  return visitorNumberPromise
}

const approvedArtworks = seedArtworks.filter((item) => item.status === 'approved')
const artHomeRef = ref(null)
const lightGearDragging = ref(false)
const lightGearDragRow = ref('top')
const lightGearReady = ref(false)
const homeRouteSuspended = ref(false)
const activeHomeLightsOut = ref(false)
const renderInactiveHomeUi = ref(false)
const renderHomeGears = ref(false)
const renderHaruhiTextField = ref(false)
const homeVisualsVisible = ref(false)
const homeBackdropVisible = ref(false)
const homeGearsVisible = ref(false)
let lightGearOffset = 0
let lightGearPointerId = null
let lightGearLastX = null
let lightGearLastTime = 0
let lightGearVelocity = 0
let lightGearMoved = false
let lightGearDownTile = null
let lightGearTotalMove = 0
let lightGearInertiaFrame = 0
let lightGearInertiaLastTime = 0
let lightGearAutoFrame = 0
let lightGearAutoLastTime = 0
let lightGearRenderFrame = 0
let lightGearTopBelts = []
let lightGearBottomBelts = []
let lightGearMeasuredCycleWidth = 0
let lightGearBaseOffset = 0
let lightGearResizeObserver = null
let lightGearLayoutFrame = 0
let lightGearAutoPausedForSwitch = false
let lightGearSwitchResumeTimer = 0
let inactiveHomeUiIdle = 0
let inactiveHomeUiTimer = 0
let homeBackdropTimer = 0
let homeGearsTimer = 0
let homeViewMounted = false
let homeViewHasMounted = false
let homeViewListenersActive = false
const shouldRenderLightsOutUi = computed(() => activeHomeLightsOut.value || renderInactiveHomeUi.value)
const shouldRenderLightsOnUi = computed(() => !activeHomeLightsOut.value || renderInactiveHomeUi.value)


function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value))
}

function readHomeLightsOut() {
  if (typeof document !== 'undefined' && document.documentElement.classList.contains('art-home-lights-out')) return true
  return typeof window !== 'undefined' && window.localStorage.getItem('haruhi-art-lights-out') === '1'
}

function afterHomeFirstPaint(callback) {
  if (typeof window === 'undefined') return
  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(callback)
  })
}

function clearInactiveHomeUiRender() {
  if (inactiveHomeUiIdle && typeof window !== 'undefined') {
    window.cancelIdleCallback?.(inactiveHomeUiIdle)
  }
  if (inactiveHomeUiTimer && typeof window !== 'undefined') {
    window.clearTimeout(inactiveHomeUiTimer)
  }
  inactiveHomeUiIdle = 0
  inactiveHomeUiTimer = 0
}

function clearHomeEntranceTimers() {
  if (homeBackdropTimer && typeof window !== 'undefined') {
    window.clearTimeout(homeBackdropTimer)
  }
  if (homeGearsTimer && typeof window !== 'undefined') {
    window.clearTimeout(homeGearsTimer)
  }
  homeBackdropTimer = 0
  homeGearsTimer = 0
}

function scheduleHomeEntranceSequence() {
  if (typeof window === 'undefined') return
  clearHomeEntranceTimers()

  if (prefersReducedMotion()) {
    homeVisualsVisible.value = true
    homeBackdropVisible.value = true
    homeGearsVisible.value = true
    return
  }

  homeVisualsVisible.value = true
  homeBackdropTimer = window.setTimeout(() => {
    homeBackdropTimer = 0
    if (!homeViewMounted || homeRouteSuspended.value) return
    homeBackdropVisible.value = true
  }, 0)
  homeGearsTimer = window.setTimeout(() => {
    homeGearsTimer = 0
    if (!homeViewMounted || homeRouteSuspended.value) return
    homeGearsVisible.value = true
  }, 0)
}

function refreshLightGearAfterDomUpdate({ includeAll = false, restartObserver = false } = {}) {
  nextTick(() => {
    if (!homeViewMounted || homeRouteSuspended.value) return
    if (restartObserver) {
      stopLightGearResizeObserver()
      startLightGearResizeObserver()
    }
    refreshLightGearLayout({ includeAll })
  })
}

function revealInactiveHomeUi() {
  clearInactiveHomeUiRender()
  if (!homeViewMounted || homeRouteSuspended.value || renderInactiveHomeUi.value) return
  renderInactiveHomeUi.value = true
  refreshLightGearAfterDomUpdate({ includeAll: true, restartObserver: true })
}

function scheduleInactiveHomeUiRender() {
  if (typeof window === 'undefined' || renderInactiveHomeUi.value || inactiveHomeUiIdle || inactiveHomeUiTimer) return

  if ('requestIdleCallback' in window) {
    inactiveHomeUiIdle = window.requestIdleCallback(revealInactiveHomeUi, { timeout: 1800 })
  } else {
    inactiveHomeUiTimer = window.setTimeout(revealInactiveHomeUi, 900)
  }
}

function addHomeViewListeners() {
  if (typeof window === 'undefined' || homeViewListenersActive) return
  window.addEventListener('art-home-lights-switch', onHomeLightsSwitch)
  homeViewListenersActive = true
}

function removeHomeViewListeners() {
  if (typeof window === 'undefined' || !homeViewListenersActive) return
  window.removeEventListener('art-home-lights-switch', onHomeLightsSwitch)
  homeViewListenersActive = false
}

function getLightGearCycleWidth() {
  return lightGearMeasuredCycleWidth || lightGearCycleWidth.value
}

function normalizeLightGearOffset(value) {
  const cycle = getLightGearCycleWidth()
  if (!Number.isFinite(cycle) || cycle <= 1) return value

  let next = value % cycle
  if (next > cycle / 2) next -= cycle
  if (next < -cycle / 2) next += cycle
  return next
}

function applyLightGearDelta(delta, { immediate = false } = {}) {
  lightGearOffset = normalizeLightGearOffset(lightGearOffset + delta)
  if (immediate) {
    syncLightGearOffset()
  } else {
    scheduleLightGearOffsetSync()
  }
}

function cacheLightGearBelts({ measure = false } = {}) {
  const root = artHomeRef.value
  lightGearTopBelts = Array.from(root?.querySelectorAll('.light-gear--top .light-gear__belt') || [])
  lightGearBottomBelts = Array.from(root?.querySelectorAll('.light-gear--bottom .light-gear__belt') || [])
  const measuredWidth = measure
    ? Math.max(
        0,
        ...lightGearTopBelts.map((belt) => belt.getBoundingClientRect().width),
        ...lightGearBottomBelts.map((belt) => belt.getBoundingClientRect().width)
      )
    : 0

  if (measuredWidth > 0) {
    lightGearMeasuredCycleWidth = measuredWidth / LIGHT_GEAR_COPY_COUNT
    lightGearBaseOffset = -measuredWidth / LIGHT_GEAR_COPY_COUNT
    lightGearOffset = normalizeLightGearOffset(lightGearOffset)
  } else {
    lightGearMeasuredCycleWidth = 0
    lightGearBaseOffset = -getLightGearCycleWidth()
  }
}

function getLightGearBeltsForRender(belts, includeAll = false) {
  if (includeAll) return belts
  const lightsOutActive = activeHomeLightsOut.value
  return belts.filter((belt) => Boolean(belt.closest('.home-lights-out-ui')) === lightsOutActive)
}

function syncLightGearOffset({ includeAll = false } = {}) {
  lightGearRenderFrame = 0
  if (!lightGearTopBelts.length || !lightGearBottomBelts.length || !lightGearBaseOffset) {
    cacheLightGearBelts()
  }

  const topOffset = (lightGearBaseOffset + lightGearOffset).toFixed(3)
  const bottomOffset = (lightGearBaseOffset - lightGearOffset).toFixed(3)
  for (const belt of getLightGearBeltsForRender(lightGearTopBelts, includeAll)) {
    belt.style.transform = `translate3d(${topOffset}px, 0, 0)`
  }
  for (const belt of getLightGearBeltsForRender(lightGearBottomBelts, includeAll)) {
    belt.style.transform = `translate3d(${bottomOffset}px, 0, 0)`
  }
}

function refreshLightGearLayout({ includeAll = false, measure = false } = {}) {
  lightGearLayoutFrame = 0
  cacheLightGearBelts({ measure })
  syncLightGearOffset({ includeAll })
}

function scheduleLightGearLayoutRefresh() {
  if (typeof window === 'undefined' || lightGearLayoutFrame) return
  lightGearLayoutFrame = window.requestAnimationFrame(refreshLightGearLayout)
}

function scheduleLightGearOffsetSync() {
  if (typeof window === 'undefined' || lightGearRenderFrame) return
  lightGearRenderFrame = window.requestAnimationFrame(syncLightGearOffset)
}

function startLightGearResizeObserver() {
  if (typeof window === 'undefined' || typeof window.ResizeObserver === 'undefined') return

  lightGearResizeObserver = new window.ResizeObserver(scheduleLightGearLayoutRefresh)
  if (artHomeRef.value) {
    lightGearResizeObserver.observe(artHomeRef.value)
    for (const gear of artHomeRef.value.querySelectorAll('.light-gear')) {
      lightGearResizeObserver.observe(gear)
    }
  }
}

function stopLightGearResizeObserver() {
  lightGearResizeObserver?.disconnect()
  lightGearResizeObserver = null

  if (lightGearLayoutFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(lightGearLayoutFrame)
  }
  lightGearLayoutFrame = 0
}

function clearLightGearSwitchResume() {
  if (lightGearSwitchResumeTimer && typeof window !== 'undefined') {
    window.clearTimeout(lightGearSwitchResumeTimer)
  }
  lightGearSwitchResumeTimer = 0
}

function suspendHomeRouteMotion() {
  homeRouteSuspended.value = true
  clearInactiveHomeUiRender()
  clearHomeEntranceTimers()
  cancelLightGearInertia()
  cancelLightGearAuto()
  cancelLightGearRender()
  clearLightGearSwitchResume()
}

function onHomeLightsSwitch(event) {
  if (!homeViewMounted) return

  const phase = event?.detail?.phase
  const nextLightsOut = Boolean(event?.detail?.value)
  if (phase === 'start') {
    activeHomeLightsOut.value = nextLightsOut
    if (!renderInactiveHomeUi.value) {
      renderInactiveHomeUi.value = true
      refreshLightGearAfterDomUpdate({ includeAll: true, restartObserver: true })
    }
    clearInactiveHomeUiRender()
    clearLightGearSwitchResume()
    lightGearAutoPausedForSwitch = Boolean(lightGearAutoFrame)
    cancelLightGearAuto()
    cancelLightGearRender()
    return
  }

  if (phase === 'end') {
    activeHomeLightsOut.value = nextLightsOut
    refreshLightGearAfterDomUpdate({ includeAll: true })
    if (lightGearAutoPausedForSwitch && !prefersReducedMotion()) {
      lightGearSwitchResumeTimer = window.setTimeout(() => {
        lightGearSwitchResumeTimer = 0
        lightGearAutoPausedForSwitch = false
        startLightGearAuto()
      }, 80)
    } else {
      lightGearAutoPausedForSwitch = false
    }
  }
}

function cancelLightGearRender() {
  if (lightGearRenderFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(lightGearRenderFrame)
  }
  lightGearRenderFrame = 0
}

function cancelLightGearInertia() {
  if (lightGearInertiaFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(lightGearInertiaFrame)
  }
  lightGearInertiaFrame = 0
  lightGearInertiaLastTime = 0
}

function prefersReducedMotion() {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

function cancelLightGearAuto() {
  if (lightGearAutoFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(lightGearAutoFrame)
  }
  lightGearAutoFrame = 0
  lightGearAutoLastTime = 0
}

function getLightGearAutoSpeed() {
  return LIGHT_GEAR_AUTO_SPEED / 1000
}

function tickLightGearAuto(time) {
  if (!lightGearAutoFrame) return

  if (!lightGearAutoLastTime) lightGearAutoLastTime = time
  const delta = Math.min(40, time - lightGearAutoLastTime)
  lightGearAutoLastTime = time

  if (!lightGearDragging.value && !lightGearInertiaFrame) {
    applyLightGearDelta(-getLightGearAutoSpeed() * delta, { immediate: true })
  }

  lightGearAutoFrame = window.requestAnimationFrame(tickLightGearAuto)
}

function startLightGearAuto() {
  if (
    typeof window === 'undefined' ||
    !homeViewMounted ||
    homeRouteSuspended.value ||
    lightGearAutoFrame ||
    prefersReducedMotion()
  ) return
  lightGearAutoLastTime = 0
  lightGearAutoFrame = window.requestAnimationFrame(tickLightGearAuto)
}

function releaseLightGearCapture(event) {
  const target = event.currentTarget
  if (target?.hasPointerCapture?.(event.pointerId)) {
    target.releasePointerCapture(event.pointerId)
  }
}

function onLightGearPointerDown(row, event) {
  if (event.button !== undefined && event.button !== 0) return

  event.preventDefault()
  cancelLightGearInertia()
  cancelLightGearAuto()
  lightGearDragging.value = true
  lightGearDragRow.value = row
  lightGearPointerId = event.pointerId
  lightGearLastX = event.clientX
  lightGearLastTime = event.timeStamp || performance.now()
  lightGearVelocity = 0
  lightGearMoved = false
  lightGearTotalMove = 0
  // 记录按下的画作格，用于 pointerup 时判定“点击”（pointerdown 已 preventDefault，原生 click 不会触发）
  lightGearDownTile = event.target?.closest?.('.light-gear__tile') || null
  event.currentTarget?.setPointerCapture?.(event.pointerId)
}

function onLightGearDrag(event) {
  if (!lightGearDragging.value || event.pointerId !== lightGearPointerId) return

  event.preventDefault()
  const nowTime = event.timeStamp || performance.now()
  if (lightGearLastX === null) {
    lightGearLastX = event.clientX
    lightGearLastTime = nowTime
    return
  }

  const diff = event.clientX - lightGearLastX
  const elapsed = Math.max(8, nowTime - lightGearLastTime)
  lightGearTotalMove += Math.abs(diff)
  if (Math.abs(diff) >= 0.2) {
    const signedDelta = lightGearDragRow.value === 'top' ? diff : -diff
    const instantVelocity = signedDelta / elapsed
    applyLightGearDelta(signedDelta)
    lightGearMoved = true
    lightGearVelocity = clamp(
      lightGearVelocity * 0.24 + instantVelocity * 0.76,
      -LIGHT_GEAR_MAX_VELOCITY,
      LIGHT_GEAR_MAX_VELOCITY
    )
  }

  lightGearLastX = event.clientX
  lightGearLastTime = nowTime
}

function stopLightGearInertia({ resumeAuto = false } = {}) {
  cancelLightGearInertia()
  lightGearVelocity = 0
  if (resumeAuto) startLightGearAuto()
}

function tickLightGearInertia(time) {
  if (!lightGearInertiaFrame) return

  if (!lightGearInertiaLastTime) lightGearInertiaLastTime = time
  const delta = Math.min(40, time - lightGearInertiaLastTime)
  lightGearInertiaLastTime = time

  applyLightGearDelta(lightGearVelocity * delta, { immediate: true })
  lightGearVelocity *= Math.pow(LIGHT_GEAR_INERTIA_DECAY, delta / 16.67)

  if (Math.abs(lightGearVelocity) <= LIGHT_GEAR_STOP_VELOCITY) {
    stopLightGearInertia({ resumeAuto: true })
    return
  }

  lightGearInertiaFrame = window.requestAnimationFrame(tickLightGearInertia)
}

function startLightGearInertia() {
  if (typeof window === 'undefined' || !lightGearMoved || Math.abs(lightGearVelocity) <= LIGHT_GEAR_STOP_VELOCITY) {
    stopLightGearInertia({ resumeAuto: true })
    return
  }

  lightGearVelocity = clamp(
    lightGearVelocity * LIGHT_GEAR_THROW_BOOST,
    -LIGHT_GEAR_MAX_VELOCITY,
    LIGHT_GEAR_MAX_VELOCITY
  )
  lightGearInertiaLastTime = 0
  lightGearInertiaFrame = window.requestAnimationFrame(tickLightGearInertia)
}

function onLightGearGrabEnd(event) {
  if (!lightGearDragging.value || event.pointerId !== lightGearPointerId) return

  if (lightGearDownTile && lightGearTotalMove < 8) {
    // 累计位移很小 → 视为点击，打开该画作详情（容忍真实点击的轻微手抖）
    openGearArtwork(lightGearDownTile.dataset.artworkId)
  } else if (lightGearMoved) {
    event.preventDefault()
  }
  lightGearDownTile = null
  releaseLightGearCapture(event)
  lightGearDragging.value = false
  lightGearPointerId = null
  lightGearLastX = null
  lightGearLastTime = 0
  startLightGearInertia()
}

function onLightGearGrabCancel(event) {
  if (event.pointerId !== lightGearPointerId) return

  releaseLightGearCapture(event)
  lightGearDragging.value = false
  lightGearPointerId = null
  lightGearLastX = null
  lightGearLastTime = 0
  stopLightGearInertia({ resumeAuto: true })
}


function activateHomeView() {
  if (homeViewMounted) return
  homeViewMounted = true
  homeRouteSuspended.value = false
  activeHomeLightsOut.value = readHomeLightsOut()
  renderInactiveHomeUi.value = false
  renderHomeGears.value = false
  renderHaruhiTextField.value = false
  homeVisualsVisible.value = false
  homeBackdropVisible.value = false
  homeGearsVisible.value = false
  lightGearReady.value = false
  addHomeViewListeners()
  loadHaruhiBackgroundText()
  afterHomeFirstPaint(() => {
    if (!homeViewMounted || homeRouteSuspended.value) return
    renderHaruhiTextField.value = true
    renderHomeGears.value = true
    nextTick(() => {
      if (!homeViewMounted || homeRouteSuspended.value) return
      refreshLightGearLayout()
      startLightGearResizeObserver()
      lightGearReady.value = true
      startLightGearAuto()
      scheduleInactiveHomeUiRender()
      window.requestAnimationFrame(() => {
        if (!homeViewMounted || homeRouteSuspended.value) return
        scheduleHomeEntranceSequence()
      })
    })
  })
}

function deactivateHomeView() {
  if (!homeViewMounted && !homeViewListenersActive) return
  homeViewMounted = false
  suspendHomeRouteMotion()
  renderInactiveHomeUi.value = false
  renderHomeGears.value = false
  renderHaruhiTextField.value = false
  homeVisualsVisible.value = false
  homeBackdropVisible.value = false
  homeGearsVisible.value = false
  lightGearReady.value = false
  stopVisitorRoll()
  stopLightGearResizeObserver()
  removeHomeViewListeners()
}

onMounted(() => {
  homeViewHasMounted = true
  startVisitorRoll()
  loadVisitorNumber()
  loadGalleryPool()
  activateHomeView()
})

onActivated(() => {
  if (!homeViewHasMounted) return
  if (!visitorNumber.value) loadVisitorNumber()
  startVisitorRoll()
  activateHomeView()
})

onBeforeRouteLeave(() => {
  suspendHomeRouteMotion()
})

onDeactivated(deactivateHomeView)

onBeforeUnmount(deactivateHomeView)


const router = useRouter()

// 首页画作来自全局绘画池：初始用本地 seed 占位（保证首屏齿轮不空），
// 挂载后异步拉取真实的已通过作品替换；无后端（本地开发）时保持 seed。
const seedGearPool = approvedArtworks
const galleryGearPool = ref(seedGearPool)
const lightGearSource = computed(() =>
  galleryGearPool.value.length ? galleryGearPool.value : seedGearPool
)
const lightGearCycleWidth = computed(
  () => Math.max(1, lightGearSource.value.length) * LIGHT_GEAR_TILE_STEP
)

function makeLightGearItems(reverse, source) {
  if (!source.length) return []
  return Array.from({ length: source.length * LIGHT_GEAR_COPY_COUNT }, (_, index) => {
    const sourceIndex = index % source.length
    const artwork = reverse
      ? source[(source.length - 1 - sourceIndex + source.length) % source.length]
      : source[sourceIndex]

    return {
      key: `${reverse ? 'bottom' : 'top'}-${index}-${artwork.id}`,
      id: artwork.id,
      title: artwork.title,
      // 缩略图静态直出（nginx 命中即返回），避免首页一次性加载多张原图
      imageUrl: thumbUrl(artwork.image_url, 640),
    }
  })
}

const lightGearTopItems = computed(() => makeLightGearItems(false, lightGearSource.value))
const lightGearBottomItems = computed(() => makeLightGearItems(true, lightGearSource.value))

// 异步拉取全局绘画池（随机一批已通过作品）替换 seed 占位
async function loadGalleryPool() {
  try {
    const res = await api.artworksList({
      status: 'approved',
      sort: 'random',
      seed: Math.floor(Math.random() * 2147483647),
      page: 1,
      pageSize: 24,
    })
    const items = (res?.data || []).filter((item) => item && item.image_url)
    if (items.length) {
      galleryGearPool.value = items
      await nextTick()
      // 作品数变化→齿轮带宽度变化，重测循环宽度以保持无缝轮播
      refreshLightGearLayout({ includeAll: true, measure: true })
    }
  } catch {
    // 无后端 / 拉取失败：保持 seed 占位，不打断首页
  }
}

// 点击画作打开其详情（复用画廊页的 ?artwork= 详情弹层）；拖拽刚结束则抑制误触
function openGearArtwork(id) {
  if (!id) return
  router.push({ name: 'gallery', query: { artwork: id } })
}

</script>

<style scoped>
.art-home {
  width: min(1500px, calc(100% - 32px));
  padding-top: 8px;
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: stretch;
  isolation: isolate;
}

:global(html.art-home-route:not(.art-home-lights-out)) .art-home {
  width: 100%;
  margin-bottom: 0;
  padding: 0;
}

.art-home::before,
.art-home::after {
  content: "";
  position: absolute;
  display: none;
  pointer-events: none;
}

.art-home .home-arrival-veil {
  position: fixed;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  opacity: 1;
  background:
    radial-gradient(ellipse at 50% 48%, rgba(255, 255, 255, 0.72), rgba(255, 255, 255, 0.34) 24%, transparent 56%),
    radial-gradient(ellipse at 50% 50%, transparent 0 44%, rgba(255, 246, 238, 0.5) 74%, rgba(255, 246, 238, 0.78) 100%);
  transform: translateZ(0);
  transition:
    opacity 4.8s cubic-bezier(0.16, 1, 0.3, 1),
    transform 4.8s cubic-bezier(0.16, 1, 0.3, 1);
}

.art-home.is-home-visuals-visible .home-arrival-veil {
  opacity: 0;
  transform: translate3d(0, -10px, 0) scale(1.018);
}

:global(html.art-home-route.art-home-lights-out .art-home .home-arrival-veil) {
  background:
    radial-gradient(ellipse at 50% 48%, rgba(141, 240, 255, 0.16), rgba(72, 88, 180, 0.14) 24%, transparent 58%),
    radial-gradient(ellipse at 50% 50%, transparent 0 42%, rgba(2, 8, 22, 0.56) 72%, rgba(1, 4, 13, 0.86) 100%),
    linear-gradient(135deg, rgba(1, 4, 13, 0.72), rgba(9, 5, 23, 0.7));
}

.art-home .haruhi-text-field {
  --haruhi-text-duration: 900s;
  position: fixed;
  inset: -46vmax;
  z-index: 1;
  overflow: hidden;
  pointer-events: none;
  opacity: 0;
  mix-blend-mode: multiply;
  transform: rotate(-35deg) translate3d(0, -18px, 0) scale(1.012);
  transform-origin: center;
  mask-image: linear-gradient(90deg, transparent 0%, black 13%, black 86%, transparent 100%);
  transition:
    opacity 4s ease,
    transform 4s cubic-bezier(0.16, 1, 0.3, 1);
}

.art-home.is-home-backdrop-visible .haruhi-text-field {
  opacity: 0.28;
  transform: rotate(-35deg) translate3d(0, 0, 0) scale(1);
}

.art-home .haruhi-text-field::before,
.art-home .haruhi-text-field::after {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.art-home .haruhi-text-field::before {
  background:
    repeating-linear-gradient(
      0deg,
      transparent 0 56px,
      rgba(217, 70, 103, 0.13) 56px 57px,
      transparent 57px 114px
    ),
    repeating-linear-gradient(
      90deg,
      transparent 0 132px,
      rgba(27, 139, 155, 0.1) 132px 133px,
      transparent 133px 264px
    );
  opacity: 0.5;
}

.art-home .haruhi-text-field::after {
  background:
    radial-gradient(circle at 24% 35%, rgba(217, 70, 103, 0.14), transparent 22%),
    radial-gradient(circle at 72% 62%, rgba(27, 139, 155, 0.12), transparent 24%);
  opacity: 0.55;
}

.art-home .haruhi-text-flow {
  position: absolute;
  inset: 0;
  display: flex;
  width: max-content;
  transform: translate3d(0, 0, 0);
  animation: haruhiTextDrift var(--haruhi-text-duration) linear infinite;
  will-change: transform;
}

.art-home.is-route-suspended .haruhi-text-flow {
  animation-play-state: paused;
}

.art-home .haruhi-text-sheet {
  position: relative;
  flex: 0 0 max(220vw, 3200px);
  height: 100%;
  overflow: hidden;
}

.art-home .haruhi-text-row {
  position: absolute;
  top: calc(50% + var(--row-y));
  left: -12%;
  z-index: 1;
  display: flex;
  align-items: center;
  width: max-content;
  color: rgba(217, 70, 103, var(--row-alpha));
  font-size: clamp(22px, 2.6vw, 42px);
  font-weight: 950;
  line-height: 1;
  letter-spacing: 0;
  white-space: nowrap;
  text-shadow:
    0 0 18px rgba(217, 70, 103, 0.18),
    0 0 34px rgba(27, 139, 155, 0.08);
  transform: translate3d(var(--row-offset), 0, 0);
}

.art-home .haruhi-text-row::before {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(217, 70, 103, 0.16), rgba(27, 139, 155, 0.12), transparent);
  transform: translateY(-50%);
  pointer-events: none;
}

.art-home .haruhi-text-row span {
  position: relative;
  z-index: 1;
  display: inline-block;
  padding-right: 1.4em;
}

.art-home .home-lights-out-ui,
.art-home .home-lights-on-ui {
  position: relative;
  grid-area: 1 / 1;
  min-width: 0;
  z-index: 2;
  backface-visibility: hidden;
  transform: translateZ(0);
}

:global(html.art-home-route.art-home-lights-out) .art-home::before,
:global(html.art-home-route.art-home-lights-out) .art-home::after {
  display: block;
}

:global(html.art-home-route.art-home-lights-out .art-home .haruhi-text-field) {
  mix-blend-mode: screen;
}

:global(html.art-home-route.art-home-lights-out .art-home.is-home-backdrop-visible .haruhi-text-field) {
  opacity: 0.38;
}

:global(html.art-home-route.art-home-lights-out .art-home .haruhi-text-row) {
  color: rgba(248, 252, 255, var(--row-alpha));
  text-shadow:
    0 0 14px rgba(141, 240, 255, 0.2),
    0 0 34px rgba(255, 83, 124, 0.12);
}


.art-home .home-light-gear-ui {
  --light-home-pad: clamp(22px, 3.2dvh, 54px);
  --light-gear-bleed: clamp(56px, 5vw, 120px);
  position: relative;
  height: calc(100dvh - 120px);
  min-height: 0;
  overflow: visible;
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  justify-items: center;
  gap: clamp(12px, 2dvh, 24px);
  padding: var(--light-home-pad) 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
  color: #12333c;
}

.art-home .light-gear,
.art-home .day-hero-panel {
  position: relative;
  z-index: 2;
}

.art-home .light-gear {
  width: calc(100vw + var(--light-gear-bleed) + var(--light-gear-bleed));
  height: 100%;
  min-height: 0;
  margin-inline: calc(50% - 50vw - var(--light-gear-bleed));
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
  cursor: grab;
  touch-action: none;
  user-select: none;
  opacity: 0;
  transform: translate3d(0, 0, 0);
  backface-visibility: hidden;
  contain: layout paint;
  isolation: isolate;
  transition: opacity 4.4s cubic-bezier(0.42, 0, 0.2, 1);
}

.art-home.is-home-gears-visible .light-gear {
  opacity: 1;
  transform: translate3d(0, 0, 0);
}

.art-home .light-gear.is-dragging {
  cursor: grabbing;
}

.art-home .light-gear::before,
.art-home .light-gear::after {
  content: none;
  position: absolute;
  left: 0;
  right: 0;
  z-index: 3;
  height: 15px;
  pointer-events: none;
  background:
    repeating-linear-gradient(
      90deg,
      rgba(27, 139, 155, 0.32) 0 14px,
      rgba(255, 255, 255, 0.55) 14px 22px,
      transparent 22px 34px
    );
  filter: drop-shadow(0 4px 10px rgba(27, 139, 155, 0.12));
}

.art-home .light-gear::before {
  top: 0;
}

.art-home .light-gear::after {
  bottom: 0;
  transform: rotate(180deg);
}

.art-home .light-gear__belt {
  position: absolute;
  top: 50%;
  left: 0;
  display: flex;
  align-items: center;
  gap: 28px;
  width: max-content;
  height: 236px;
  will-change: transform;
  contain: layout paint;
  backface-visibility: hidden;
  transform-style: preserve-3d;
  margin-top: -118px;
  opacity: 0;
  transition: none;
}

.art-home.is-route-suspended .light-gear__belt {
  transition: none;
}

.art-home.is-light-gear-ready .light-gear__belt {
  opacity: 1;
}

.art-home .light-gear--top .light-gear__belt {
  transform: translate3d(calc(-25% + var(--light-gear-offset, 0px)), 0, 0);
}

.art-home .light-gear--bottom .light-gear__belt {
  transform: translate3d(calc(-25% - var(--light-gear-offset, 0px)), 0, 0);
}

.art-home .light-gear__tile {
  position: relative;
  flex: 0 0 316px;
  width: 316px;
  height: 236px;
  margin: 0;
  overflow: hidden;
  border: 0;
  border-radius: 22px;
  background: transparent;
  box-shadow: inset 0 0 0 0 transparent;
  transform: translateZ(0);
  backface-visibility: hidden;
  cursor: pointer;
  transition: box-shadow 0.32s ease;
}

.art-home .light-gear__tile::before,
.art-home .light-gear__tile::after {
  content: none;
  position: absolute;
  left: 12px;
  right: 12px;
  z-index: 2;
  height: 5px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  pointer-events: none;
}

.art-home .light-gear__tile::before {
  top: 8px;
}

.art-home .light-gear__tile::after {
  bottom: 8px;
}

.art-home .light-gear__tile img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
  transform: scale(1.02);
  backface-visibility: hidden;
  contain: paint;
  transition: transform 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .light-gear__tile:hover img {
  transform: scale(1.12);
}

.art-home .light-gear__tile figcaption {
  position: absolute;
  left: 14px;
  right: 14px;
  bottom: 14px;
  z-index: 3;
  overflow: hidden;
  padding: 7px 11px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  color: rgba(18, 51, 60, 0.78);
  font-size: 13px;
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
  backdrop-filter: blur(10px);
  opacity: 0;
  transform: translateY(8px);
  transition: opacity 0.3s ease, transform 0.3s ease;
  pointer-events: none;
}

/* 悬浮：内描边高亮 + 底部压暗衬托标题（都在 tile 内，不被 section 裁剪） */
.art-home .light-gear__tile:hover {
  box-shadow:
    inset 0 0 0 3px rgba(255, 255, 255, 0.92),
    inset 0 -70px 56px -34px rgba(0, 0, 0, 0.55);
}

.art-home .light-gear__tile:hover figcaption {
  opacity: 1;
  transform: translateY(0);
}

:global(html.art-home-route.art-home-lights-out) .art-home .light-gear__tile:hover {
  box-shadow:
    inset 0 0 0 2px rgba(141, 240, 255, 0.85),
    inset 0 0 30px rgba(141, 240, 255, 0.22),
    inset 0 -70px 56px -34px rgba(0, 0, 0, 0.6);
}

.art-home .light-gear--bottom {
  background: transparent;
}

.art-home .day-hero-panel {
  z-index: 12;
  width: min(620px, 100%);
  padding: 0;
  text-align: center;
  opacity: 0;
  transform: translate3d(0, 10px, 0) scale(0.996);
  transition:
    opacity 0.85s ease,
    transform 1.1s cubic-bezier(0.16, 1, 0.3, 1);
}

.art-home.is-home-visuals-visible .day-hero-panel {
  opacity: 1;
  transform: translate3d(0, 0, 0) scale(1);
}

.art-home .day-kicker {
  margin: 0 0 12px;
  color: #d94667;
  font-size: 13px;
  font-weight: 950;
  letter-spacing: 0;
  text-transform: uppercase;
}

.art-home .day-visitor {
  margin: 0;
  color: rgba(13, 43, 52, 0.82);
  font-size: clamp(22px, 3.2vw, 38px);
  line-height: 1.18;
  font-weight: 950;
  text-shadow:
    0 1px 0 rgba(255, 255, 255, 0.92),
    0 12px 30px rgba(60, 155, 170, 0.18);
}

.art-home .day-visitor strong {
  color: #1b8b9b;
  font-size: 1.12em;
  font-weight: 950;
  font-variant-numeric: tabular-nums;
  display: inline-block;
  min-width: 2.6em;
  text-align: center;
}

:global(html.art-home-route.art-home-lights-out .art-home .day-kicker) {
  color: rgba(141, 240, 255, 0.86);
  text-shadow: 0 0 18px rgba(141, 240, 255, 0.28);
}

:global(html.art-home-route.art-home-lights-out .art-home .day-visitor) {
  color: rgba(248, 252, 255, 0.98);
  text-shadow:
    0 1px 0 rgba(255, 255, 255, 0.16),
    0 0 20px rgba(141, 240, 255, 0.24),
    0 16px 38px rgba(0, 0, 0, 0.54);
}

:global(html.art-home-route.art-home-lights-out .art-home .day-visitor strong) {
  color: #ffffff;
  text-shadow:
    0 0 14px rgba(255, 255, 255, 0.46),
    0 0 28px rgba(141, 240, 255, 0.3);
}

.art-home::before {
  inset: -80px -90px -60px;
  z-index: -2;
  border-radius: 34px;
  background:
    radial-gradient(ellipse at 50% 18%, rgba(72, 158, 255, 0.12), transparent 34%),
    radial-gradient(ellipse at 22% 70%, rgba(177, 140, 255, 0.1), transparent 30%),
    radial-gradient(ellipse at 82% 78%, rgba(255, 99, 125, 0.05), transparent 28%),
    linear-gradient(145deg, #01030a 0%, #060b18 42%, #0b0716 100%);
}

.art-home::after {
  inset: -40px -44px -48px;
  z-index: -1;
  border-radius: 30px;
  background:
    radial-gradient(ellipse at center, transparent 0 55%, rgba(0, 0, 0, 0.38) 86%, rgba(0, 0, 0, 0.66) 100%),
    radial-gradient(circle at 16% 24%, rgba(0, 0, 0, 0.48), transparent 24%),
    radial-gradient(circle at 84% 66%, rgba(0, 0, 0, 0.36), transparent 22%);
  opacity: 0.86;
}


.art-home h1,
.art-home h2,
.art-home p {
  margin: 0;
}





@keyframes haruhiTextDrift {
  from {
    transform: translate3d(0, 0, 0);
  }

  to {
    transform: translate3d(-50%, 0, 0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .art-home .haruhi-text-flow {
    animation: none !important;
  }

  .art-home .home-arrival-veil,
  .art-home .haruhi-text-field,
  .art-home .day-hero-panel,
  .art-home .light-gear,
  .art-home .light-gear__belt {
    transition: none !important;
  }

  .art-home .home-arrival-veil {
    opacity: 0 !important;
  }
}


@media (max-height: 820px) and (min-width: 821px) {
  :global(html.art-home-route:not(.art-home-lights-out)) .art-home {
    margin-bottom: 0;
  }

  .art-home .home-light-gear-ui {
    --light-home-pad: clamp(18px, 3dvh, 34px);
    height: calc(100dvh - 108px);
    min-height: 0;
    grid-template-rows: 205px auto 205px;
    gap: clamp(8px, 1.6dvh, 16px);
  }

  .art-home .light-gear {
    height: 205px;
  }

  .art-home .light-gear__belt {
    gap: 22px;
    height: 176px;
    margin-top: -88px;
  }

  .art-home .light-gear__tile {
    flex-basis: 238px;
    width: 238px;
    height: 176px;
    border-radius: 18px;
  }

  .art-home .light-gear__tile figcaption {
    bottom: 11px;
    font-size: 11px;
  }

  .art-home .day-visitor {
    font-size: clamp(20px, 2.35vw, 30px);
  }
}

@media (max-height: 700px) and (min-width: 821px) {
  .art-home .home-light-gear-ui {
    --light-home-pad: 14px;
    height: calc(100dvh - 104px);
    min-height: 0;
    grid-template-rows: 162px auto 162px;
    gap: 8px;
  }

  .art-home .light-gear {
    height: 162px;
  }

  .art-home .light-gear__belt {
    gap: 18px;
    height: 132px;
    margin-top: -66px;
  }

  .art-home .light-gear__tile {
    flex-basis: 190px;
    width: 190px;
    height: 132px;
    border-radius: 16px;
  }

  .art-home .day-kicker {
    margin-bottom: 6px;
    font-size: 11px;
  }

  .art-home .day-visitor {
    font-size: clamp(18px, 2.1vw, 25px);
  }
}

@media (max-width: 820px) {
  .art-home {
    width: min(100% - 20px, 1500px);
    padding: 0;
  }

  .art-home .home-light-gear-ui {
    height: calc(100dvh - 108px);
    min-height: 0;
    --light-home-pad: 16px;
    --light-gear-bleed: clamp(24px, 8vw, 56px);
    grid-template-rows: 190px auto 190px;
    gap: 18px;
    padding: 24px 0;
    border-radius: 0;
  }

  .art-home .light-gear {
    height: 190px;
  }

  .art-home .light-gear__belt {
    top: 50%;
    gap: 20px;
    height: 156px;
    margin-top: -78px;
  }

  .art-home .light-gear__tile {
    flex-basis: 220px;
    width: 220px;
    height: 156px;
    border-radius: 18px;
  }

  .art-home .light-gear__tile figcaption {
    font-size: 11px;
  }

  .art-home .day-hero-panel {
    width: min(92%, 520px);
  }

  .art-home .haruhi-text-field {
    inset: -64vmax;
  }

  .art-home.is-home-backdrop-visible .haruhi-text-field {
    opacity: 0.24;
  }

  :global(html.art-home-route.art-home-lights-out .art-home.is-home-backdrop-visible .haruhi-text-field) {
    opacity: 0.34;
  }

  .art-home .haruhi-text-row {
    font-size: clamp(20px, 6vw, 30px);
  }

  .art-home::before {
    inset: -42px -28px -34px;
  }

}

@media (max-width: 820px) and (max-height: 700px) {
  .art-home {
    width: min(100% - 16px, 1500px);
  }

  .art-home .home-light-gear-ui {
    --light-home-pad: 10px;
    --light-gear-bleed: clamp(18px, 7vw, 42px);
    height: calc(100dvh - 96px);
    grid-template-rows: 132px auto 132px;
    gap: 6px;
    padding: 12px 0;
    border-radius: 0;
  }

  .art-home .light-gear {
    height: 132px;
  }

  .art-home .light-gear__belt {
    gap: 14px;
    height: 108px;
    margin-top: -54px;
  }

  .art-home .light-gear__tile {
    flex-basis: 154px;
    width: 154px;
    height: 108px;
    border-radius: 14px;
  }

  .art-home .light-gear__tile figcaption {
    left: 9px;
    right: 9px;
    bottom: 9px;
    padding: 5px 8px;
    font-size: 10px;
  }

  .art-home .day-kicker {
    display: none;
  }

  .art-home .day-visitor {
    font-size: clamp(16px, 4vw, 22px);
  }
}
</style>
