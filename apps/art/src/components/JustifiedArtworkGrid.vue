<template>
  <div
    ref="root"
    class="justified-artwork-grid"
    :style="{ height: `${layout.height}px` }"
  >
    <div
      v-for="entry in layout.entries"
      :key="entry.key"
      class="justified-artwork-grid__item"
      :style="entry.style"
    >
      <slot :item="entry.item" :index="entry.index"></slot>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import justifiedLayout from 'justified-layout'
import { thumbUrl } from '../services/api.js'

const props = defineProps({
  items: { type: Array, default: () => [] },
  targetRowHeight: { type: Number, default: 250 },
  mobileTargetRowHeight: { type: Number, default: 165 },
  gap: { type: Number, default: 12 },
  mobileGap: { type: Number, default: 8 },
  mobilePairs: { type: Boolean, default: true },
  randomizeRowHeights: { type: Boolean, default: false },
  balancedRowCount: { type: Number, default: 0 },
  minRowHeight: { type: Number, default: 0 },
  maxRowHeight: { type: Number, default: 0 },
  mobileMinRowHeight: { type: Number, default: 0 },
  mobileMaxRowHeight: { type: Number, default: 0 },
  centerIncompleteRows: { type: Boolean, default: false },
})

const ratioCache = new Map()
const ratioRequests = new Map()
const root = ref(null)
const containerWidth = ref(0)
const measuredRatios = ref([])
const layoutEntropy = ref(Math.floor(Math.random() * 0x7fffffff))
let resizeObserver = null
let ratioLoadToken = 0

function itemKey(item, index) {
  return item?.id ?? item?.file_path ?? item?.title ?? item?.image_url ?? index
}

function imageSource(item) {
  return thumbUrl(item?.image_url || item?.imageUrl || item?.url || '', 640)
}

function numericRatio(width, height) {
  const w = Number(width)
  const h = Number(height)
  return w > 0 && h > 0 ? w / h : 0
}

function storedRatio(item) {
  return numericRatio(item?.image_width, item?.image_height)
    || numericRatio(item?.width, item?.height)
    || numericRatio(item?.images?.[0]?.width, item?.images?.[0]?.height)
}

function requestRatio(url) {
  if (!url) return Promise.resolve(4 / 3)
  if (ratioCache.has(url)) return Promise.resolve(ratioCache.get(url))
  if (ratioRequests.has(url)) return ratioRequests.get(url)
  const request = new Promise(resolve => {
    const image = new Image()
    image.decoding = 'async'
    image.onload = () => resolve(numericRatio(image.naturalWidth, image.naturalHeight) || 4 / 3)
    image.onerror = () => resolve(4 / 3)
    image.src = url
  }).then(ratio => {
    ratioCache.set(url, ratio)
    ratioRequests.delete(url)
    return ratio
  })
  ratioRequests.set(url, request)
  return request
}

async function measureItems() {
  const token = ++ratioLoadToken
  const items = props.items || []
  measuredRatios.value = items.map(item => storedRatio(item) || ratioCache.get(imageSource(item)) || 4 / 3)
  await Promise.all(items.map(async (item, index) => {
    if (storedRatio(item)) return
    const ratio = await requestRatio(imageSource(item))
    if (token !== ratioLoadToken || props.items[index] !== item) return
    const next = [...measuredRatios.value]
    next[index] = ratio
    measuredRatios.value = next
  }))
}

function pairLayout(items, ratios, width, gap) {
  const entries = []
  let top = 0
  for (let start = 0; start < items.length; start += 2) {
    const rowItems = items.slice(start, start + 2)
    const rowRatios = ratios.slice(start, start + 2)
    const spacing = gap * Math.max(0, rowItems.length - 1)
    const naturalHeight = rowItems.length === 1
      ? Math.min(props.mobileTargetRowHeight, width / rowRatios[0])
      : (width - spacing) / rowRatios.reduce((sum, ratio) => sum + ratio, 0)
    const rowHeight = Math.min(
      props.mobileMaxRowHeight || Number.POSITIVE_INFINITY,
      Math.max(props.mobileMinRowHeight || 0, naturalHeight),
    )
    const availableWidth = width - spacing
    const ratioSum = rowRatios.reduce((sum, ratio) => sum + ratio, 0)
    let left = 0
    rowItems.forEach((item, offset) => {
      const boxWidth = availableWidth * rowRatios[offset] / ratioSum
      const index = start + offset
      entries.push({ item, index, key: itemKey(item, index), left, top, width: boxWidth, height: rowHeight })
      left += boxWidth + gap
    })
    top += rowHeight + gap
  }
  return { entries, height: Math.max(0, top - gap) }
}

function randomizedUniformRowsLayout(items, ratios, width, gap, targetHeight, itemsPerRow, seed) {
  const entries = []
  let top = 0
  let rowIndex = 0
  for (let start = 0; start < items.length; start += itemsPerRow) {
    const rowItems = items.slice(start, start + itemsPerRow)
    const rowRatios = ratios.slice(start, start + itemsPerRow)
    const weights = rowRatios.map(ratio => Math.min(1.65, Math.max(0.72, Math.sqrt(ratio))))
    const availableWidth = width - gap * Math.max(0, rowItems.length - 1)
    const weightSum = weights.reduce((sum, weight) => sum + weight, 0)
    const rowHeight = targetHeight * (0.88 + seededFraction(seed, rowIndex + 73) * 0.24)
    let left = 0
    rowItems.forEach((item, offset) => {
      const boxWidth = availableWidth * weights[offset] / weightSum
      const index = start + offset
      entries.push({ item, index, key: itemKey(item, index), left, top, width: boxWidth, height: rowHeight })
      left += boxWidth + gap
    })
    top += rowHeight + gap
    rowIndex += 1
  }
  return { entries, height: Math.max(0, top - gap) }
}

function balancedRowsLayout(items, ratios, width, gap, targetHeight, rowCount, minHeight, maxHeight) {
  if (rowCount !== 2 || items.length < 4) return null
  let best = null
  for (let split = 2; split <= items.length - 2; split += 1) {
    const firstRatios = ratios.slice(0, split)
    const secondRatios = ratios.slice(split)
    const firstHeight = (width - gap * (firstRatios.length - 1))
      / firstRatios.reduce((sum, ratio) => sum + ratio, 0)
    const secondHeight = (width - gap * (secondRatios.length - 1))
      / secondRatios.reduce((sum, ratio) => sum + ratio, 0)
    const targetCost = Math.abs(Math.log(firstHeight / targetHeight))
      + Math.abs(Math.log(secondHeight / targetHeight))
    const balanceCost = Math.abs(firstHeight - secondHeight) / targetHeight
    const cost = targetCost + balanceCost * 0.7
    if (!best || cost < best.cost) best = { split, cost }
  }
  const rows = [
    Array.from({ length: best.split }, (_, index) => index),
    Array.from({ length: items.length - best.split }, (_, index) => best.split + index),
  ]
  const entries = []
  let top = 0
  for (const indices of rows) {
    const ratioSum = indices.reduce((sum, index) => sum + ratios[index], 0)
    const availableWidth = width - gap * (indices.length - 1)
    const naturalHeight = availableWidth / ratioSum
    const rowHeight = Math.min(
      maxHeight || Number.POSITIVE_INFINITY,
      Math.max(minHeight || 0, naturalHeight),
    )
    let left = 0
    for (const index of indices) {
      const boxWidth = availableWidth * ratios[index] / ratioSum
      entries.push({
        item: items[index],
        index,
        key: itemKey(items[index], index),
        left,
        top,
        width: boxWidth,
        height: rowHeight,
      })
      left += boxWidth + gap
    }
    top += rowHeight + gap
  }
  return {
    entries,
    height: entries.reduce((height, entry) => Math.max(height, entry.top + entry.height), 0),
  }
}

function normalizeRows(boxes, items, ratios, width, gap, targetHeight, maxHeight, centerIncompleteRows) {
  const rows = []
  for (let index = 0; index < boxes.length; index += 1) {
    const box = boxes[index]
    const last = rows.at(-1)
    if (!last || last.top !== box.top) rows.push({ top: box.top, indices: [index] })
    else last.indices.push(index)
  }
  if (!rows.length) return []

  const fillHeight = row => {
    const ratioSum = row.indices.reduce((sum, index) => sum + ratios[index], 0)
    return (width - gap * (row.indices.length - 1)) / ratioSum
  }
  const last = rows.at(-1)
  if (rows.length > 1 && (last.indices.length < 2 || fillHeight(last) > targetHeight * 1.45)) {
    const donor = rows.at(-2)
    const combined = [...donor.indices, ...last.indices]
    if (combined.length < 4) {
      last.indices = combined
      rows.splice(rows.length - 2, 1)
    } else {
      let best = null
      for (let split = 2; split <= combined.length - 2; split += 1) {
        const first = { indices: combined.slice(0, split) }
        const second = { indices: combined.slice(split) }
        const firstHeight = fillHeight(first)
        const secondHeight = fillHeight(second)
        const cost = Math.abs(Math.log(firstHeight / targetHeight))
          + Math.abs(Math.log(secondHeight / targetHeight))
          + Math.abs(firstHeight - secondHeight) / targetHeight
        if (!best || cost < best.cost) best = { split, cost }
      }
      donor.indices = combined.slice(0, best.split)
      last.indices = combined.slice(best.split)
    }
  }

  const entries = []
  let top = 0
  for (const row of rows) {
    const single = row.indices.length === 1
    const naturalHeight = single
      ? Math.min(targetHeight, width / ratios[row.indices[0]])
      : fillHeight(row)
    const rowHeight = Math.min(maxHeight || Number.POSITIVE_INFINITY, naturalHeight)
    const rowWidth = row.indices.reduce((sum, index) => sum + rowHeight * ratios[index], 0)
      + gap * Math.max(0, row.indices.length - 1)
    let left = centerIncompleteRows ? Math.max(0, (width - rowWidth) / 2) : 0
    for (const index of row.indices) {
      const boxWidth = rowHeight * ratios[index]
      entries.push({
        item: items[index],
        index,
        key: itemKey(items[index], index),
        left,
        top,
        width: boxWidth,
        height: rowHeight,
      })
      left += boxWidth + gap
    }
    top += rowHeight + gap
  }
  return entries
}

function seededFraction(seed, index) {
  let value = (seed ^ Math.imul(index + 1, 0x9e3779b1)) >>> 0
  value = Math.imul(value ^ (value >>> 16), 0x21f0aaad) >>> 0
  value = Math.imul(value ^ (value >>> 15), 0x735a2d97) >>> 0
  return ((value ^ (value >>> 15)) >>> 0) / 0x100000000
}

function randomizedRows(ratios, width, gap, targetHeight, seed) {
  const minHeight = targetHeight * 0.78
  const maxHeight = targetHeight * 1.32

  const fillHeight = (start, count) => {
    const ratioSum = ratios.slice(start, start + count).reduce((sum, ratio) => sum + ratio, 0)
    return (width - gap * (count - 1)) / ratioSum
  }

  const rows = []
  let start = 0
  let rowIndex = 0
  while (start < ratios.length) {
    const remaining = ratios.length - start
    const counts = remaining === 1
      ? [1]
      : Array.from({ length: Math.min(6, remaining) - 1 }, (_, index) => index + 2)
        .filter(count => remaining - count !== 1)
    const candidates = counts.map(count => {
      const height = count === 1
        ? Math.min(targetHeight, width / ratios[start])
        : fillHeight(start, count)
      const belowBound = Math.max(0, minHeight - height) / targetHeight
      const aboveBound = Math.max(0, height - maxHeight) / targetHeight
      return { count, height, boundCost: belowBound + aboveBound }
    })
    const bounded = candidates.filter(candidate => candidate.height >= minHeight && candidate.height <= maxHeight)
    let pool = bounded
    if (!pool.length) {
      const ranked = [...candidates].sort((a, b) => a.boundCost - b.boundCost)
      const bestCost = ranked[0]?.boundCost ?? 0
      pool = ranked.filter(candidate => candidate.boundCost <= bestCost + 0.18).slice(0, 2)
    }
    const pick = Math.min(
      pool.length - 1,
      Math.floor(seededFraction(seed, rowIndex * 17 + start + 31) * pool.length),
    )
    const count = pool[pick].count
    rows.push(Array.from({ length: count }, (_, offset) => start + offset))
    start += count
    rowIndex += 1
  }

  return rows
}

function entriesFromRows(rows, items, ratios, width, gap, targetHeight) {
  const entries = []
  let top = 0
  for (const indices of rows) {
    const ratioSum = indices.reduce((sum, index) => sum + ratios[index], 0)
    const rowHeight = indices.length === 1
      ? Math.min(targetHeight, width / ratioSum)
      : (width - gap * (indices.length - 1)) / ratioSum
    let left = 0
    for (const index of indices) {
      const boxWidth = rowHeight * ratios[index]
      entries.push({
        item: items[index],
        index,
        key: itemKey(items[index], index),
        left,
        top,
        width: boxWidth,
        height: rowHeight,
      })
      left += boxWidth + gap
    }
    top += rowHeight + gap
  }
  return entries
}

const layout = computed(() => {
  const items = props.items || []
  const width = containerWidth.value
  if (!items.length || width <= 0) return { entries: [], height: 0 }
  const ratios = items.map((_, index) => measuredRatios.value[index] || 4 / 3)
  const mobile = width <= 640
  const gap = mobile ? props.mobileGap : props.gap
  let result

  if (mobile && props.randomizeRowHeights) {
    result = randomizedUniformRowsLayout(
      items,
      ratios,
      width,
      gap,
      props.mobileTargetRowHeight,
      2,
      layoutEntropy.value,
    )
  } else if (mobile && props.mobilePairs) {
    result = pairLayout(items, ratios, width, gap)
  } else if (props.balancedRowCount > 0) {
    result = balancedRowsLayout(
      items,
      ratios,
      width,
      gap,
      props.targetRowHeight,
      props.balancedRowCount,
      props.minRowHeight,
      props.maxRowHeight,
    )
  } else {
    const targetHeight = mobile ? props.mobileTargetRowHeight : props.targetRowHeight
    let entries
    if (props.randomizeRowHeights) {
      const rows = randomizedRows(ratios, width, gap, targetHeight, layoutEntropy.value)
      entries = entriesFromRows(rows, items, ratios, width, gap, targetHeight)
    } else {
      const geometry = justifiedLayout(ratios, {
        containerWidth: width,
        containerPadding: 0,
        boxSpacing: gap,
        targetRowHeight: targetHeight,
        targetRowHeightTolerance: 0.28,
        showWidows: true,
      })
      entries = normalizeRows(
        geometry.boxes,
        items,
        ratios,
        width,
        gap,
        targetHeight,
        mobile ? props.mobileMaxRowHeight : props.maxRowHeight,
        props.centerIncompleteRows,
      )
    }
    result = {
      entries,
      height: entries.reduce((height, entry) => Math.max(height, entry.top + entry.height), 0),
    }
  }

  return {
    height: result.height,
    entries: result.entries.map(entry => ({
      ...entry,
      style: {
        left: `${entry.left}px`,
        top: `${entry.top}px`,
        width: `${entry.width}px`,
        height: `${entry.height}px`,
      },
    })),
  }
})

watch(() => props.items, () => {
  layoutEntropy.value = Math.floor(Math.random() * 0x7fffffff)
  measureItems()
}, { immediate: true })

onMounted(() => {
  resizeObserver = new ResizeObserver(entries => {
    containerWidth.value = entries[0]?.contentRect?.width || root.value?.clientWidth || 0
  })
  resizeObserver.observe(root.value)
})

onBeforeUnmount(() => {
  ratioLoadToken += 1
  resizeObserver?.disconnect()
})
</script>

<style scoped>
.justified-artwork-grid {
  position: relative;
  width: 100%;
  transition: height 0.28s var(--sos-ease-out);
}

.justified-artwork-grid__item {
  position: absolute;
  min-width: 0;
  overflow: visible;
  transition:
    left 0.28s var(--sos-ease-out),
    top 0.28s var(--sos-ease-out),
    width 0.28s var(--sos-ease-out),
    height 0.28s var(--sos-ease-out);
}

.justified-artwork-grid__item > :deep(*) {
  width: 100%;
  height: 100%;
}

@media (prefers-reduced-motion: reduce) {
  .justified-artwork-grid,
  .justified-artwork-grid__item { transition: none; }
}
</style>
