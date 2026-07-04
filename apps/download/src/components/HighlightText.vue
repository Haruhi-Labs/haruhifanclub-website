<script setup>
import { computed } from 'vue'

const props = defineProps({
  text: { type: String, default: '' },
  query: { type: String, default: '' },
})

// 把 text 按 query 的分词切段，命中段用 <mark> 高亮。不使用 v-html，天然防注入。
const segments = computed(() => {
  const text = props.text || ''
  const q = (props.query || '').trim()
  if (!q) return [{ t: text, hit: false }]
  const tokens = [...new Set(q.toLowerCase().split(/\s+/).filter(Boolean))]
  if (!tokens.length) return [{ t: text, hit: false }]
  const esc = tokens.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
  const re = new RegExp(`(${esc.join('|')})`, 'ig')
  // split 带捕获组：命中的子串会作为独立数组项返回，其小写必等于某个 token。
  return text
    .split(re)
    .filter((p) => p !== '')
    .map((p) => ({ t: p, hit: tokens.includes(p.toLowerCase()) }))
})
</script>

<template>
  <span class="dl-hltext">
    <component :is="s.hit ? 'mark' : 'span'" v-for="(s, i) in segments" :key="i" :class="s.hit ? 'dl-hl' : undefined">{{ s.t }}</component>
  </span>
</template>

<style scoped>
.dl-hl {
  background: var(--sos-signal);
  color: var(--sos-ink-950, #1a1d24);
  border-radius: 3px;
  padding: 0 1px;
  font-weight: 700;
}
</style>
