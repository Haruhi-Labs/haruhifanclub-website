<script setup>
import { computed } from 'vue'
import { coverUrl } from '@/api'
import { categoryLabel } from '@/lib/format'

const props = defineProps({
  path: { type: String, default: null },
  title: { type: String, default: '' },
  category: { type: String, default: 'other' },
})

const url = computed(() => coverUrl(props.path))
const catLabel = computed(() => categoryLabel(props.category))

// 无封面时按分类生成明快的扁平渐变海报（三色同系渐变 + 抽象几何），非仿真书封
const HUES = {
  daily: 28,
  romance: 340,
  school: 210,
  supernatural: 268,
  scifi: 195,
  adventure: 16,
  parallel: 250,
  comedy: 45,
  drama: 4,
  healing: 158,
  other: 220,
}
const placeholderStyle = computed(() => {
  const h = HUES[props.category] ?? 220
  return {
    '--cover-h1': `${h}`,
    '--cover-h2': `${(h + 18) % 360}`,
    '--cover-h3': `${(h + 40) % 360}`,
  }
})
</script>

<template>
  <div class="fiction-cover">
    <img v-if="url" :src="url" :alt="title" loading="lazy" />
    <div v-else class="fiction-cover__ph" :style="placeholderStyle">
      <span class="fiction-cover__cat">{{ catLabel }}</span>
      <span class="fiction-cover__name">{{ title }}</span>
    </div>
  </div>
</template>
