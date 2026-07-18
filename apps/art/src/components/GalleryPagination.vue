<template>
  <nav v-if="pageCount > 1" class="gallery-pagination" aria-label="分页">
    <button
      class="gallery-pagination__arrow"
      type="button"
      :disabled="page <= 1 || loading"
      aria-label="上一页"
      title="上一页"
      @click="go(page - 1)"
    >
      <ChevronLeft :size="18" aria-hidden="true" />
    </button>

    <template v-for="(item, index) in pageList" :key="`${item}-${index}`">
      <span v-if="item === '…'" class="gallery-pagination__ellipsis" aria-hidden="true">…</span>
      <button
        v-else
        class="gallery-pagination__page"
        type="button"
        :class="{ active: item === page }"
        :disabled="loading"
        :aria-current="item === page ? 'page' : undefined"
        :aria-label="`第 ${item} 页`"
        @click="go(item)"
      >
        {{ item }}
      </button>
    </template>

    <button
      class="gallery-pagination__arrow"
      type="button"
      :disabled="page >= pageCount || loading"
      aria-label="下一页"
      title="下一页"
      @click="go(page + 1)"
    >
      <ChevronRight :size="18" aria-hidden="true" />
    </button>
  </nav>
</template>

<script setup>
import { computed } from 'vue'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'

const props = defineProps({
  page: { type: Number, default: 1 },
  pageCount: { type: Number, default: 1 },
  loading: { type: Boolean, default: false }
})

const emit = defineEmits(['goPage'])

const pageList = computed(() => {
  const total = props.pageCount
  const current = props.page
  if (total <= 7) return Array.from({ length: total }, (_, index) => index + 1)

  const pages = new Set([1, total, current - 1, current, current + 1])
  const sorted = [...pages].filter(value => value >= 1 && value <= total).sort((a, b) => a - b)
  const result = []
  let previous = 0
  for (const value of sorted) {
    if (previous && value - previous > 1) result.push('…')
    result.push(value)
    previous = value
  }
  return result
})

function go(target) {
  const page = Math.min(Math.max(Number(target) || 1, 1), props.pageCount)
  if (page !== props.page) emit('goPage', page)
}
</script>

<style scoped>
.gallery-pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin-top: 34px;
}

.gallery-pagination button {
  display: inline-grid;
  flex: 0 0 auto;
  width: 38px;
  height: 38px;
  place-items: center;
  padding: 0;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 13px;
  font-weight: 750;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(95, 215, 226, 0.7);
  border-radius: 50%;
}

.gallery-pagination button:hover:not(:disabled) {
  color: var(--sos-text-primary);
  border-color: rgb(59, 190, 204);
}

.gallery-pagination__page.active {
  color: white;
  background: rgb(186, 112, 235);
  border-color: rgb(186, 112, 235);
}

.gallery-pagination button:disabled { cursor: not-allowed; opacity: 0.38; }

.gallery-pagination__ellipsis {
  display: inline-grid;
  flex: 0 0 14px;
  height: 38px;
  place-items: center;
  color: var(--sos-text-tertiary);
}

@media (max-width: 480px) {
  .gallery-pagination { gap: 3px; }
  .gallery-pagination button { width: 32px; height: 32px; font-size: 12px; }
  .gallery-pagination__arrow { width: 34px !important; height: 34px !important; }
  .gallery-pagination__ellipsis { flex-basis: 14px; height: 34px; }
}
</style>
