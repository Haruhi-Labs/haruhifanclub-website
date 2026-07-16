<script setup>
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { SosBadge } from '@haruhi/ui'

const props = defineProps({
  items: { type: Array, default: () => [] },
  compact: { type: Boolean, default: false },
})

const viewport = ref(null)
const dragging = ref(false)
const dragThreshold = 6
let activePointerId = null
let dragStartY = 0
let dragStartScrollTop = 0
let didDrag = false
let suppressClick = false
let clickResetTimer = null

const formatter = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
})

function formatDateTime(value) {
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? String(value).replace('T', ' ').slice(0, 16)
    : formatter.format(date)
}

function timeRange(item) {
  const start = formatDateTime(item.startsAt)
  const end = item.endsAt ? formatDateTime(item.endsAt) : '结束时间待定'
  return `${start}—${end}`
}

function eventPath(item) {
  return `/branches/${item.branchSlug}/events/${item.eventSlug || item.slug}`
}

function removePointerListeners() {
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', finishPointerDrag)
  window.removeEventListener('pointercancel', finishPointerDrag)
}

function onPointerDown(event) {
  if (props.compact || !event.isPrimary || event.pointerType === 'touch' || event.button !== 0)
    return
  activePointerId = event.pointerId
  dragStartY = event.clientY
  dragStartScrollTop = viewport.value?.scrollTop || 0
  didDrag = false
  window.addEventListener('pointermove', onPointerMove, { passive: false })
  window.addEventListener('pointerup', finishPointerDrag)
  window.addEventListener('pointercancel', finishPointerDrag)
}

function onPointerMove(event) {
  if (event.pointerId !== activePointerId || !viewport.value) return
  const distance = event.clientY - dragStartY
  if (!didDrag && Math.abs(distance) < dragThreshold) return
  if (!didDrag) {
    didDrag = true
    dragging.value = true
    viewport.value.setPointerCapture?.(event.pointerId)
  }
  event.preventDefault()
  viewport.value.scrollTop = dragStartScrollTop - distance
}

function finishPointerDrag(event) {
  if (event.pointerId !== activePointerId) return
  if (didDrag) {
    suppressClick = true
    window.clearTimeout(clickResetTimer)
    clickResetTimer = window.setTimeout(() => {
      suppressClick = false
    }, 0)
  }
  if (viewport.value?.hasPointerCapture?.(event.pointerId)) {
    viewport.value.releasePointerCapture(event.pointerId)
  }
  activePointerId = null
  didDrag = false
  dragging.value = false
  removePointerListeners()
}

function onClickCapture(event) {
  if (!suppressClick) return
  event.preventDefault()
  event.stopPropagation()
}

function onWheel(event) {
  if (props.compact || event.ctrlKey) return
  event.preventDefault()
  const deltaScale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? window.innerHeight : 1
  window.scrollBy({
    top: event.deltaY * deltaScale,
    left: event.deltaX * deltaScale,
    behavior: 'auto',
  })
}

function onKeydown(event) {
  if (props.compact || !viewport.value) return
  const pageDistance = Math.max(48, viewport.value.clientHeight * 0.8)
  const distances = {
    ArrowUp: -48,
    ArrowDown: 48,
    PageUp: -pageDistance,
    PageDown: pageDistance,
  }
  if (event.key === 'Home') {
    event.preventDefault()
    viewport.value.scrollTo({ top: 0 })
  } else if (event.key === 'End') {
    event.preventDefault()
    viewport.value.scrollTo({ top: viewport.value.scrollHeight })
  } else if (event.key in distances) {
    event.preventDefault()
    viewport.value.scrollBy({ top: distances[event.key] })
  }
}

watch(
  () =>
    props.items
      .map((item) => `${item.id}:${item.startsAt}:${item.title}:${item.branchLocalityName || ''}`)
      .join('|'),
  async () => {
    await nextTick()
    viewport.value?.scrollTo({ top: 0 })
  }
)

onBeforeUnmount(() => {
  removePointerListeners()
  window.clearTimeout(clickResetTimer)
})
</script>

<template>
  <div class="activity-timeline__block">
    <div
      ref="viewport"
      class="activity-timeline__viewport"
      :class="{
        'activity-timeline__viewport--compact': compact,
        'is-dragging': dragging,
      }"
      :tabindex="compact ? undefined : 0"
      role="region"
      :aria-label="compact ? '活动时间线' : '活动时间线，可按住拖动、触摸滑动或使用方向键浏览'"
      @pointerdown="onPointerDown"
      @click.capture="onClickCapture"
      @wheel="onWheel"
      @keydown="onKeydown"
    >
      <ol class="activity-timeline" :class="{ 'activity-timeline--compact': compact }">
        <li v-for="item in items" :key="item.id" class="activity-timeline__item">
          <time :datetime="item.startsAt">{{ timeRange(item) }}</time>
          <span class="activity-timeline__marker" aria-hidden="true"></span>
          <div class="activity-timeline__event">
            <SosBadge
              v-if="item.branchLocalityName"
              class="activity-timeline__locality"
              variant="outline"
            >
              {{ item.branchLocalityName }}
            </SosBadge>
            <RouterLink :to="eventPath(item)" draggable="false" @dragstart.prevent>
              {{ item.title }}
            </RouterLink>
          </div>
        </li>
      </ol>
    </div>
    <p v-if="!compact && items.length > 5" class="activity-timeline__hint">
      按住并上下拖动查看更多活动
    </p>
  </div>
</template>
