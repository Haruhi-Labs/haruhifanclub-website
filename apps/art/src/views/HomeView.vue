<template>
  <section class="container-card art-home">
    <div class="home-lights-out-ui">
    <div class="endless-screen" :style="stageStyle">
      <div class="space-field" aria-hidden="true">
        <span class="star-dust"></span>
        <span class="star-layer star-layer-a"></span>
        <span class="star-layer star-layer-b"></span>
        <span class="star-layer star-layer-c"></span>
        <span class="nebula nebula-a"></span>
        <span class="nebula nebula-b"></span>
        <span class="galaxy-halo halo-a"></span>
        <span class="galaxy-halo halo-b"></span>
        <span class="bright-stars"></span>
        <span class="void-shadow"></span>
        <span class="viewport-glass"></span>
      </div>

      <div class="screen-header">
        <div>
          <p class="eyebrow">Endless Eight Observatory</p>
          <h1>八月循环观测中枢</h1>
        </div>
        <div class="status-chip">
          <span></span>
          本地 seed 同步
        </div>
      </div>

      <div
        :class="[
          'visual-stage',
          { 'is-gallery-orbit-active': galleryOrbitVisible || galleryOrbitDragging || galleryOrbitSpinning },
        ]"
      >
        <div class="summer-strip" aria-label="暑假倒计时">
          <span>暑假倒计时</span>
          <strong>{{ summerCountdown }}</strong>
          <span>days</span>
        </div>

        <div
          ref="shiftHoverFieldRef"
          :class="['shift-hover-field', { 'is-hovering': shiftHovering, 'is-dragging': shiftDragging }]"
          aria-hidden="true"
          @click="showShiftWheel"
          @pointerdown="onShiftPointerDown"
          @pointermove="onShiftDrag"
          @pointerup="onShiftGrabEnd"
          @pointercancel="onShiftGrabCancel"
        ></div>

        <div
          ref="shiftStackRef"
          :class="[
            'time-shift-stack',
            {
              'is-hovering': shiftHovering,
              'is-dragging': shiftDragging,
              'is-spinning': shiftSpinning,
            },
          ]"
          aria-label="时间跃迁层"
        >
          <div class="shift-label">
            <span>TIME JUMP</span>
            <strong>{{ shiftStateText }}</strong>
          </div>
          <div class="shift-module-shell" aria-hidden="true">
            <span class="shift-module-shell__edge"></span>
          </div>
          <div class="shift-window">
            <div class="shift-track">
              <span
                v-for="layer in timeShiftLayers"
                :key="layer.id"
                class="shift-layer"
                :style="{
                  '--panel-x': `${layer.x}px`,
                  '--panel-y': `${layer.y}px`,
                  '--depth': `${layer.depth}px`,
                  '--scale': layer.scale,
                  '--alpha': layer.alpha,
                  '--layer-width': `${layer.width}px`,
                  '--z': layer.z,
                  pointerEvents: layer.hitEvents,
                }"
                @click="showShiftWheel"
                @pointerdown="onShiftPointerDown"
                @pointermove="onShiftDrag"
                @pointerup="onShiftGrabEnd"
                @pointercancel="onShiftGrabCancel"
              >
                <span class="shift-layer__line"></span>
                <span class="shift-layer__meta">{{ layer.code }}</span>
              </span>
            </div>
          </div>
        </div>

        <div
          ref="galleryOrbitFieldRef"
          :class="['gallery-orbit-field', { 'is-hovering': galleryOrbitVisible, 'is-dragging': galleryOrbitDragging }]"
          aria-hidden="true"
          @click="showGalleryOrbit"
          @pointerdown="onGalleryOrbitPointerDown"
          @pointermove="onGalleryOrbitDrag"
          @pointerup="onGalleryOrbitGrabEnd"
          @pointercancel="onGalleryOrbitGrabCancel"
        ></div>

        <div
          ref="galleryOrbitStackRef"
          :class="[
            'gallery-art-stack',
            {
              'is-hovering': galleryOrbitVisible,
              'is-dragging': galleryOrbitDragging,
              'is-spinning': galleryOrbitSpinning,
            },
          ]"
          aria-label="画作跃迁轮盘"
        >
          <div class="gallery-orbit-label">
            <span>ART JUMP</span>
            <strong>{{ galleryOrbitStateText }}</strong>
          </div>
          <div class="gallery-module-shell" aria-hidden="true">
            <span class="gallery-module-shell__edge"></span>
          </div>
          <div class="gallery-orbit-window">
            <div class="gallery-orbit-track">
              <button
                v-for="layer in galleryOrbitLayers"
                :key="layer.id"
                type="button"
                class="gallery-orbit-layer"
                :aria-label="`双击跳转到画作：${layer.title}`"
                :title="`${layer.title} / 双击打开`"
                :style="{
                  '--panel-x': `${layer.x}px`,
                  '--panel-y': `${layer.y}px`,
                  '--depth': `${layer.depth}px`,
                  '--scale': layer.scale,
                  '--alpha': layer.alpha,
                  '--layer-width': `${layer.width}px`,
                  '--layer-height': `${layer.height}px`,
                  '--z': layer.z,
                  pointerEvents: layer.hitEvents,
                }"
                @click="onGalleryOrbitLayerClick"
                @dblclick="openGalleryOrbitArtwork(layer.to, $event)"
                @pointerdown="onGalleryOrbitPointerDown"
                @pointermove="onGalleryOrbitDrag"
                @pointerup="onGalleryOrbitGrabEnd"
                @pointercancel="onGalleryOrbitGrabCancel"
              >
                <img :src="layer.imageUrl" :alt="layer.title" draggable="false" />
                <span class="gallery-orbit-layer__title">{{ layer.title }}</span>
              </button>
            </div>
          </div>
        </div>

        <div class="time-device" aria-label="画廊数据时间环">
          <div class="orbit orbit-outer"></div>
          <div class="orbit orbit-middle"></div>
          <div class="orbit orbit-inner"></div>
          <div class="ratio-orbit"></div>
          <div class="tick-ring"></div>
          <div class="scan-sweep"></div>

          <div class="ratio-note note-haruhi">
            <strong>{{ haruhiRatio }}%</strong>
            <span>凉宫画作</span>
          </div>
          <div class="ratio-note note-other">
            <strong>{{ otherRatio }}%</strong>
            <span>非凉宫画作</span>
          </div>

          <div class="observer-core">
            <span class="loop-stamp">LOOP {{ loopCode }}</span>
            <p>
              画廊的第 <strong>{{ visitorNumberText }}</strong> 位访问者，你好
            </p>
            <div class="core-split">
              <span>{{ haruhiCount }} Haruhi</span>
              <span>{{ otherCount }} Other</span>
            </div>
          </div>
        </div>

        <article
          v-for="metric in satelliteMetrics"
          :key="metric.key"
          :class="['satellite-node', `node-${metric.key}`]"
        >
          <span>{{ metric.label }}</span>
          <strong>{{ metric.value }}</strong>
          <small>{{ metric.note }}</small>
        </article>
      </div>
    </div>

    <div class="bottom-grid">
      <article class="endless-panel repeat-panel">
        <div class="panel-head">
          <p class="eyebrow">Repeated Observation Log</p>
          <h2>重复观测记录</h2>
        </div>
        <div class="record-list">
          <RouterLink
            v-for="record in repeatRecords"
            :key="record.code"
            class="record-row"
            :to="{ name: 'gallery', query: { artwork: record.id } }"
          >
            <span class="record-code">{{ record.code }}</span>
            <div>
              <strong>{{ record.title }}</strong>
              <small>{{ record.meta }}</small>
            </div>
            <span class="record-time">{{ record.time }}</span>
          </RouterLink>
        </div>
      </article>

      <article class="endless-panel tag-panel">
        <div class="panel-head">
          <p class="eyebrow">Tag Loop Spectrum</p>
          <h2>标签循环频谱</h2>
        </div>
        <div class="tag-radar">
          <div class="tag-row" v-for="tag in topTags" :key="tag.name">
            <div class="tag-label">
              <span>#{{ tag.name }}</span>
              <strong>{{ tag.count }}</strong>
            </div>
            <div class="tag-track">
              <span :style="{ width: `${tag.percent}%` }"></span>
            </div>
          </div>
        </div>
      </article>
    </div>
    </div>

    <div class="home-lights-on-ui">
      <div class="day-gallery-wall" aria-hidden="true">
        <span class="day-wash"></span>
        <figure
          v-for="tile in lightWallTiles"
          :key="tile.key"
          class="wall-tile"
          :class="`wall-tile-${tile.tone}`"
          :style="{
            '--tile-x': `${tile.x}%`,
            '--tile-y': `${tile.y}%`,
            '--tile-w': `${tile.width}px`,
            '--tile-r': `${tile.rotate}deg`,
            '--tile-z': tile.z,
          }"
        >
          <img :src="tile.imageUrl" :alt="tile.title" draggable="false" />
        </figure>
      </div>

      <section class="day-hero-panel">
        <p class="day-kicker">Haruhi Fanclub Gallery</p>
        <h1>开灯展厅已就绪</h1>
        <p class="day-visitor">画廊的第 <strong>{{ visitorNumberText }}</strong> 位访问者，你好</p>
        <div class="day-stats" aria-label="本地画廊统计">
          <span v-for="item in lightStats" :key="item.label">
            <strong>{{ item.value }}</strong>
            <small>{{ item.label }}</small>
          </span>
        </div>
      </section>

      <section class="day-curation-strip" aria-label="本地 seed 展示">
        <article v-for="artwork in lightFeaturedArtworks" :key="artwork.id" class="day-art-card">
          <img :src="artwork.image_url" :alt="artwork.title" draggable="false" />
          <div>
            <strong>{{ artwork.title }}</strong>
            <span>{{ artwork.content_type === 'haruhi' ? '凉宫观测' : '社团应援' }}</span>
          </div>
        </article>
      </section>
    </div>
  </section>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { seedArtworks, seedCreators } from '../mock/seedData'

const router = useRouter()
const VISITOR_KEY = 'haruhi-art-visitor-number'
const DAY_MS = 24 * 60 * 60 * 1000
const SHIFT_LAYER_COUNT = 32
const SHIFT_VISIBLE_SIDE = 5.35
const SHIFT_ANGLE_STEP = 360 / SHIFT_LAYER_COUNT
const SHIFT_WHEEL_RADIUS_Y = 388
const SHIFT_WHEEL_RADIUS_X = 244
const SHIFT_WHEEL_DEPTH = 210
const SHIFT_DRAG_SPEED = 0.34
const SHIFT_MAX_VELOCITY = 1.25
const SHIFT_INERTIA_DECAY = 0.94
const SHIFT_STOP_VELOCITY = 0.012

function makeVisitorNumber() {
  const fallback = 5200 + seedArtworks.length * 31 + seedCreators.length * 17

  if (typeof window === 'undefined') return fallback

  const saved = window.localStorage.getItem(VISITOR_KEY)
  const parsed = Number(saved)
  if (Number.isFinite(parsed) && parsed > 0) return parsed

  const daySeed = Math.floor(Date.now() / DAY_MS)
  const generated = 5200 + (daySeed % 1000) * 9 + Math.floor(Math.random() * 180)
  window.localStorage.setItem(VISITOR_KEY, String(generated))
  return generated
}

function formatShortTime(value) {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

function getSummerCountdown() {
  const nowDate = new Date()
  const year = nowDate.getFullYear()
  let end = new Date(year, 7, 31, 23, 59, 59)
  if (nowDate > end) end = new Date(year + 1, 7, 31, 23, 59, 59)
  return Math.max(0, Math.ceil((end.getTime() - nowDate.getTime()) / DAY_MS))
}

const visitorNumber = makeVisitorNumber()
const visitorNumberText = visitorNumber.toLocaleString('zh-CN')
const loopCode = String(15000 + (visitorNumber % 532)).padStart(5, '0')

const approvedArtworks = seedArtworks.filter((item) => item.status === 'approved')
const artworkCount = approvedArtworks.length
const creatorCount = seedCreators.length
const haruhiCount = approvedArtworks.filter((item) => item.content_type === 'haruhi').length
const otherCount = Math.max(artworkCount - haruhiCount, 0)
const haruhiRatio = artworkCount ? Math.round((haruhiCount / artworkCount) * 100) : 0
const otherRatio = Math.max(0, 100 - haruhiRatio)
const totalLikes = approvedArtworks.reduce((sum, item) => sum + Number(item.like_total || 0), 0)

const latestArtwork = approvedArtworks
  .slice()
  .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())[0]

const latestUploadText = latestArtwork ? formatShortTime(latestArtwork.created_at) : '暂无'
const summerCountdown = getSummerCountdown()

const now = Date.now()
const todayCount = Math.max(
  1,
  approvedArtworks.filter((item) => now - new Date(item.created_at).getTime() <= DAY_MS * 1.2).length
)
const weekCount = approvedArtworks.filter((item) => now - new Date(item.created_at).getTime() <= DAY_MS * 7.2).length
const heatScore = Math.min(99, Math.round(totalLikes * 2 + (visitorNumber % 23)))

const satelliteMetrics = [
  { key: 'artworks', label: '当前画作', value: artworkCount, note: 'approved seed' },
  { key: 'creators', label: '创作者', value: creatorCount, note: '本地观测对象' },
  { key: 'latest', label: '最近上传', value: latestUploadText, note: latestArtwork?.title || '暂无作品' },
  { key: 'likes', label: '应援热度', value: `${heatScore}%`, note: `${totalLikes} 次点赞` },
  { key: 'week', label: '本周入库', value: weekCount, note: `今日新增 ${todayCount}` },
]

const shiftHovering = ref(false)
const shiftDragging = ref(false)
const shiftSpinning = ref(false)
const shiftRotation = ref(0)
const shiftHoverFieldRef = ref(null)
const shiftStackRef = ref(null)
let shiftLastY = null
let shiftLastTime = 0
let shiftPointerId = null
let shiftVelocity = 0
let shiftInertiaFrame = 0
let shiftInertiaLastTime = 0
let shiftMoved = false
const galleryOrbitVisible = ref(false)
const galleryOrbitDragging = ref(false)
const galleryOrbitSpinning = ref(false)
const galleryOrbitRotation = ref(0)
const galleryOrbitFieldRef = ref(null)
const galleryOrbitStackRef = ref(null)
let galleryOrbitLastY = null
let galleryOrbitLastTime = 0
let galleryOrbitPointerId = null
let galleryOrbitVelocity = 0
let galleryOrbitInertiaFrame = 0
let galleryOrbitInertiaLastTime = 0
let galleryOrbitMoved = false
let galleryOrbitSuppressClickUntil = 0

function normalizeShiftAngle(value) {
  let next = value % 360
  if (next > 180) next -= 360
  if (next < -180) next += 360
  return next
}

function normalizeShiftRotation(value) {
  let next = value % 360
  if (next < 0) next += 360
  return next
}

const timeShiftLayers = computed(() =>
  Array.from({ length: SHIFT_LAYER_COUNT }, (_, index) => {
    const angle = normalizeShiftAngle(index * SHIFT_ANGLE_STEP + shiftRotation.value)
    const radians = (angle * Math.PI) / 180
    const sideDistance = Math.abs(angle) / SHIFT_ANGLE_STEP
    const inVisibleSide = sideDistance <= SHIFT_VISIBLE_SIDE
    const edgeWeight = inVisibleSide ? Math.max(0, 1 - sideDistance / SHIFT_VISIBLE_SIDE) : 0
    const depthWeight = Math.max(0, Math.cos(radians))
    const codeNumber = String(index + 1).padStart(2, '0')

    return {
      id: `shift-${index}`,
      code: index === 0 ? 'PHASE-00' : `LAYER-${codeNumber}`,
      x: Number((Math.cos(radians) * SHIFT_WHEEL_RADIUS_X).toFixed(2)),
      y: Number((Math.sin(radians) * SHIFT_WHEEL_RADIUS_Y).toFixed(2)),
      depth: Number((Math.cos(radians) * SHIFT_WHEEL_DEPTH).toFixed(2)),
      scale: Number((0.72 + depthWeight * 0.3).toFixed(3)),
      alpha: Number((inVisibleSide ? 0.34 + edgeWeight * 0.64 : 0).toFixed(3)),
      width: Number((306 + depthWeight * 74).toFixed(2)),
      z: Math.round(20 + depthWeight * 120),
      hitEvents: shiftHovering.value && inVisibleSide ? 'auto' : 'none',
    }
  })
)

const galleryOrbitLayers = computed(() => {
  const source = approvedArtworks.length ? approvedArtworks : seedArtworks
  if (!source.length) return []

  return Array.from({ length: SHIFT_LAYER_COUNT }, (_, index) => {
    const artwork = source[index % source.length]
    const angle = normalizeShiftAngle(index * SHIFT_ANGLE_STEP + galleryOrbitRotation.value)
    const radians = (angle * Math.PI) / 180
    const sideDistance = Math.abs(angle) / SHIFT_ANGLE_STEP
    const inVisibleSide = sideDistance <= SHIFT_VISIBLE_SIDE
    const edgeWeight = inVisibleSide ? Math.max(0, 1 - sideDistance / SHIFT_VISIBLE_SIDE) : 0
    const depthWeight = Math.max(0, Math.cos(radians))
    const layerHeight = 154 + depthWeight * 82

    return {
      id: `gallery-orbit-${index}-${artwork.id}`,
      artwork,
      title: artwork.title,
      imageUrl: artwork.image_url,
      to: { name: 'gallery', query: { artwork: artwork.id } },
      x: Number((Math.cos(radians) * SHIFT_WHEEL_RADIUS_X).toFixed(2)),
      y: Number((Math.sin(radians) * SHIFT_WHEEL_RADIUS_Y).toFixed(2)),
      depth: Number((Math.cos(radians) * SHIFT_WHEEL_DEPTH).toFixed(2)),
      scale: Number((0.72 + depthWeight * 0.32).toFixed(3)),
      alpha: Number((inVisibleSide ? 0.36 + edgeWeight * 0.62 : 0).toFixed(3)),
      width: Number((layerHeight * 1.05).toFixed(2)),
      height: Number(layerHeight.toFixed(2)),
      z: Math.round(20 + depthWeight * 260),
      hitEvents: galleryOrbitVisible.value && inVisibleSide ? 'auto' : 'none',
    }
  })
})

const shiftStateText = computed(() => {
  if (shiftDragging.value) return '拖拽跃迁'
  if (shiftSpinning.value) return '惯性回环'
  if (shiftHovering.value) return '边缘抓取'
  return '边缘待命'
})

const galleryOrbitStateText = computed(() => {
  if (galleryOrbitDragging.value) return '拖拽索引'
  if (galleryOrbitSpinning.value) return '惯性巡游'
  if (galleryOrbitVisible.value) return '画作可选'
  return '边缘待命'
})

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value))
}

function applyShiftRotation(delta) {
  shiftRotation.value = normalizeShiftRotation(shiftRotation.value + delta)
}

function cancelShiftInertia() {
  if (shiftInertiaFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(shiftInertiaFrame)
  }
  shiftInertiaFrame = 0
  shiftInertiaLastTime = 0
  shiftSpinning.value = false
}

function hideShiftWheel() {
  shiftHovering.value = false
  shiftDragging.value = false
  shiftSpinning.value = false
  shiftLastY = null
  shiftLastTime = 0
  shiftPointerId = null
  shiftVelocity = 0
  shiftMoved = false
}

function releaseShiftCapture(event) {
  const target = event.currentTarget
  if (target?.hasPointerCapture?.(event.pointerId)) {
    target.releasePointerCapture(event.pointerId)
  }
}

function showShiftWheel(event) {
  if (event?.button !== undefined && event.button !== 0) return
  if (shiftHovering.value) return
  cancelShiftInertia()
  shiftHovering.value = true
}

function onShiftPointerDown(event) {
  if (event.button !== undefined && event.button !== 0) return

  if (!shiftHovering.value) {
    return
  }

  event.preventDefault()
  cancelShiftInertia()
  shiftDragging.value = true
  shiftPointerId = event.pointerId
  shiftLastY = event.clientY
  shiftLastTime = event.timeStamp || performance.now()
  shiftVelocity = 0
  shiftMoved = false
  event.currentTarget?.setPointerCapture?.(event.pointerId)
}

function onShiftDrag(event) {
  if (!shiftDragging.value || event.pointerId !== shiftPointerId) return
  event.preventDefault()

  const nowTime = event.timeStamp || performance.now()
  if (shiftLastY === null) {
    shiftLastY = event.clientY
    shiftLastTime = nowTime
    return
  }

  const diff = event.clientY - shiftLastY
  const elapsed = Math.max(8, nowTime - shiftLastTime)
  if (Math.abs(diff) >= 0.2) {
    const rotationDelta = diff * SHIFT_DRAG_SPEED
    const instantVelocity = rotationDelta / elapsed
    applyShiftRotation(rotationDelta)
    shiftMoved = true
    shiftVelocity = clamp(
      shiftVelocity * 0.28 + instantVelocity * 0.72,
      -SHIFT_MAX_VELOCITY,
      SHIFT_MAX_VELOCITY
    )
  }

  shiftLastY = event.clientY
  shiftLastTime = nowTime
}

function stopShiftInertia() {
  cancelShiftInertia()
  shiftVelocity = 0
}

function tickShiftInertia(time) {
  if (!shiftSpinning.value) {
    shiftInertiaFrame = 0
    return
  }

  if (!shiftInertiaLastTime) shiftInertiaLastTime = time
  const delta = Math.min(40, time - shiftInertiaLastTime)
  shiftInertiaLastTime = time

  applyShiftRotation(shiftVelocity * delta)
  shiftVelocity *= Math.pow(SHIFT_INERTIA_DECAY, delta / 16.67)

  if (Math.abs(shiftVelocity) <= SHIFT_STOP_VELOCITY) {
    stopShiftInertia()
    return
  }

  shiftInertiaFrame = window.requestAnimationFrame(tickShiftInertia)
}

function startShiftInertia() {
  if (typeof window === 'undefined' || !shiftMoved || Math.abs(shiftVelocity) <= SHIFT_STOP_VELOCITY) {
    stopShiftInertia()
    return
  }

  shiftSpinning.value = true
  shiftInertiaLastTime = 0
  shiftInertiaFrame = window.requestAnimationFrame(tickShiftInertia)
}

function onShiftGrabEnd(event) {
  if (!shiftDragging.value || event.pointerId !== shiftPointerId) return
  event.preventDefault()
  releaseShiftCapture(event)
  shiftDragging.value = false
  shiftPointerId = null
  shiftLastY = null
  shiftLastTime = 0
  startShiftInertia()
}

function onShiftGrabCancel(event) {
  if (event.pointerId !== shiftPointerId) return
  releaseShiftCapture(event)
  stopShiftInertia()
  shiftDragging.value = false
  shiftPointerId = null
}

function applyGalleryOrbitRotation(delta) {
  galleryOrbitRotation.value = normalizeShiftRotation(galleryOrbitRotation.value + delta)
}

function cancelGalleryOrbitInertia() {
  if (galleryOrbitInertiaFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(galleryOrbitInertiaFrame)
  }
  galleryOrbitInertiaFrame = 0
  galleryOrbitInertiaLastTime = 0
  galleryOrbitSpinning.value = false
}

function hideGalleryOrbit() {
  galleryOrbitVisible.value = false
  galleryOrbitDragging.value = false
  galleryOrbitSpinning.value = false
  galleryOrbitLastY = null
  galleryOrbitLastTime = 0
  galleryOrbitPointerId = null
  galleryOrbitVelocity = 0
  galleryOrbitMoved = false
}

function showGalleryOrbit(event) {
  if (event?.button !== undefined && event.button !== 0) return
  if (galleryOrbitVisible.value) return
  cancelGalleryOrbitInertia()
  galleryOrbitVisible.value = true
}

function onGalleryOrbitPointerDown(event) {
  if (event.button !== undefined && event.button !== 0) return
  if (!galleryOrbitVisible.value) return

  cancelGalleryOrbitInertia()
  galleryOrbitDragging.value = true
  galleryOrbitPointerId = event.pointerId
  galleryOrbitLastY = event.clientY
  galleryOrbitLastTime = event.timeStamp || performance.now()
  galleryOrbitVelocity = 0
  galleryOrbitMoved = false
  event.currentTarget?.setPointerCapture?.(event.pointerId)
}

function onGalleryOrbitDrag(event) {
  if (!galleryOrbitDragging.value || event.pointerId !== galleryOrbitPointerId) return
  event.preventDefault()

  const nowTime = event.timeStamp || performance.now()
  if (galleryOrbitLastY === null) {
    galleryOrbitLastY = event.clientY
    galleryOrbitLastTime = nowTime
    return
  }

  const diff = event.clientY - galleryOrbitLastY
  const elapsed = Math.max(8, nowTime - galleryOrbitLastTime)
  if (Math.abs(diff) >= 0.2) {
    const rotationDelta = diff * SHIFT_DRAG_SPEED
    const instantVelocity = rotationDelta / elapsed
    applyGalleryOrbitRotation(rotationDelta)
    galleryOrbitMoved = true
    galleryOrbitVelocity = clamp(
      galleryOrbitVelocity * 0.28 + instantVelocity * 0.72,
      -SHIFT_MAX_VELOCITY,
      SHIFT_MAX_VELOCITY
    )
  }

  galleryOrbitLastY = event.clientY
  galleryOrbitLastTime = nowTime
}

function stopGalleryOrbitInertia() {
  cancelGalleryOrbitInertia()
  galleryOrbitVelocity = 0
}

function tickGalleryOrbitInertia(time) {
  if (!galleryOrbitSpinning.value) {
    galleryOrbitInertiaFrame = 0
    return
  }

  if (!galleryOrbitInertiaLastTime) galleryOrbitInertiaLastTime = time
  const delta = Math.min(40, time - galleryOrbitInertiaLastTime)
  galleryOrbitInertiaLastTime = time

  applyGalleryOrbitRotation(galleryOrbitVelocity * delta)
  galleryOrbitVelocity *= Math.pow(SHIFT_INERTIA_DECAY, delta / 16.67)

  if (Math.abs(galleryOrbitVelocity) <= SHIFT_STOP_VELOCITY) {
    stopGalleryOrbitInertia()
    return
  }

  galleryOrbitInertiaFrame = window.requestAnimationFrame(tickGalleryOrbitInertia)
}

function startGalleryOrbitInertia() {
  if (typeof window === 'undefined' || !galleryOrbitMoved || Math.abs(galleryOrbitVelocity) <= SHIFT_STOP_VELOCITY) {
    stopGalleryOrbitInertia()
    return
  }

  galleryOrbitSpinning.value = true
  galleryOrbitInertiaLastTime = 0
  galleryOrbitInertiaFrame = window.requestAnimationFrame(tickGalleryOrbitInertia)
}

function onGalleryOrbitGrabEnd(event) {
  if (!galleryOrbitDragging.value || event.pointerId !== galleryOrbitPointerId) return
  if (galleryOrbitMoved) {
    event.preventDefault()
    galleryOrbitSuppressClickUntil = (event.timeStamp || performance.now()) + 220
  }
  releaseShiftCapture(event)
  galleryOrbitDragging.value = false
  galleryOrbitPointerId = null
  galleryOrbitLastY = null
  galleryOrbitLastTime = 0
  startGalleryOrbitInertia()
}

function onGalleryOrbitGrabCancel(event) {
  if (event.pointerId !== galleryOrbitPointerId) return
  releaseShiftCapture(event)
  stopGalleryOrbitInertia()
  galleryOrbitDragging.value = false
  galleryOrbitPointerId = null
}

function onGalleryOrbitLayerClick(event) {
  const nowTime = event.timeStamp || performance.now()
  if (nowTime < galleryOrbitSuppressClickUntil) {
    event.preventDefault()
    event.stopPropagation()
  }
}

function openGalleryOrbitArtwork(to, event) {
  const nowTime = event?.timeStamp || performance.now()
  if (nowTime < galleryOrbitSuppressClickUntil || galleryOrbitDragging.value) {
    event?.preventDefault()
    event?.stopPropagation()
    return
  }

  event?.preventDefault()
  event?.stopPropagation()
  router.push(to)
}

function onGlobalShiftContextMenu(event) {
  const shiftActive = shiftHovering.value || shiftDragging.value || shiftSpinning.value
  const galleryActive = galleryOrbitVisible.value || galleryOrbitDragging.value || galleryOrbitSpinning.value
  if (!shiftActive && !galleryActive) return
  event.preventDefault()
  cancelShiftInertia()
  hideShiftWheel()
  cancelGalleryOrbitInertia()
  hideGalleryOrbit()
}

onMounted(() => {
  window.addEventListener('contextmenu', onGlobalShiftContextMenu, true)
})

onBeforeUnmount(() => {
  cancelShiftInertia()
  cancelGalleryOrbitInertia()
  window.removeEventListener('contextmenu', onGlobalShiftContextMenu, true)
})

const tagCounts = approvedArtworks.reduce((map, item) => {
  for (const tag of item.tags || []) {
    map.set(tag, (map.get(tag) || 0) + 1)
  }
  return map
}, new Map())

const maxTagCount = Math.max(...tagCounts.values(), 1)
const topTags = Array.from(tagCounts.entries())
  .map(([name, count]) => ({
    name,
    count,
    percent: Math.max(14, Math.round((count / maxTagCount) * 100)),
  }))
  .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name, 'zh-CN'))
  .slice(0, 6)

const repeatRecords = approvedArtworks
  .slice()
  .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
  .slice(0, 4)
  .map((item, index) => ({
    id: item.id,
    code: `OBS-${String(index + 1).padStart(2, '0')}`,
    title: item.title,
    meta: `${item.content_type === 'haruhi' ? '凉宫线' : '支线'} / ${(item.tags || []).slice(0, 2).join(' · ') || '未标记'}`,
    time: formatShortTime(item.created_at),
  }))

const lightFeaturedArtworks = (approvedArtworks.length ? approvedArtworks : seedArtworks).slice(0, 5)

const lightStats = [
  { label: '当前画作', value: artworkCount },
  { label: '创作者', value: creatorCount },
  { label: '凉宫占比', value: `${haruhiRatio}%` },
  { label: '最近上传', value: latestUploadText },
]

const wallFrames = [
  { x: 3, y: 4, width: 168, rotate: -15, tone: 'cyan' },
  { x: 16, y: 10, width: 148, rotate: -15, tone: 'pink' },
  { x: 28, y: 2, width: 176, rotate: -15, tone: 'violet' },
  { x: 42, y: 8, width: 154, rotate: -15, tone: 'blue' },
  { x: 55, y: 1, width: 184, rotate: -15, tone: 'cyan' },
  { x: 70, y: 8, width: 144, rotate: -15, tone: 'pink' },
  { x: 82, y: 2, width: 174, rotate: -15, tone: 'violet' },
  { x: 8, y: 32, width: 182, rotate: -15, tone: 'blue' },
  { x: 23, y: 27, width: 154, rotate: -15, tone: 'cyan' },
  { x: 36, y: 35, width: 174, rotate: -15, tone: 'pink' },
  { x: 52, y: 29, width: 150, rotate: -15, tone: 'violet' },
  { x: 66, y: 34, width: 188, rotate: -15, tone: 'blue' },
  { x: 82, y: 28, width: 158, rotate: -15, tone: 'cyan' },
  { x: 1, y: 64, width: 176, rotate: -15, tone: 'pink' },
  { x: 17, y: 58, width: 190, rotate: -15, tone: 'violet' },
  { x: 34, y: 67, width: 152, rotate: -15, tone: 'blue' },
  { x: 48, y: 60, width: 180, rotate: -15, tone: 'cyan' },
  { x: 63, y: 70, width: 146, rotate: -15, tone: 'pink' },
  { x: 75, y: 60, width: 184, rotate: -15, tone: 'violet' },
  { x: 90, y: 68, width: 152, rotate: -15, tone: 'blue' },
]

const lightWallSource = approvedArtworks.length ? approvedArtworks : seedArtworks
const lightWallTiles = lightWallSource.flatMap((artwork, artworkIndex) =>
  Array.from({ length: 2 }, (_, copyIndex) => {
    const frame = wallFrames[(artworkIndex * 2 + copyIndex) % wallFrames.length]
    return {
      key: `${artwork.id}-${copyIndex}`,
      title: artwork.title,
      imageUrl: artwork.image_url,
      ...frame,
      z: (artworkIndex + copyIndex) % 6,
    }
  })
)

function getSectorLabelPosition(angle, radius = 34) {
  const radians = (angle * Math.PI) / 180
  return {
    x: `${(50 + Math.sin(radians) * radius).toFixed(2)}%`,
    y: `${(50 - Math.cos(radians) * radius).toFixed(2)}%`,
  }
}

const haruhiAngle = haruhiRatio * 3.6
const haruhiLabelPosition = getSectorLabelPosition(-90 + haruhiAngle / 2)
const otherLabelPosition = getSectorLabelPosition(-90 + haruhiAngle + (360 - haruhiAngle) / 2)

const stageStyle = {
  '--haruhi-angle': `${haruhiAngle}deg`,
  '--haruhi-label-x': haruhiLabelPosition.x,
  '--haruhi-label-y': haruhiLabelPosition.y,
  '--other-label-x': otherLabelPosition.x,
  '--other-label-y': otherLabelPosition.y,
  '--heat-level': `${heatScore}%`,
  '--week-level': `${Math.min(100, weekCount * 18)}%`,
  '--loop-offset': `${visitorNumber % 360}deg`,
}
</script>

<style scoped>
.art-home {
  width: min(1500px, calc(100% - 32px));
  padding-top: 8px;
  position: relative;
  isolation: isolate;
  --space-bg: #050814;
  --space-bg-2: #091427;
  --space-bg-3: #150d2a;
  --hud-panel: rgba(8, 18, 38, 0.72);
  --hud-panel-strong: rgba(10, 24, 50, 0.88);
  --hud-line: rgba(122, 211, 255, 0.24);
  --hud-line-strong: rgba(126, 227, 255, 0.58);
  --hud-text: rgba(239, 247, 255, 0.96);
  --hud-muted: rgba(183, 204, 232, 0.72);
  --hud-cyan: #74e7ff;
  --hud-blue: #5d8cff;
  --hud-violet: #b18cff;
  --hud-red: #ff637d;
}

.art-home .home-lights-out-ui {
  display: none;
}

:global(html.art-lights-out) .art-home .home-lights-out-ui {
  display: block;
}

:global(html.art-lights-out) .art-home .home-lights-on-ui {
  display: none;
}

.art-home::before,
.art-home::after {
  content: "";
  position: absolute;
  display: none;
  pointer-events: none;
}

:global(html.art-lights-out) .art-home::before,
:global(html.art-lights-out) .art-home::after {
  display: block;
}

.art-home .home-lights-on-ui {
  position: relative;
  min-height: calc(100dvh - 136px);
  overflow: hidden;
  display: grid;
  align-content: center;
  gap: 24px;
  padding: clamp(36px, 6vw, 84px);
  border: 1px solid rgba(42, 110, 116, 0.16);
  border-radius: 28px;
  background:
    radial-gradient(circle at 18% 16%, rgba(255, 255, 255, 0.86), transparent 26%),
    radial-gradient(circle at 76% 10%, rgba(126, 221, 231, 0.26), transparent 26%),
    linear-gradient(135deg, rgba(255, 251, 244, 0.9), rgba(234, 249, 247, 0.9) 48%, rgba(255, 242, 235, 0.92));
  box-shadow: 0 28px 90px rgba(38, 88, 96, 0.16), inset 0 1px 0 rgba(255, 255, 255, 0.92);
  color: #12333c;
}

.art-home .day-gallery-wall {
  position: absolute;
  inset: -8% -7%;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
  transform: rotate(-15deg) scale(1.06);
  transform-origin: center;
}

.art-home .day-wash {
  position: absolute;
  inset: -12%;
  z-index: 5;
  background:
    radial-gradient(ellipse at 50% 42%, rgba(255, 255, 255, 0.38), transparent 32%),
    linear-gradient(90deg, rgba(255, 248, 238, 0.68), rgba(245, 253, 252, 0.38), rgba(255, 246, 238, 0.72));
}

.art-home .wall-tile {
  position: absolute;
  left: var(--tile-x);
  top: var(--tile-y);
  z-index: var(--tile-z);
  width: var(--tile-w);
  aspect-ratio: 1 / 1;
  margin: 0;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.72);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 18px 42px rgba(39, 86, 95, 0.18), inset 0 1px 0 rgba(255, 255, 255, 0.86);
  transform: translate(-50%, -50%) rotate(var(--tile-r));
}

.art-home .wall-tile img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  filter: saturate(1.08) contrast(1.02);
}

.art-home .wall-tile-cyan {
  box-shadow: 0 18px 44px rgba(50, 177, 194, 0.18);
}

.art-home .wall-tile-pink {
  box-shadow: 0 18px 44px rgba(226, 88, 128, 0.16);
}

.art-home .wall-tile-violet {
  box-shadow: 0 18px 44px rgba(126, 96, 210, 0.16);
}

.art-home .wall-tile-blue {
  box-shadow: 0 18px 44px rgba(74, 132, 210, 0.16);
}

.art-home .day-hero-panel,
.art-home .day-curation-strip {
  position: relative;
  z-index: 2;
}

.art-home .day-hero-panel {
  width: min(760px, 100%);
  padding: clamp(26px, 4vw, 46px);
  border: 1px solid rgba(255, 255, 255, 0.72);
  border-radius: 26px;
  background:
    radial-gradient(circle at 14% 0%, rgba(255, 255, 255, 0.92), transparent 32%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.82), rgba(247, 252, 250, 0.68));
  box-shadow: 0 24px 70px rgba(26, 71, 80, 0.18), inset 0 1px 0 rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(18px);
}

.art-home .day-kicker {
  margin: 0 0 12px;
  color: #d94667;
  font-size: 13px;
  font-weight: 950;
  letter-spacing: 0;
  text-transform: uppercase;
}

.art-home .day-hero-panel h1 {
  margin: 0;
  color: #0f2c35;
  font-size: clamp(42px, 7vw, 86px);
  line-height: 0.95;
  font-weight: 950;
  text-shadow: 0 10px 32px rgba(60, 155, 170, 0.14);
}

.art-home .day-visitor {
  margin: 18px 0 0;
  color: rgba(18, 51, 60, 0.76);
  font-size: clamp(17px, 2vw, 22px);
  font-weight: 900;
}

.art-home .day-visitor strong {
  color: #1b8b9b;
  font-size: 1.35em;
  font-weight: 950;
}

.art-home .day-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  margin-top: 26px;
}

.art-home .day-stats span {
  min-width: 0;
  padding: 13px 14px;
  border: 1px solid rgba(42, 110, 116, 0.14);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.62);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.74);
}

.art-home .day-stats strong,
.art-home .day-stats small {
  display: block;
}

.art-home .day-stats strong {
  overflow: hidden;
  color: #12333c;
  font-size: clamp(18px, 2vw, 28px);
  font-weight: 950;
  line-height: 1.05;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.art-home .day-stats small {
  margin-top: 5px;
  color: rgba(18, 51, 60, 0.58);
  font-size: 12px;
  font-weight: 900;
}

.art-home .day-curation-strip {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 12px;
}

.art-home .day-art-card {
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.7);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.74);
  box-shadow: 0 16px 42px rgba(26, 71, 80, 0.14), inset 0 1px 0 rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(14px);
  transition: transform 0.22s ease, box-shadow 0.22s ease;
}

.art-home .day-art-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 22px 54px rgba(26, 71, 80, 0.2), 0 0 24px rgba(47, 171, 190, 0.14);
}

.art-home .day-art-card img {
  width: 100%;
  aspect-ratio: 4 / 3;
  object-fit: cover;
}

.art-home .day-art-card div {
  display: grid;
  gap: 4px;
  padding: 12px;
}

.art-home .day-art-card strong {
  overflow: hidden;
  color: #12333c;
  font-size: 14px;
  font-weight: 950;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.art-home .day-art-card span {
  color: rgba(18, 51, 60, 0.6);
  font-size: 12px;
  font-weight: 900;
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

.art-home .endless-screen {
  position: relative;
  min-height: 720px;
  overflow: hidden;
  border: 1px solid var(--hud-line);
  border-radius: 22px;
  background:
    radial-gradient(ellipse at 48% 40%, rgba(116, 231, 255, 0.12), transparent 31%),
    radial-gradient(ellipse at 22% 16%, rgba(116, 231, 255, 0.09), transparent 29%),
    radial-gradient(ellipse at 82% 18%, rgba(177, 140, 255, 0.12), transparent 32%),
    radial-gradient(ellipse at 55% 88%, rgba(0, 0, 0, 0.55), transparent 42%),
    linear-gradient(135deg, var(--space-bg), var(--space-bg-2) 54%, var(--space-bg-3));
  box-shadow:
    0 34px 120px rgba(0, 0, 0, 0.44),
    0 0 56px rgba(80, 174, 255, 0.13),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
  color: var(--hud-text);
}

.art-home .endless-screen::before,
.art-home .endless-screen::after {
  content: "";
  position: absolute;
  pointer-events: none;
}

.art-home .endless-screen::before {
  inset: 18px;
  z-index: 1;
  border: 1px solid rgba(122, 211, 255, 0.12);
  border-radius: 16px;
  background:
    linear-gradient(rgba(116, 231, 255, 0.06) 1px, transparent 1px),
    linear-gradient(90deg, rgba(116, 231, 255, 0.06) 1px, transparent 1px);
  background-size: 34px 34px;
  clip-path: polygon(0 18px, 18px 0, calc(100% - 18px) 0, 100% 18px, 100% calc(100% - 18px), calc(100% - 18px) 100%, 18px 100%, 0 calc(100% - 18px));
  mask-image: radial-gradient(circle at center, black 0 61%, transparent 80%);
}

.art-home .endless-screen::after {
  inset: 0;
  z-index: 1;
  background:
    repeating-linear-gradient(0deg, rgba(150, 220, 255, 0.07) 0 1px, transparent 1px 8px),
    linear-gradient(110deg, transparent 18%, rgba(116, 231, 255, 0.12) 48%, transparent 72%);
  opacity: 0.76;
  mix-blend-mode: screen;
}

.art-home .space-field,
.art-home .space-field span {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.art-home .space-field {
  z-index: 0;
  overflow: hidden;
  background:
    radial-gradient(ellipse at 46% 44%, transparent 0 30%, rgba(0, 0, 0, 0.16) 62%, rgba(0, 0, 0, 0.5) 100%),
    linear-gradient(135deg, rgba(1, 4, 13, 0.16), rgba(3, 7, 18, 0.7));
}

.art-home .star-dust,
.art-home .star-layer {
  background-repeat: repeat;
  will-change: background-position, transform;
}

.art-home .star-dust {
  background-image:
    radial-gradient(circle, rgba(255, 255, 255, 0.42) 0 0.55px, transparent 0.9px),
    radial-gradient(circle, rgba(156, 214, 255, 0.34) 0 0.45px, transparent 0.85px),
    radial-gradient(circle, rgba(210, 196, 255, 0.28) 0 0.4px, transparent 0.8px);
  background-position: 0 0, 37px 24px, 83px 58px;
  background-size: 76px 68px, 112px 104px, 148px 132px;
  opacity: 0.62;
  animation: homeStarDrift 240s linear infinite;
}

.art-home .star-layer-a {
  background-image:
    radial-gradient(circle, rgba(255, 255, 255, 0.78) 0 0.9px, transparent 1.35px),
    radial-gradient(circle, rgba(116, 231, 255, 0.54) 0 1px, transparent 1.55px);
  background-position: 0 0, 64px 38px;
  background-size: 132px 104px, 192px 148px;
  opacity: 0.7;
  animation: homeStarDrift 310s linear infinite reverse;
}

.art-home .star-layer-b {
  background-image:
    radial-gradient(circle, rgba(177, 140, 255, 0.58) 0 1.1px, transparent 1.7px),
    radial-gradient(circle, rgba(255, 99, 125, 0.36) 0 0.95px, transparent 1.55px),
    radial-gradient(circle, rgba(255, 255, 255, 0.74) 0 1.2px, transparent 1.9px);
  background-position: 32px 20px, 20px 70px, 140px 92px;
  background-size: 236px 196px, 318px 236px, 420px 360px;
  opacity: 0.5;
  animation: homeStarFloat 360s linear infinite;
}

.art-home .star-layer-c {
  background-image:
    radial-gradient(circle, rgba(255, 255, 255, 0.9) 0 1.4px, transparent 2.4px),
    radial-gradient(circle, rgba(141, 240, 255, 0.72) 0 1.2px, transparent 2.2px),
    radial-gradient(circle, rgba(255, 224, 176, 0.58) 0 1px, transparent 2px);
  background-position: 90px 80px, 270px 150px, 520px 310px;
  background-size: 540px 420px, 680px 520px, 760px 580px;
  opacity: 0.58;
  filter: drop-shadow(0 0 4px rgba(141, 240, 255, 0.32));
  animation: homeStarFloat 420s linear infinite reverse;
}

.art-home .nebula,
.art-home .galaxy-halo {
  inset: auto;
  border-radius: 50%;
  mix-blend-mode: screen;
}

.art-home .nebula {
  filter: blur(20px);
  opacity: 0.74;
  will-change: transform, opacity;
  animation: homeNebulaDrift 170s ease-in-out infinite alternate;
}

.art-home .nebula-a {
  width: 68%;
  height: 66%;
  right: -18%;
  top: -22%;
  background:
    radial-gradient(ellipse at 42% 42%, rgba(147, 112, 255, 0.28), transparent 46%),
    radial-gradient(ellipse at 68% 34%, rgba(91, 167, 255, 0.16), transparent 38%),
    radial-gradient(ellipse at 34% 68%, rgba(255, 99, 125, 0.08), transparent 38%);
}

.art-home .nebula-b {
  width: 58%;
  height: 54%;
  left: -16%;
  bottom: -20%;
  background:
    radial-gradient(ellipse at 46% 48%, rgba(28, 180, 255, 0.18), transparent 48%),
    radial-gradient(ellipse at 62% 62%, rgba(177, 140, 255, 0.13), transparent 40%);
  animation-duration: 210s;
}

.art-home .galaxy-halo {
  width: 168px;
  height: 70px;
  opacity: 0.58;
  filter: blur(0.4px) drop-shadow(0 0 18px rgba(141, 240, 255, 0.16));
  background:
    radial-gradient(ellipse at center, rgba(255, 255, 255, 0.62), rgba(141, 240, 255, 0.18) 18%, rgba(177, 140, 255, 0.08) 42%, transparent 70%),
    linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.22), transparent);
  animation: homeGalaxyDrift 260s ease-in-out infinite alternate;
}

.art-home .halo-a {
  top: 18%;
  left: 12%;
  transform: rotate(-14deg);
}

.art-home .halo-b {
  right: 10%;
  bottom: 20%;
  width: 124px;
  height: 52px;
  opacity: 0.38;
  transform: rotate(18deg);
  animation-name: homeGalaxyDriftB;
  animation-duration: 320s;
}

.art-home .bright-stars {
  background:
    radial-gradient(circle at 18% 28%, rgba(255, 255, 255, 0.96) 0 1.6px, rgba(141, 240, 255, 0.42) 2px, transparent 9px),
    radial-gradient(circle at 76% 22%, rgba(255, 255, 255, 0.9) 0 1.4px, rgba(177, 140, 255, 0.32) 2px, transparent 11px),
    radial-gradient(circle at 66% 72%, rgba(255, 255, 255, 0.82) 0 1px, rgba(116, 231, 255, 0.26) 2px, transparent 8px),
    radial-gradient(circle at 34% 74%, rgba(255, 233, 191, 0.78) 0 1px, rgba(255, 99, 125, 0.18) 2px, transparent 7px);
  opacity: 0.78;
  filter: drop-shadow(0 0 8px rgba(141, 240, 255, 0.25));
}

.art-home .void-shadow {
  background:
    radial-gradient(ellipse at 42% 54%, transparent 0 34%, rgba(0, 0, 0, 0.14) 58%, rgba(0, 0, 0, 0.56) 100%),
    radial-gradient(circle at 18% 78%, rgba(0, 0, 0, 0.56), transparent 26%),
    radial-gradient(circle at 84% 52%, rgba(0, 0, 0, 0.42), transparent 24%);
  opacity: 0.86;
}

.art-home .viewport-glass {
  z-index: 1;
  background:
    radial-gradient(ellipse at center, transparent 0 58%, rgba(0, 0, 0, 0.36) 86%, rgba(0, 0, 0, 0.68) 100%),
    linear-gradient(115deg, transparent 18%, rgba(255, 255, 255, 0.06) 48%, transparent 72%);
  box-shadow: inset 0 0 80px rgba(0, 0, 0, 0.52);
  opacity: 0.92;
}

.art-home .screen-header {
  position: relative;
  z-index: 3;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
  padding: 28px 32px 0;
}

.art-home .eyebrow {
  margin: 0 0 8px;
  color: var(--hud-cyan);
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0;
  text-transform: uppercase;
  text-shadow: 0 0 18px rgba(116, 231, 255, 0.48);
}

.art-home h1,
.art-home h2,
.art-home p {
  margin: 0;
}

.art-home h1 {
  color: var(--hud-text);
  font-size: 34px;
  line-height: 1.1;
  font-weight: 950;
  text-shadow: 0 0 28px rgba(116, 231, 255, 0.2);
}

.art-home .status-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding: 10px 14px;
  border: 1px solid var(--hud-line);
  border-radius: 999px;
  background: rgba(8, 18, 38, 0.58);
  color: var(--hud-muted);
  font-size: 13px;
  font-weight: 900;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 0 22px rgba(116, 231, 255, 0.08);
  backdrop-filter: blur(14px);
}

.art-home .status-chip span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--hud-cyan);
  box-shadow: 0 0 14px rgba(116, 231, 255, 0.86);
}

.art-home .visual-stage {
  position: relative;
  z-index: 2;
  min-height: 630px;
  display: grid;
  place-items: center;
  padding: 10px 32px 42px;
}

.art-home .visual-stage.is-gallery-orbit-active {
  z-index: 6;
}

.art-home .summer-strip {
  position: absolute;
  top: 40px;
  left: 50%;
  z-index: 5;
  display: inline-grid;
  grid-template-columns: auto auto auto;
  align-items: baseline;
  gap: 8px;
  transform: translateX(-50%);
  padding: 9px 16px;
  border: 1px solid rgba(116, 231, 255, 0.3);
  border-radius: 999px;
  background: rgba(5, 13, 28, 0.72);
  color: var(--hud-muted);
  font-weight: 950;
  box-shadow: 0 0 24px rgba(116, 231, 255, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
}

.art-home .summer-strip strong {
  color: var(--hud-red);
  font-size: 30px;
  line-height: 1;
  text-shadow: 0 0 18px rgba(255, 99, 125, 0.45);
}

.art-home .shift-hover-field {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 9;
  width: min(18vw, 156px);
  height: 100dvh;
  pointer-events: auto;
  background: transparent;
  cursor: grab;
  touch-action: none;
  transition: width 0.28s ease;
}

.art-home .shift-hover-field.is-hovering {
  width: min(30vw, 268px);
}

.art-home .shift-hover-field.is-dragging {
  cursor: grabbing;
}

.art-home .time-shift-stack {
  position: fixed;
  left: 0;
  top: 52dvh;
  z-index: 8;
  width: 760px;
  height: min(96dvh, 940px);
  min-height: 760px;
  transform: translate3d(-804px, -50%, 0) rotateY(-36deg);
  transform-origin: left center;
  transform-style: preserve-3d;
  perspective: 1400px;
  pointer-events: none;
  opacity: 0.24;
  filter: saturate(0.62) brightness(0.78);
  isolation: isolate;
  will-change: transform, opacity, filter;
  backface-visibility: hidden;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
  transition:
    opacity 0.28s ease,
    filter 0.28s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .time-shift-stack.is-hovering {
  opacity: 0.98;
  filter: saturate(1.18) brightness(1.1) contrast(1.08);
  transform: translate3d(-732px, -50%, 0) rotateY(-22deg);
}

.art-home .time-shift-stack.is-dragging {
  opacity: 0.98;
  filter: saturate(1.22) brightness(1.12) contrast(1.1);
}

.art-home .shift-module-shell {
  position: absolute;
  left: 472px;
  top: 50%;
  z-index: 0;
  width: 470px;
  height: min(108dvh, 1040px);
  min-height: 830px;
  transform: translate3d(-50%, -50%, -260px) rotateY(-18deg);
  transform-origin: left center;
  pointer-events: none;
  opacity: 0.32;
  border-radius: 0 999px 999px 0;
  background:
    radial-gradient(ellipse at 102% 50%, rgba(116, 231, 255, 0.12), transparent 42%),
    radial-gradient(ellipse at 84% 22%, rgba(177, 140, 255, 0.12), transparent 34%),
    linear-gradient(90deg, rgba(68, 74, 86, 0.1), rgba(128, 135, 148, 0.22), rgba(52, 60, 76, 0.16));
  box-shadow:
    inset -28px 0 58px rgba(232, 240, 255, 0.08),
    inset 28px 0 78px rgba(0, 0, 0, 0.36),
    0 0 42px rgba(116, 231, 255, 0.06);
  backdrop-filter: blur(13px);
  clip-path: ellipse(68% 50% at 100% 50%);
  transition:
    opacity 0.3s ease,
    filter 0.3s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .shift-module-shell::before,
.art-home .shift-module-shell::after,
.art-home .shift-module-shell__edge {
  content: "";
  position: absolute;
  pointer-events: none;
  border-radius: inherit;
}

.art-home .shift-module-shell::before {
  inset: 18px -4px 18px 40px;
  border-right: 1px solid rgba(116, 231, 255, 0.42);
  box-shadow:
    18px 0 28px rgba(116, 231, 255, 0.16),
    26px 0 42px rgba(177, 140, 255, 0.12);
  clip-path: ellipse(66% 50% at 100% 50%);
}

.art-home .shift-module-shell::after {
  inset: 52px 12px 52px 96px;
  border-right: 1px solid rgba(255, 99, 125, 0.2);
  opacity: 0.6;
  clip-path: ellipse(62% 50% at 100% 50%);
}

.art-home .shift-module-shell__edge {
  inset: -1px -2px -1px auto;
  width: 126px;
  background:
    radial-gradient(ellipse at 100% 18%, rgba(255, 99, 125, 0.38), transparent 32%),
    radial-gradient(ellipse at 100% 50%, rgba(116, 231, 255, 0.58), transparent 46%),
    radial-gradient(ellipse at 100% 82%, rgba(177, 140, 255, 0.42), transparent 36%);
  filter: blur(0.2px);
  mix-blend-mode: screen;
  opacity: 0.72;
  clip-path: ellipse(76% 50% at 100% 50%);
}

.art-home .time-shift-stack.is-hovering .shift-module-shell {
  opacity: 0.86;
  filter: saturate(1.16) brightness(1.08) contrast(1.06);
  transform: translate3d(-50%, -50%, -260px) rotateY(-10deg);
}

.art-home .time-shift-stack.is-dragging .shift-module-shell,
.art-home .time-shift-stack.is-spinning .shift-module-shell {
  opacity: 0.94;
  filter: saturate(1.24) brightness(1.12) contrast(1.08);
}

.art-home .shift-label {
  position: absolute;
  left: 556px;
  top: 26px;
  z-index: 4;
  display: grid;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid rgba(116, 231, 255, 0.28);
  border-radius: 12px;
  background: rgba(3, 12, 28, 0.78);
  box-shadow:
    0 14px 30px rgba(0, 0, 0, 0.26),
    0 0 22px rgba(116, 231, 255, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  clip-path: polygon(0 10px, 10px 0, 100% 0, 100% calc(100% - 10px), calc(100% - 10px) 100%, 0 100%);
}

.art-home .shift-label span {
  color: rgba(219, 235, 255, 0.86);
  font-size: 10px;
  font-weight: 950;
}

.art-home .shift-label strong {
  color: var(--hud-cyan);
  font-size: 14px;
  font-weight: 950;
  text-shadow: 0 0 18px rgba(116, 231, 255, 0.42);
}

.art-home .shift-window {
  position: absolute;
  inset: 18px 0;
  z-index: 2;
  overflow: visible;
  transform-style: preserve-3d;
  mask-image: none;
  backface-visibility: hidden;
}

.art-home .shift-window::before,
.art-home .shift-window::after {
  content: "";
  position: absolute;
  left: 176px;
  right: 64px;
  z-index: 2;
  height: 1px;
  pointer-events: none;
  background: linear-gradient(90deg, transparent, rgba(116, 231, 255, 0.35), transparent);
}

.art-home .shift-window::before {
  top: 50%;
  box-shadow: 0 0 18px rgba(116, 231, 255, 0.22);
}

.art-home .shift-window::after {
  top: calc(50% + 44px);
  opacity: 0.42;
}

.art-home .shift-track {
  position: absolute;
  inset: 0;
  height: 100%;
  transform-style: preserve-3d;
  pointer-events: none;
  will-change: transform;
  backface-visibility: hidden;
}

.art-home .shift-layer {
  position: absolute;
  left: 72%;
  top: 50%;
  z-index: var(--z);
  width: var(--layer-width);
  height: 40px;
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  opacity: var(--alpha);
  cursor: grab;
  user-select: none;
  border: 1px solid rgba(116, 231, 255, 0.3);
  border-radius: 14px;
  background:
    linear-gradient(90deg, rgba(4, 12, 27, 0.62), rgba(22, 58, 104, 0.86), rgba(18, 18, 55, 0.64)),
    radial-gradient(circle at 18% 50%, rgba(116, 231, 255, 0.24), transparent 38%);
  box-shadow:
    0 16px 34px rgba(0, 0, 0, 0.3),
    0 0 26px rgba(116, 231, 255, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.12);
  transform:
    translate3d(calc(-50% + var(--panel-x)), calc(-50% + var(--panel-y)), var(--depth))
    rotateY(-22deg)
    rotateZ(-2deg)
    scale3d(var(--scale), var(--scale), 1)
    translateZ(0.01px);
  transform-style: preserve-3d;
  transform-origin: center center;
  will-change: transform, opacity;
  backface-visibility: hidden;
  contain: layout paint;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
  backdrop-filter: blur(7px);
  clip-path: polygon(0 12px, 12px 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px));
  transition:
    opacity 0.28s ease,
    border-color 0.28s ease,
    box-shadow 0.28s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .time-shift-stack.is-dragging .shift-layer {
  cursor: grabbing;
}

.art-home .time-shift-stack.is-dragging .shift-layer,
.art-home .time-shift-stack.is-spinning .shift-layer {
  transition:
    opacity 0.14s linear,
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.art-home .shift-layer::before {
  content: "";
  position: absolute;
  inset: -1px;
  border-radius: inherit;
  background: linear-gradient(90deg, transparent, rgba(116, 231, 255, 0.28), transparent);
  opacity: 0;
  transition: opacity 0.18s ease;
  pointer-events: none;
  backface-visibility: hidden;
}

.art-home .time-shift-stack.is-hovering .shift-layer::before {
  opacity: 0.24;
}

.art-home .time-shift-stack.is-hovering .shift-layer {
  border-color: rgba(116, 231, 255, 0.48);
  box-shadow:
    0 18px 38px rgba(0, 0, 0, 0.34),
    0 0 32px rgba(116, 231, 255, 0.22),
    inset 0 1px 0 rgba(255, 255, 255, 0.16);
}

.art-home .shift-layer__line {
  height: 3px;
  border-radius: 999px;
  background:
    linear-gradient(90deg, transparent, rgba(116, 231, 255, 0.92), rgba(177, 140, 255, 0.64), transparent);
  box-shadow: 0 0 18px rgba(116, 231, 255, 0.44);
  transform: translateZ(0.02px);
  backface-visibility: hidden;
}

.art-home .shift-layer__meta {
  color: rgba(244, 250, 255, 0.96);
  font-size: 11px;
  font-weight: 950;
  letter-spacing: 0;
  white-space: nowrap;
  text-shadow: 0 0 12px rgba(116, 231, 255, 0.34);
  transform: translateZ(0.02px);
  backface-visibility: hidden;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
}

.art-home .time-shift-stack.is-hovering .shift-layer__line {
  background:
    linear-gradient(90deg, transparent, rgba(255, 99, 125, 0.46), rgba(116, 231, 255, 0.9), rgba(177, 140, 255, 0.68), transparent);
  box-shadow: 0 0 20px rgba(116, 231, 255, 0.42);
}

.art-home .gallery-orbit-field {
  position: fixed;
  right: 0;
  top: 0;
  z-index: 7;
  width: min(18vw, 156px);
  height: 100dvh;
  pointer-events: auto;
  background: transparent;
  cursor: grab;
  touch-action: none;
  transition: width 0.28s ease;
}

.art-home .gallery-orbit-field.is-hovering {
  width: min(30vw, 268px);
}

.art-home .gallery-orbit-field.is-dragging {
  cursor: grabbing;
}

.art-home .gallery-art-stack {
  position: fixed;
  right: 0;
  top: 52dvh;
  z-index: 8;
  width: 760px;
  height: min(96dvh, 940px);
  min-height: 760px;
  transform: translate3d(760px, -50%, 0) rotateY(32deg);
  transform-origin: right center;
  transform-style: preserve-3d;
  perspective: 1400px;
  pointer-events: none;
  opacity: 0.52;
  filter: saturate(0.86) brightness(0.92);
  isolation: isolate;
  will-change: transform, opacity, filter;
  backface-visibility: hidden;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
  transition:
    opacity 0.28s ease,
    filter 0.28s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .gallery-art-stack.is-hovering {
  opacity: 0.98;
  filter: saturate(1.16) brightness(1.08) contrast(1.08);
  transform: translate3d(604px, -50%, 0) rotateY(14deg);
}

.art-home .gallery-art-stack.is-dragging {
  opacity: 1;
  filter: saturate(1.22) brightness(1.12) contrast(1.1);
}

.art-home .gallery-module-shell {
  position: absolute;
  right: 472px;
  top: 50%;
  z-index: 0;
  width: 470px;
  height: min(108dvh, 1040px);
  min-height: 830px;
  transform: translate3d(50%, -50%, -260px) rotateY(18deg);
  transform-origin: right center;
  pointer-events: none;
  opacity: 0.32;
  border-radius: 999px 0 0 999px;
  background:
    radial-gradient(ellipse at -2% 50%, rgba(255, 99, 125, 0.1), transparent 42%),
    radial-gradient(ellipse at 16% 22%, rgba(116, 231, 255, 0.13), transparent 34%),
    linear-gradient(270deg, rgba(68, 74, 86, 0.1), rgba(128, 135, 148, 0.22), rgba(52, 60, 76, 0.16));
  box-shadow:
    inset 28px 0 58px rgba(232, 240, 255, 0.08),
    inset -28px 0 78px rgba(0, 0, 0, 0.36),
    0 0 42px rgba(255, 99, 125, 0.06);
  backdrop-filter: blur(13px);
  clip-path: ellipse(68% 50% at 0% 50%);
  transition:
    opacity 0.3s ease,
    filter 0.3s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .gallery-module-shell::before,
.art-home .gallery-module-shell::after,
.art-home .gallery-module-shell__edge {
  content: "";
  position: absolute;
  pointer-events: none;
  border-radius: inherit;
}

.art-home .gallery-module-shell::before {
  inset: 18px 40px 18px -4px;
  border-left: 1px solid rgba(255, 99, 125, 0.38);
  box-shadow:
    -18px 0 28px rgba(255, 99, 125, 0.16),
    -26px 0 42px rgba(116, 231, 255, 0.12);
  clip-path: ellipse(66% 50% at 0% 50%);
}

.art-home .gallery-module-shell::after {
  inset: 52px 96px 52px 12px;
  border-left: 1px solid rgba(116, 231, 255, 0.22);
  opacity: 0.62;
  clip-path: ellipse(62% 50% at 0% 50%);
}

.art-home .gallery-module-shell__edge {
  inset: -1px auto -1px -2px;
  width: 126px;
  background:
    radial-gradient(ellipse at 0% 18%, rgba(116, 231, 255, 0.42), transparent 32%),
    radial-gradient(ellipse at 0% 50%, rgba(255, 99, 125, 0.58), transparent 46%),
    radial-gradient(ellipse at 0% 82%, rgba(177, 140, 255, 0.42), transparent 36%);
  filter: blur(0.2px);
  mix-blend-mode: screen;
  opacity: 0.72;
  clip-path: ellipse(76% 50% at 0% 50%);
}

.art-home .gallery-art-stack.is-hovering .gallery-module-shell {
  opacity: 0.86;
  filter: saturate(1.16) brightness(1.08) contrast(1.06);
  transform: translate3d(50%, -50%, -260px) rotateY(6deg);
}

.art-home .gallery-art-stack.is-dragging .gallery-module-shell,
.art-home .gallery-art-stack.is-spinning .gallery-module-shell {
  opacity: 0.94;
  filter: saturate(1.24) brightness(1.12) contrast(1.08);
}

.art-home .gallery-orbit-label {
  position: absolute;
  right: 556px;
  top: 26px;
  z-index: 4;
  display: grid;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid rgba(255, 99, 125, 0.28);
  border-radius: 12px;
  background: rgba(3, 12, 28, 0.78);
  box-shadow:
    0 14px 30px rgba(0, 0, 0, 0.26),
    0 0 22px rgba(255, 99, 125, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  clip-path: polygon(0 0, calc(100% - 10px) 0, 100% 10px, 100% 100%, 10px 100%, 0 calc(100% - 10px));
}

.art-home .gallery-orbit-label span {
  color: rgba(219, 235, 255, 0.86);
  font-size: 10px;
  font-weight: 950;
}

.art-home .gallery-orbit-label strong {
  color: var(--hud-red);
  font-size: 14px;
  font-weight: 950;
  text-shadow: 0 0 18px rgba(255, 99, 125, 0.42);
}

.art-home .gallery-orbit-window {
  position: absolute;
  inset: 18px 0;
  z-index: 2;
  overflow: visible;
  transform-style: preserve-3d;
  mask-image: none;
  backface-visibility: hidden;
}

.art-home .gallery-orbit-window::before,
.art-home .gallery-orbit-window::after {
  content: "";
  position: absolute;
  left: 64px;
  right: 176px;
  z-index: 2;
  height: 1px;
  pointer-events: none;
  background: linear-gradient(90deg, transparent, rgba(255, 99, 125, 0.35), transparent);
}

.art-home .gallery-orbit-window::before {
  top: 50%;
  box-shadow: 0 0 18px rgba(255, 99, 125, 0.22);
}

.art-home .gallery-orbit-window::after {
  top: calc(50% + 44px);
  opacity: 0.42;
}

.art-home .gallery-orbit-track {
  position: absolute;
  inset: 0;
  height: 100%;
  transform-style: preserve-3d;
  pointer-events: none;
  will-change: transform;
  backface-visibility: hidden;
}

.art-home .gallery-orbit-layer {
  position: absolute;
  left: 28%;
  top: 50%;
  z-index: var(--z);
  width: var(--layer-width);
  height: var(--layer-height);
  display: block;
  overflow: hidden;
  padding: 0;
  opacity: var(--alpha);
  cursor: pointer;
  user-select: none;
  appearance: none;
  font: inherit;
  text-decoration: none;
  border: 1px solid rgba(255, 99, 125, 0.34);
  border-radius: 16px;
  background:
    radial-gradient(circle at 50% 42%, rgba(116, 231, 255, 0.1), transparent 56%),
    rgba(4, 12, 27, 0.76);
  box-shadow:
    0 16px 34px rgba(0, 0, 0, 0.34),
    0 0 26px rgba(255, 99, 125, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.12);
  transform:
    translate3d(calc(-50% - var(--panel-x)), calc(-50% + var(--panel-y)), var(--depth))
    rotateY(22deg)
    rotateZ(2deg)
    scale3d(var(--scale), var(--scale), 1)
    translateZ(0.01px);
  transform-style: preserve-3d;
  transform-origin: center center;
  will-change: transform, opacity;
  backface-visibility: hidden;
  contain: layout paint;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
  backdrop-filter: blur(7px);
  clip-path: polygon(0 8px, 8px 0, calc(100% - 12px) 0, 100% 12px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 12px 100%, 0 calc(100% - 12px));
  transition:
    opacity 0.28s ease,
    border-color 0.28s ease,
    box-shadow 0.28s ease,
    filter 0.28s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.art-home .gallery-art-stack.is-dragging .gallery-orbit-layer {
  cursor: grabbing;
}

.art-home .gallery-art-stack.is-dragging .gallery-orbit-layer,
.art-home .gallery-art-stack.is-spinning .gallery-orbit-layer {
  transition:
    opacity 0.14s linear,
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    filter 0.18s ease;
}

.art-home .gallery-orbit-layer img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: contain;
  transform: translateZ(0.02px) scale(0.92);
  backface-visibility: hidden;
  -webkit-font-smoothing: antialiased;
  filter: saturate(1.08) contrast(1.04) brightness(1);
  pointer-events: none;
}

.art-home .gallery-orbit-layer__title {
  position: absolute;
  pointer-events: none;
  transform: translateZ(0.03px);
  backface-visibility: hidden;
}

.art-home .gallery-orbit-layer__title {
  left: 12px;
  right: 10px;
  bottom: 9px;
  overflow: hidden;
  color: rgba(246, 250, 255, 0.98);
  font-size: 12px;
  font-weight: 950;
  line-height: 1.15;
  letter-spacing: 0;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-shadow:
    0 0 14px rgba(255, 99, 125, 0.52),
    0 2px 8px rgba(0, 0, 0, 0.62);
}

.art-home .gallery-art-stack.is-hovering .gallery-orbit-layer {
  border-color: rgba(255, 99, 125, 0.58);
  box-shadow:
    0 18px 38px rgba(0, 0, 0, 0.36),
    0 0 30px rgba(255, 99, 125, 0.22),
    0 0 20px rgba(116, 231, 255, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.16);
}

.art-home .gallery-orbit-layer:hover {
  filter: saturate(1.18) brightness(1.1) contrast(1.06);
  border-color: rgba(116, 231, 255, 0.62);
  box-shadow:
    0 20px 44px rgba(0, 0, 0, 0.4),
    0 0 34px rgba(116, 231, 255, 0.24),
    0 0 24px rgba(255, 99, 125, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.18);
}

.art-home .gallery-orbit-layer:hover img {
  filter: saturate(1.18) contrast(1.08) brightness(1.03);
}

.art-home .time-device {
  position: relative;
  width: 560px;
  height: 560px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  isolation: isolate;
}

.art-home .time-device::before,
.art-home .time-device::after {
  content: "";
  position: absolute;
  border-radius: 50%;
  pointer-events: none;
}

.art-home .time-device::before {
  inset: 0;
  background:
    radial-gradient(circle, transparent 0 42%, rgba(116, 231, 255, 0.07) 43% 45%, transparent 46%),
    radial-gradient(circle, transparent 0 61%, rgba(177, 140, 255, 0.08) 62% 64%, transparent 65%),
    conic-gradient(
      from var(--loop-offset),
      transparent 0deg,
      rgba(116, 231, 255, 0.06) 22deg,
      rgba(116, 231, 255, 0.32) 42deg,
      rgba(177, 140, 255, 0.26) 58deg,
      transparent 84deg,
      transparent 142deg,
      rgba(255, 99, 125, 0.16) 168deg,
      rgba(116, 231, 255, 0.22) 196deg,
      transparent 228deg,
      transparent 278deg,
      rgba(177, 140, 255, 0.2) 314deg,
      transparent 360deg
    );
  box-shadow: 0 0 104px rgba(116, 231, 255, 0.16), inset 0 0 56px rgba(177, 140, 255, 0.08);
}

.art-home .time-device::after {
  inset: 54px;
  border: 1px solid rgba(116, 231, 255, 0.16);
  background:
    conic-gradient(from calc(var(--loop-offset) + 90deg), transparent 0 28deg, rgba(116, 231, 255, 0.1) 44deg, transparent 74deg, transparent 180deg, rgba(177, 140, 255, 0.1) 212deg, transparent 250deg),
    radial-gradient(circle, transparent 0 56%, rgba(116, 231, 255, 0.05) 58% 60%, transparent 62%);
  box-shadow: inset 0 0 34px rgba(116, 231, 255, 0.08), 0 0 44px rgba(116, 231, 255, 0.08);
}

.art-home .orbit,
.art-home .ratio-orbit,
.art-home .tick-ring,
.art-home .scan-sweep {
  position: absolute;
  border-radius: 50%;
}

.art-home .orbit-outer {
  inset: 8px;
  border: 1px solid rgba(116, 231, 255, 0.18);
  box-shadow: 0 0 34px rgba(116, 231, 255, 0.1), inset 0 0 28px rgba(116, 231, 255, 0.04);
  animation: homeOrbit 30s linear infinite;
}

.art-home .orbit-middle {
  inset: 62px;
  border: 1px solid rgba(177, 140, 255, 0.18);
  background:
    conic-gradient(from calc(var(--loop-offset) + 18deg), transparent 0 38deg, rgba(177, 140, 255, 0.16) 52deg, transparent 78deg, transparent 190deg, rgba(116, 231, 255, 0.12) 220deg, transparent 252deg);
  mask-image: radial-gradient(circle, transparent 0 49%, black 50% 51%, transparent 52%);
  animation: homeOrbitReverse 22s linear infinite;
}

.art-home .orbit-inner {
  inset: 116px;
  border: 1px solid rgba(116, 231, 255, 0.18);
  box-shadow: inset 0 0 34px rgba(116, 231, 255, 0.12);
  animation: homeOrbit 18s linear infinite;
}

.art-home .ratio-orbit {
  inset: 28px;
  background:
    conic-gradient(
      from -90deg,
      rgba(116, 231, 255, 0.96) 0 var(--haruhi-angle),
      rgba(141, 240, 255, 0.48) var(--haruhi-angle),
      rgba(177, 140, 255, 0.28) calc(var(--haruhi-angle) + 7deg) 360deg
    );
  mask-image: radial-gradient(circle, transparent 0 40%, black 41% 48%, transparent 49%);
  box-shadow: 0 0 66px rgba(116, 231, 255, 0.22), 0 0 28px rgba(177, 140, 255, 0.12);
}

.art-home .tick-ring {
  inset: 86px;
  background:
    conic-gradient(
      from calc(var(--loop-offset) + 4deg),
      transparent 0deg,
      rgba(116, 231, 255, 0.08) 28deg,
      rgba(116, 231, 255, 0.22) 44deg,
      transparent 74deg,
      transparent 126deg,
      rgba(177, 140, 255, 0.18) 156deg,
      transparent 188deg,
      transparent 244deg,
      rgba(255, 99, 125, 0.12) 274deg,
      rgba(116, 231, 255, 0.16) 304deg,
      transparent 340deg
    ),
    repeating-conic-gradient(from 0deg, rgba(116, 231, 255, 0.16) 0 0.45deg, transparent 0.45deg 18deg);
  mask-image: radial-gradient(circle, transparent 0 47%, black 48% 50%, transparent 51%);
  opacity: 0.88;
}

.art-home .scan-sweep {
  inset: 0;
  background:
    conic-gradient(from 0deg, transparent 0 268deg, rgba(116, 231, 255, 0.06) 292deg, rgba(116, 231, 255, 0.2) 318deg, rgba(177, 140, 255, 0.08) 334deg, transparent 356deg);
  animation: homeOrbit 11s linear infinite;
  mix-blend-mode: screen;
}

.art-home .observer-core {
  position: relative;
  z-index: 2;
  width: 268px;
  min-height: 268px;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 14px;
  padding: 28px;
  border: 1px solid rgba(116, 231, 255, 0.28);
  border-radius: 50%;
  background:
    radial-gradient(circle at 50% 18%, rgba(93, 140, 255, 0.28), rgba(5, 13, 28, 0.9)),
    linear-gradient(135deg, rgba(8, 18, 38, 0.94), rgba(16, 18, 42, 0.88));
  color: var(--hud-text);
  text-align: center;
  box-shadow:
    0 24px 72px rgba(0, 0, 0, 0.38),
    0 0 42px rgba(116, 231, 255, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.09);
}

.art-home .observer-core::before {
  content: "";
  position: absolute;
  inset: 16px;
  border-radius: 50%;
  border: 1px solid rgba(255, 99, 125, 0.18);
  pointer-events: none;
}

.art-home .loop-stamp {
  display: inline-flex;
  padding: 6px 12px;
  border: 1px solid rgba(116, 231, 255, 0.28);
  border-radius: 999px;
  background: rgba(10, 28, 58, 0.74);
  color: var(--hud-cyan);
  font-size: 12px;
  font-weight: 950;
  box-shadow: 0 0 18px rgba(116, 231, 255, 0.1);
}

.art-home .observer-core p {
  color: var(--hud-text);
  font-size: 17px;
  line-height: 1.55;
  font-weight: 900;
}

.art-home .observer-core p strong {
  display: block;
  color: var(--hud-cyan);
  font-size: 40px;
  line-height: 1.05;
  font-weight: 950;
  text-shadow: 0 0 24px rgba(116, 231, 255, 0.42);
}

.art-home .core-split {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.art-home .core-split span {
  padding: 5px 9px;
  border: 1px solid rgba(116, 231, 255, 0.16);
  border-radius: 999px;
  background: rgba(9, 24, 50, 0.72);
  color: var(--hud-muted);
  font-size: 11px;
  font-weight: 950;
}

.art-home .ratio-note {
  position: absolute;
  z-index: 3;
  display: grid;
  place-items: center;
  min-width: 92px;
  padding: 9px 12px;
  border: 1px solid var(--sector-line);
  border-radius: 999px;
  background:
    radial-gradient(circle at 50% 0%, var(--sector-glow), transparent 62%),
    rgba(4, 13, 29, 0.68);
  box-shadow:
    0 16px 34px rgba(0, 0, 0, 0.28),
    0 0 28px var(--sector-glow),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  transform: translate(-50%, -50%);
  pointer-events: none;
}

.art-home .ratio-note strong {
  color: var(--sector-color);
  font-size: 26px;
  line-height: 1;
  font-weight: 950;
  text-shadow: 0 0 18px var(--sector-glow);
}

.art-home .ratio-note span {
  margin-top: 4px;
  color: rgba(230, 241, 255, 0.78);
  font-size: 11px;
  font-weight: 900;
}

.art-home .note-haruhi {
  --sector-color: var(--hud-cyan);
  --sector-line: rgba(116, 231, 255, 0.46);
  --sector-glow: rgba(116, 231, 255, 0.26);
  top: var(--haruhi-label-y);
  left: var(--haruhi-label-x);
}

.art-home .note-other {
  --sector-color: var(--hud-violet);
  --sector-line: rgba(177, 140, 255, 0.42);
  --sector-glow: rgba(177, 140, 255, 0.24);
  top: var(--other-label-y);
  left: var(--other-label-x);
}

.art-home .satellite-node {
  position: absolute;
  z-index: 4;
  width: 172px;
  min-height: 108px;
  display: grid;
  align-content: center;
  gap: 6px;
  padding: 16px;
  border: 1px solid var(--hud-line);
  border-radius: 12px;
  background:
    linear-gradient(135deg, rgba(10, 28, 58, 0.82), rgba(12, 18, 44, 0.7)),
    radial-gradient(circle at top right, rgba(116, 231, 255, 0.12), transparent 48%);
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.28), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(14px);
  clip-path: polygon(0 12px, 12px 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px));
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
}

.art-home .satellite-node:hover {
  border-color: var(--hud-line-strong);
  box-shadow: 0 20px 52px rgba(0, 0, 0, 0.32), 0 0 24px rgba(116, 231, 255, 0.18);
  transform: translateY(-2px);
}

.art-home .satellite-node::before {
  content: "";
  position: absolute;
  top: 10px;
  left: 12px;
  width: 32px;
  height: 2px;
  background: linear-gradient(90deg, var(--hud-cyan), transparent);
  pointer-events: none;
}

.art-home .satellite-node span {
  color: var(--hud-muted);
  font-size: 12px;
  font-weight: 950;
}

.art-home .satellite-node strong {
  color: var(--hud-text);
  font-size: 30px;
  line-height: 1;
  font-weight: 950;
  text-shadow: 0 0 20px rgba(116, 231, 255, 0.16);
}

.art-home .satellite-node small {
  color: rgba(197, 216, 244, 0.66);
  font-size: 11px;
  line-height: 1.4;
  font-weight: 850;
}

.art-home .node-artworks {
  top: 120px;
  left: 7%;
}

.art-home .node-creators {
  top: 120px;
  right: 7%;
}

.art-home .node-latest {
  left: 9%;
  bottom: 92px;
}

.art-home .node-likes {
  right: 10%;
  bottom: 86px;
}

.art-home .node-week {
  right: 3%;
  top: 50%;
  transform: translateY(-50%);
}

.art-home .node-week:hover {
  transform: translateY(calc(-50% - 2px));
}

.art-home .bottom-grid {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
  margin-top: 18px;
}

.art-home .endless-panel {
  position: relative;
  overflow: hidden;
  border: 1px solid var(--hud-line);
  border-radius: 14px;
  background:
    linear-gradient(135deg, rgba(8, 18, 38, 0.8), rgba(13, 17, 44, 0.72)),
    radial-gradient(circle at top right, rgba(116, 231, 255, 0.1), transparent 42%);
  box-shadow: 0 18px 52px rgba(0, 0, 0, 0.26), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  color: var(--hud-text);
  clip-path: polygon(0 16px, 16px 0, 100% 0, 100% calc(100% - 16px), calc(100% - 16px) 100%, 0 100%);
}

.art-home .endless-panel::before {
  content: "";
  position: absolute;
  inset: 0;
  background:
    linear-gradient(rgba(116, 231, 255, 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(116, 231, 255, 0.04) 1px, transparent 1px);
  background-size: 28px 28px;
  pointer-events: none;
}

.art-home .panel-head {
  position: relative;
  padding: 22px 24px 0;
}

.art-home .panel-head h2 {
  color: var(--hud-text);
  font-size: 22px;
  font-weight: 950;
}

.art-home .record-list,
.art-home .tag-radar {
  position: relative;
  display: grid;
  gap: 12px;
  padding: 20px 24px 24px;
}

.art-home .record-row {
  display: grid;
  grid-template-columns: 72px 1fr auto;
  align-items: center;
  gap: 14px;
  padding: 12px;
  border: 1px solid rgba(116, 231, 255, 0.14);
  border-radius: 10px;
  background: rgba(6, 16, 34, 0.56);
  color: inherit;
  text-decoration: none;
  transition: border-color 0.18s ease, background 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
}

.art-home .record-row:hover,
.art-home .record-row:focus-visible {
  border-color: rgba(116, 231, 255, 0.36);
  background: rgba(11, 28, 58, 0.7);
  box-shadow: 0 0 22px rgba(116, 231, 255, 0.08);
  transform: translateY(-1px);
  outline: none;
}

.art-home .record-code {
  color: var(--hud-cyan);
  font-size: 12px;
  font-weight: 950;
}

.art-home .record-row strong {
  display: block;
  color: var(--hud-text);
  font-size: 14px;
  font-weight: 950;
}

.art-home .record-row small {
  display: block;
  margin-top: 4px;
  color: var(--hud-muted);
  font-size: 12px;
  font-weight: 850;
}

.art-home .record-time {
  color: rgba(197, 216, 244, 0.76);
  font-size: 12px;
  font-weight: 900;
}

.art-home .tag-row {
  display: grid;
  gap: 8px;
}

.art-home .tag-label {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  color: var(--hud-muted);
  font-size: 14px;
  font-weight: 900;
}

.art-home .tag-label strong {
  color: var(--hud-text);
  font-weight: 950;
}

.art-home .tag-track {
  height: 12px;
  overflow: hidden;
  border-radius: 999px;
  background: rgba(4, 10, 23, 0.72);
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.34);
}

.art-home .tag-track span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--hud-cyan), var(--hud-blue), var(--hud-violet));
  box-shadow: 0 0 20px rgba(116, 231, 255, 0.22);
}

:global(html.art-lights-out) .art-home {
  --space-bg: #01040d;
  --space-bg-2: #050d1d;
  --space-bg-3: #10071f;
  --hud-panel: rgba(3, 9, 22, 0.78);
  --hud-panel-strong: rgba(5, 14, 32, 0.94);
  --hud-line: rgba(126, 227, 255, 0.28);
  --hud-line-strong: rgba(141, 236, 255, 0.68);
  --hud-text: rgba(247, 251, 255, 0.98);
  --hud-muted: rgba(196, 218, 245, 0.76);
  --hud-cyan: #8df0ff;
  --hud-blue: #6d94ff;
  --hud-violet: #c7a6ff;
  --hud-red: #ff6f89;
}

:global(html.art-lights-out) .art-home .endless-screen {
  box-shadow:
    0 36px 130px rgba(0, 0, 0, 0.58),
    0 0 74px rgba(116, 231, 255, 0.16),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

:global(html.art-lights-out) .art-home .star-dust {
  opacity: 0.78;
}

:global(html.art-lights-out) .art-home .star-layer-a {
  opacity: 0.9;
}

:global(html.art-lights-out) .art-home .star-layer-b {
  opacity: 0.66;
}

:global(html.art-lights-out) .art-home .star-layer-c {
  opacity: 0.76;
}

:global(html.art-lights-out) .art-home .bright-stars {
  opacity: 0.86;
}

:global(html.art-lights-out) .art-home .galaxy-halo {
  opacity: 0.66;
}

:global(html.art-lights-out) .art-home .nebula-a {
  background:
    radial-gradient(ellipse at 42% 42%, rgba(105, 80, 255, 0.34), transparent 46%),
    radial-gradient(ellipse at 68% 34%, rgba(85, 185, 255, 0.18), transparent 38%),
    radial-gradient(ellipse at 34% 68%, rgba(255, 99, 125, 0.1), transparent 38%);
}

:global(html.art-lights-out) .art-home .nebula-b {
  background:
    radial-gradient(ellipse at 46% 48%, rgba(0, 216, 255, 0.24), transparent 48%),
    radial-gradient(ellipse at 62% 62%, rgba(177, 140, 255, 0.16), transparent 40%);
}

:global(html.art-lights-out) .art-home .time-device::before {
  box-shadow: 0 0 118px rgba(141, 240, 255, 0.22), inset 0 0 64px rgba(177, 140, 255, 0.1);
}

:global(html.art-lights-out) .art-home .time-shift-stack {
  opacity: 0.28;
  filter: saturate(0.7) brightness(0.82);
}

:global(html.art-lights-out) .art-home .time-shift-stack.is-hovering {
  opacity: 0.98;
  filter: saturate(1.22) brightness(1.12) contrast(1.1);
}

:global(html.art-lights-out) .art-home .time-shift-stack.is-dragging {
  opacity: 1;
  filter: saturate(1.3) brightness(1.16) contrast(1.12);
}

:global(html.art-lights-out) .art-home .gallery-art-stack {
  opacity: 0.54;
  filter: saturate(0.9) brightness(0.92);
}

:global(html.art-lights-out) .art-home .gallery-art-stack.is-hovering {
  opacity: 0.98;
  filter: saturate(1.2) brightness(1.1) contrast(1.1);
}

:global(html.art-lights-out) .art-home .gallery-art-stack.is-dragging {
  opacity: 1;
  filter: saturate(1.28) brightness(1.14) contrast(1.12);
}

:global(html.art-lights-out) .art-home .shift-module-shell {
  background:
    radial-gradient(ellipse at 102% 50%, rgba(141, 240, 255, 0.16), transparent 42%),
    radial-gradient(ellipse at 84% 22%, rgba(199, 166, 255, 0.16), transparent 34%),
    linear-gradient(90deg, rgba(38, 46, 64, 0.08), rgba(112, 124, 144, 0.18), rgba(14, 24, 42, 0.28));
  box-shadow:
    inset -30px 0 64px rgba(235, 246, 255, 0.09),
    inset 32px 0 88px rgba(0, 0, 0, 0.48),
    0 0 56px rgba(141, 240, 255, 0.08);
}

:global(html.art-lights-out) .art-home .shift-module-shell::before {
  border-right-color: rgba(141, 240, 255, 0.58);
  box-shadow:
    20px 0 34px rgba(141, 240, 255, 0.2),
    30px 0 54px rgba(199, 166, 255, 0.16);
}

:global(html.art-lights-out) .art-home .shift-module-shell__edge {
  opacity: 0.84;
}

:global(html.art-lights-out) .art-home .gallery-module-shell {
  background:
    radial-gradient(ellipse at -2% 50%, rgba(255, 99, 125, 0.16), transparent 42%),
    radial-gradient(ellipse at 16% 22%, rgba(141, 240, 255, 0.16), transparent 34%),
    linear-gradient(270deg, rgba(38, 46, 64, 0.08), rgba(112, 124, 144, 0.18), rgba(14, 24, 42, 0.28));
  box-shadow:
    inset 30px 0 64px rgba(235, 246, 255, 0.09),
    inset -32px 0 88px rgba(0, 0, 0, 0.48),
    0 0 56px rgba(255, 99, 125, 0.08);
}

:global(html.art-lights-out) .art-home .gallery-module-shell::before {
  border-left-color: rgba(255, 99, 125, 0.52);
  box-shadow:
    -20px 0 34px rgba(255, 99, 125, 0.2),
    -30px 0 54px rgba(141, 240, 255, 0.14);
}

:global(html.art-lights-out) .art-home .gallery-module-shell__edge {
  opacity: 0.84;
}

:global(html.art-lights-out) .art-home .shift-layer {
  border-color: rgba(141, 240, 255, 0.38);
  background:
    linear-gradient(90deg, rgba(3, 10, 24, 0.68), rgba(24, 65, 112, 0.9), rgba(18, 14, 54, 0.68)),
    radial-gradient(circle at 18% 50%, rgba(141, 240, 255, 0.28), transparent 38%);
  box-shadow:
    0 16px 34px rgba(0, 0, 0, 0.36),
    0 0 30px rgba(141, 240, 255, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.14);
}

:global(html.art-lights-out) .art-home .gallery-orbit-layer {
  border-color: rgba(255, 99, 125, 0.44);
  background: rgba(3, 10, 24, 0.78);
  box-shadow:
    0 16px 34px rgba(0, 0, 0, 0.4),
    0 0 30px rgba(255, 99, 125, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.14);
}

:global(html.art-lights-out) .art-home .shift-label {
  border-color: rgba(141, 240, 255, 0.32);
  background: rgba(3, 10, 24, 0.84);
}

:global(html.art-lights-out) .art-home .gallery-orbit-label {
  border-color: rgba(255, 99, 125, 0.34);
  background: rgba(3, 10, 24, 0.84);
}

:global(html.art-lights-out) .art-home .observer-core,
:global(html.art-lights-out) .art-home .satellite-node,
:global(html.art-lights-out) .art-home .endless-panel,
:global(html.art-lights-out) .art-home .ratio-note,
:global(html.art-lights-out) .art-home .summer-strip,
:global(html.art-lights-out) .art-home .status-chip {
  background-color: rgba(3, 10, 24, 0.78);
}

:global(html.art-lights-out) .art-home .scan-sweep {
  background:
    conic-gradient(from 0deg, transparent 0 266deg, rgba(141, 240, 255, 0.08) 292deg, rgba(141, 240, 255, 0.28) 318deg, rgba(199, 166, 255, 0.12) 334deg, transparent 356deg);
}

@keyframes homeStarDrift {
  to {
    background-position: 180px 96px, 260px 174px, 340px 220px;
  }
}

@keyframes homeStarFloat {
  0% {
    transform: translate3d(0, 0, 0);
  }

  100% {
    transform: translate3d(-32px, 18px, 0);
  }
}

@keyframes homeNebulaDrift {
  0% {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.62;
  }

  100% {
    transform: translate3d(28px, -18px, 0) scale(1.04);
    opacity: 0.78;
  }
}

@keyframes homeGalaxyDrift {
  0% {
    transform: translate3d(0, 0, 0) rotate(-14deg);
  }

  100% {
    transform: translate3d(18px, -10px, 0) rotate(-11deg);
  }
}

@keyframes homeGalaxyDriftB {
  0% {
    transform: translate3d(0, 0, 0) rotate(18deg);
  }

  100% {
    transform: translate3d(-14px, 12px, 0) rotate(21deg);
  }
}

@keyframes homeOrbit {
  to {
    transform: rotate(360deg);
  }
}

@keyframes homeOrbitReverse {
  to {
    transform: rotate(-360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .art-home .star-dust,
  .art-home .star-layer,
  .art-home .nebula,
  .art-home .galaxy-halo,
  .art-home .orbit,
  .art-home .scan-sweep {
    animation: none !important;
  }

  .art-home .time-shift-stack,
  .art-home .shift-layer,
  .art-home .shift-layer::before,
  .art-home .gallery-art-stack,
  .art-home .gallery-orbit-layer {
    transition: none !important;
  }
}

@media (max-width: 1180px) {
  .art-home .endless-screen {
    min-height: auto;
  }

  .art-home .visual-stage {
    min-height: auto;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: center;
    padding-top: 88px;
  }

  .art-home .time-device {
    grid-column: 1 / -1;
    justify-self: center;
    width: 500px;
    height: 500px;
  }

  .art-home .satellite-node {
    position: relative;
    inset: auto;
    transform: none;
    width: 100%;
  }

  .art-home .satellite-node:hover,
  .art-home .node-week:hover {
    transform: translateY(-2px);
  }

  .art-home .summer-strip {
    top: 24px;
  }

  .art-home .time-shift-stack,
  .art-home .shift-hover-field,
  .art-home .gallery-art-stack,
  .art-home .gallery-orbit-field {
    display: none;
  }
}

@media (max-width: 820px) {
  .art-home {
    width: min(100% - 20px, 1500px);
    padding: 0;
  }

  .art-home .home-lights-on-ui {
    min-height: calc(100dvh - 118px);
    gap: 18px;
    padding: 24px 16px;
    border-radius: 20px;
  }

  .art-home .day-gallery-wall {
    inset: -6% -34%;
    opacity: 0.82;
    transform: rotate(-15deg) scale(0.92);
  }

  .art-home .wall-tile {
    width: min(var(--tile-w), 130px);
    border-radius: 14px;
  }

  .art-home .day-hero-panel {
    padding: 24px 18px;
    border-radius: 20px;
  }

  .art-home .day-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .art-home .day-curation-strip {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .art-home::before {
    inset: -42px -28px -34px;
  }

  .art-home .star-dust {
    opacity: 0.34;
    background-size: 118px 104px, 168px 142px, 220px 188px;
  }

  .art-home .star-layer-a {
    opacity: 0.48;
  }

  .art-home .star-layer-b,
  .art-home .star-layer-c {
    opacity: 0.28;
  }

  .art-home .bright-stars {
    opacity: 0.46;
  }

  .art-home .nebula {
    opacity: 0.46;
    filter: blur(28px);
  }

  .art-home .galaxy-halo {
    display: none;
  }

  .art-home .screen-header {
    display: grid;
    padding: 22px 20px 0;
  }

  .art-home h1 {
    font-size: 28px;
  }

  .art-home .visual-stage {
    grid-template-columns: 1fr;
    padding: 86px 18px 24px;
  }

  .art-home .time-device {
    width: 320px;
    height: 320px;
  }

  .art-home .observer-core {
    width: 184px;
    min-height: 184px;
    padding: 18px;
    gap: 8px;
  }

  .art-home .observer-core p {
    font-size: 14px;
  }

  .art-home .observer-core p strong {
    font-size: 28px;
  }

  .art-home .loop-stamp {
    font-size: 11px;
  }

  .art-home .ratio-note {
    min-width: 74px;
    padding: 7px 9px;
    border-radius: 999px;
  }

  .art-home .ratio-note strong {
    font-size: 22px;
  }

  .art-home .ratio-note span {
    font-size: 11px;
  }

  .art-home .note-haruhi {
    top: var(--haruhi-label-y);
    left: var(--haruhi-label-x);
  }

  .art-home .note-other {
    top: var(--other-label-y);
    left: var(--other-label-x);
  }

  .art-home .bottom-grid {
    grid-template-columns: 1fr;
    gap: 14px;
  }

  .art-home .record-row {
    grid-template-columns: 1fr;
    gap: 6px;
  }
}
</style>
