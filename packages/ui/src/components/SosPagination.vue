<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: number
    pageCount: number
    siblings?: number
  }>(),
  {
    modelValue: 1,
    siblings: 1,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const pages = computed<(number | '…')[]>(() => {
  const total = props.pageCount
  const current = props.modelValue
  const span = props.siblings
  if (total <= 1) return [1]

  const range = new Set<number>([1, total])
  for (let i = current - span; i <= current + span; i += 1) {
    if (i >= 1 && i <= total) range.add(i)
  }
  const sorted = [...range].sort((a, b) => a - b)
  const out: (number | '…')[] = []
  let prev = 0
  for (const page of sorted) {
    if (prev && page - prev > 1) out.push('…')
    out.push(page)
    prev = page
  }
  return out
})

function go(page: number) {
  const next = Math.min(Math.max(page, 1), props.pageCount)
  if (next !== props.modelValue) emit('update:modelValue', next)
}
</script>

<template>
  <nav class="sos-pagination" aria-label="分页">
    <button
      type="button"
      class="sos-pagination__item"
      :disabled="modelValue <= 1"
      aria-label="上一页"
      @click="go(modelValue - 1)"
    >
      ‹
    </button>
    <template v-for="(page, index) in pages">
      <span
        v-if="page === '…'"
        :key="`gap-${index}`"
        class="sos-pagination__item"
        aria-hidden="true"
      >
        …
      </span>
      <button
        v-else
        :key="page"
        type="button"
        class="sos-pagination__item"
        :aria-current="page === modelValue ? 'page' : undefined"
        @click="go(page)"
      >
        {{ page }}
      </button>
    </template>
    <button
      type="button"
      class="sos-pagination__item"
      :disabled="modelValue >= pageCount"
      aria-label="下一页"
      @click="go(modelValue + 1)"
    >
      ›
    </button>
  </nav>
</template>
