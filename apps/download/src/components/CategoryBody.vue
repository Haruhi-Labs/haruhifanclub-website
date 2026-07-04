<script setup>
import { computed } from 'vue'
import ResourceEntry from '@/components/ResourceEntry.vue'

// 递归渲染一个分类的内容，保留语雀 TOC 的原始顺序（直属条目与子分类可能交错）：
// 连续条目并成一段列表；子分类整组缩进一档（大纲式），标题按层级递减字号/颜色。
const props = defineProps({
  node: { type: Object, required: true },
  // 子分类深度：0 = 二级，1 = 三级，以此类推（决定缩进与标题样式）
  depth: { type: Number, default: 0 },
})

const segments = computed(() => {
  const segs = []
  let run = null
  for (const c of props.node.children || []) {
    if (c.kind === 'entry') {
      if (!run) {
        run = { type: 'entries', items: [] }
        segs.push(run)
      }
      run.items.push(c)
    } else if (c.kind === 'category') {
      run = null
      segs.push({ type: 'cat', cat: c })
    }
  }
  return segs
})
</script>

<template>
  <div class="dl-catbody">
    <template v-for="(seg, i) in segments" :key="i">
      <div v-if="seg.type === 'entries'" class="dl-entrylist">
        <ResourceEntry v-for="e in seg.items" :key="e.id" :entry="e" />
      </div>
      <div v-else class="dl-subgroup">
        <h4 class="dl-subhead" :data-level="depth">
          <span class="dl-subhead__mark" aria-hidden="true"></span>
          <span class="dl-subhead__text">{{ seg.cat.title }}</span>
          <span class="dl-subhead__n">{{ seg.cat.count }}</span>
        </h4>
        <CategoryBody :node="seg.cat" :depth="depth + 1" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.dl-catbody {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}

/* 子分类整组：每深一层向右缩进一档，形成清晰的大纲层级 */
.dl-subgroup {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
  padding-left: var(--sos-space-4);
}

.dl-subhead {
  display: flex;
  align-items: center;
  gap: var(--sos-space-2);
  margin: 0;
}
.dl-subhead__mark {
  flex: none;
}
.dl-subhead__n {
  font-size: var(--sos-text-2xs);
  color: var(--sos-text-tertiary);
  font-variant-numeric: var(--sos-numeric-tabular);
}

/* 二级（depth 0）：晴空蓝、较显眼的小标题 + 细蓝竖条 */
.dl-subhead[data-level='0'] .dl-subhead__text {
  font-size: var(--sos-text-sm);
  font-weight: 700;
  color: var(--sos-accent);
}
.dl-subhead[data-level='0'] .dl-subhead__mark {
  width: 3px;
  height: 0.95em;
  border-radius: var(--sos-radius-full);
  background: var(--sos-accent);
}

/* 三级及更深（depth ≥ 1）：更小更淡 + 空心小点 */
.dl-subhead:not([data-level='0']) .dl-subhead__text {
  font-size: var(--sos-text-xs);
  font-weight: 600;
  color: var(--sos-text-secondary);
}
.dl-subhead:not([data-level='0']) .dl-subhead__mark {
  width: 5px;
  height: 5px;
  border-radius: var(--sos-radius-full);
  border: 1.5px solid var(--sos-text-tertiary);
}
</style>
