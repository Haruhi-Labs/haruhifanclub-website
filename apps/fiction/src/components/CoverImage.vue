<script setup>
import { computed } from 'vue'
import { coverUrl } from '@/api'

const props = defineProps({
  path: { type: String, default: null },
  title: { type: String, default: '' },
  category: { type: String, default: 'other' },
})

const url = computed(() => coverUrl(props.path))
const initial = computed(() => (props.title || '文').trim().charAt(0) || '文')

// 无封面时按分类生成稳定的书脊式渐变占位
const HUES = {
  daily: 28,
  romance: 342,
  school: 208,
  supernatural: 268,
  scifi: 190,
  adventure: 18,
  parallel: 250,
  comedy: 45,
  drama: 6,
  healing: 150,
  other: 40,
}
const placeholderStyle = computed(() => {
  const h = HUES[props.category] ?? 40
  return {
    background: `linear-gradient(150deg, hsl(${h} 42% 60%), hsl(${(h + 26) % 360} 46% 44%))`,
  }
})
</script>

<template>
  <div class="fiction-cover">
    <img v-if="url" :src="url" :alt="title" loading="lazy" />
    <div v-else class="fiction-cover__ph" :style="placeholderStyle">
      <span>{{ initial }}</span>
    </div>
  </div>
</template>
