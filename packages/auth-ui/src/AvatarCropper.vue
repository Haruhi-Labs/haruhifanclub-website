<script setup>
// 头像裁切器：对齐成熟产品（拖拽平移 + 滚轮/滑块/双指缩放 + 圆形取景框）。
// 取景框为方形舞台内切的圆——导出整块方形画布（头像各处以圆形展示，方形存储即可），
// 交互上用圆环遮罩提示最终圆形效果。导出在前端裁好，服务端仍兜底统一尺寸与格式。
import { ref, reactive, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { SosModal, SosButton } from '@haruhi/ui'

const props = defineProps({
  open: { type: Boolean, default: false },
  // 待裁切图片地址（通常是所选文件的 ObjectURL / DataURL），由父级提供
  src: { type: String, default: '' },
  // 导出边长（像素）。服务端会再统一裁切，这里给足清晰度即可。
  output: { type: Number, default: 512 },
})

const emit = defineEmits(['confirm', 'cancel'])

const MAX_ZOOM = 4

const stageEl = ref(null)
const imgRef = ref(null)
const processing = ref(false)
const loaded = ref(false)

// 舞台为正方形，view = 边长（px）；nw/nh 原图自然尺寸；baseScale = 铺满舞台的基准缩放
const geo = reactive({ view: 320, nw: 0, nh: 0, baseScale: 1 })
const zoom = ref(1)
const tx = ref(0)
const ty = ref(0)

const scale = computed(() => geo.baseScale * zoom.value)
const dispW = computed(() => geo.nw * scale.value)
const dispH = computed(() => geo.nh * scale.value)

const imgStyle = computed(() => ({
  width: `${dispW.value}px`,
  height: `${dispH.value}px`,
  transform: `translate(${tx.value}px, ${ty.value}px)`,
}))

// 约束平移：图片始终铺满舞台（不露底）
function clampOffset() {
  const minTx = geo.view - dispW.value
  const minTy = geo.view - dispH.value
  tx.value = Math.min(0, Math.max(minTx, tx.value))
  ty.value = Math.min(0, Math.max(minTy, ty.value))
}

// 围绕舞台某点（ax, ay）缩放，保持该点对应的图像位置不动
function applyZoom(nextZoom, ax, ay) {
  const z = Math.min(MAX_ZOOM, Math.max(1, nextZoom))
  const old = scale.value
  const imgX = (ax - tx.value) / old
  const imgY = (ay - ty.value) / old
  zoom.value = z
  const next = scale.value
  tx.value = ax - imgX * next
  ty.value = ay - imgY * next
  clampOffset()
}

function centerImage() {
  tx.value = (geo.view - dispW.value) / 2
  ty.value = (geo.view - dispH.value) / 2
  clampOffset()
}

function measureStage() {
  const el = stageEl.value
  if (el && el.clientWidth) geo.view = el.clientWidth
}

async function setup() {
  loaded.value = false
  await nextTick()
  measureStage()
  const img = imgRef.value
  if (!img || !img.naturalWidth) return
  geo.nw = img.naturalWidth
  geo.nh = img.naturalHeight
  // 铺满：取较大比例，保证两个方向都不小于舞台
  geo.baseScale = Math.max(geo.view / geo.nw, geo.view / geo.nh)
  zoom.value = 1
  centerImage()
  loaded.value = true
}

function onImgLoad() {
  setup()
}

// ---------- 指针：单指拖拽平移 + 双指捏合缩放 ----------
const pointers = new Map()
let panStart = null
let pinchStart = null

function stagePoint(e) {
  const r = stageEl.value.getBoundingClientRect()
  return { x: e.clientX - r.left, y: e.clientY - r.top }
}

function onPointerDown(e) {
  if (!loaded.value) return
  stageEl.value.setPointerCapture?.(e.pointerId)
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
  if (pointers.size === 1) {
    panStart = { tx: tx.value, ty: ty.value, x: e.clientX, y: e.clientY }
    pinchStart = null
  } else if (pointers.size === 2) {
    const pts = [...pointers.values()]
    const dist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y)
    pinchStart = { dist: dist || 1, zoom: zoom.value }
    panStart = null
  }
}

function onPointerMove(e) {
  if (!pointers.has(e.pointerId)) return
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
  if (pointers.size >= 2 && pinchStart) {
    const pts = [...pointers.values()]
    const dist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y)
    const r = stageEl.value.getBoundingClientRect()
    const midX = (pts[0].x + pts[1].x) / 2 - r.left
    const midY = (pts[0].y + pts[1].y) / 2 - r.top
    applyZoom((pinchStart.zoom * dist) / pinchStart.dist, midX, midY)
  } else if (panStart) {
    tx.value = panStart.tx + (e.clientX - panStart.x)
    ty.value = panStart.ty + (e.clientY - panStart.y)
    clampOffset()
  }
}

function onPointerUp(e) {
  pointers.delete(e.pointerId)
  stageEl.value?.releasePointerCapture?.(e.pointerId)
  if (pointers.size === 1) {
    // 从双指退回单指：用剩余指针重置拖拽基准，避免跳变
    const [p] = [...pointers.values()]
    panStart = { tx: tx.value, ty: ty.value, x: p.x, y: p.y }
    pinchStart = null
  } else if (pointers.size === 0) {
    panStart = null
    pinchStart = null
  }
}

function onWheel(e) {
  if (!loaded.value) return
  const { x, y } = stagePoint(e)
  const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1
  applyZoom(zoom.value * factor, x, y)
}

function onSlider(e) {
  applyZoom(Number(e.target.value), geo.view / 2, geo.view / 2)
}

function step(delta) {
  applyZoom(zoom.value + delta, geo.view / 2, geo.view / 2)
}

// ---------- 导出 ----------
function exportBlob() {
  return new Promise((resolve) => {
    const img = imgRef.value
    const out = props.output
    const canvas = document.createElement('canvas')
    canvas.width = out
    canvas.height = out
    const ctx = canvas.getContext('2d')
    // 舞台 → 原图坐标：源方块左上角与边长（自然像素）
    const sNat = geo.view / scale.value
    let sx = -tx.value / scale.value
    let sy = -ty.value / scale.value
    sx = Math.max(0, Math.min(sx, geo.nw - sNat))
    sy = Math.max(0, Math.min(sy, geo.nh - sNat))
    ctx.imageSmoothingQuality = 'high'
    ctx.drawImage(img, sx, sy, sNat, sNat, 0, 0, out, out)
    // 优先 WebP；不支持则回退 PNG（服务端统一再转 WebP）
    canvas.toBlob(
      (blob) => {
        if (blob) return resolve(blob)
        canvas.toBlob((png) => resolve(png), 'image/png')
      },
      'image/webp',
      0.92
    )
  })
}

async function confirm() {
  if (!loaded.value || processing.value) return
  processing.value = true
  try {
    const blob = await exportBlob()
    if (blob) emit('confirm', blob)
  } finally {
    processing.value = false
  }
}

function cancel() {
  emit('cancel')
}

// 重新打开或换图：重置交互态，待 <img> load 后再 setup
watch(
  () => [props.open, props.src],
  () => {
    if (props.open && props.src) {
      loaded.value = false
      zoom.value = 1
      // 若浏览器命中缓存、load 不再触发，下一帧主动 setup 兜底
      nextTick(() => {
        const img = imgRef.value
        if (img && img.complete && img.naturalWidth) setup()
      })
    }
  }
)

let ro = null
watch(
  () => props.open,
  (open) => {
    if (typeof ResizeObserver === 'undefined') return
    if (open) {
      nextTick(() => {
        if (!stageEl.value) return
        ro = new ResizeObserver(() => {
          if (!loaded.value) return
          measureStage()
          geo.baseScale = Math.max(geo.view / geo.nw, geo.view / geo.nh)
          clampOffset()
        })
        ro.observe(stageEl.value)
      })
    } else if (ro) {
      ro.disconnect()
      ro = null
    }
  }
)

onBeforeUnmount(() => {
  if (ro) ro.disconnect()
})
</script>

<template>
  <SosModal
    :open="open"
    title="裁切头像"
    :close-on-backdrop="false"
    @update:open="(v) => !v && cancel()"
    @close="cancel"
  >
    <div class="hauth-cropper">
      <div
        ref="stageEl"
        class="hauth-cropper__stage"
        @wheel.prevent="onWheel"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      >
        <img
          ref="imgRef"
          class="hauth-cropper__img"
          :src="src"
          :style="imgStyle"
          alt=""
          draggable="false"
          @load="onImgLoad"
        />
        <div class="hauth-cropper__ring" aria-hidden="true"></div>
      </div>

      <div class="hauth-cropper__zoom">
        <button type="button" class="hauth-cropper__zoom-btn" aria-label="缩小" @click="step(-0.2)">
          −
        </button>
        <input
          class="hauth-cropper__slider"
          type="range"
          min="1"
          :max="MAX_ZOOM"
          step="0.01"
          :value="zoom"
          aria-label="缩放"
          @input="onSlider"
        />
        <button type="button" class="hauth-cropper__zoom-btn" aria-label="放大" @click="step(0.2)">
          +
        </button>
      </div>
      <p class="hauth-cropper__hint">拖拽调整位置，滚轮或滑块缩放</p>
    </div>

    <template #footer>
      <SosButton variant="secondary" @click="cancel">取消</SosButton>
      <SosButton :loading="processing" :disabled="!loaded" @click="confirm"> 确定 </SosButton>
    </template>
  </SosModal>
</template>
