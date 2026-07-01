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

// 无封面时按分类生成稳定的「书封」占位：渐变底 + 书脊 + 书名牌 + 印记
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
    '--cover-h1': `${h}`,
    '--cover-h2': `${(h + 26) % 360}`,
  }
})
</script>

<template>
  <div class="fiction-cover">
    <img v-if="url" :src="url" :alt="title" loading="lazy" />
    <div v-else class="fiction-cover__ph" :style="placeholderStyle">
      <span class="fiction-cover__spine" aria-hidden="true"></span>
      <span class="fiction-cover__cat">{{ catLabel }}</span>
      <span class="fiction-cover__name">{{ title }}</span>
      <span class="fiction-cover__seal" aria-hidden="true">SOS 文库</span>
    </div>
  </div>
</template>
