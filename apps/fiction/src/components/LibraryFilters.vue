<script setup>
// 书库筛选块（分类 / 连载状态 / 热门标签）。
// 抽成独立组件，供 PC 侧栏与移动端筛选抽屉复用同一套标记与样式，避免重复。
defineProps({
  filters: { type: Object, required: true }, // 父级 reactive 筛选状态
  categories: { type: Array, required: true },
  completedOptions: { type: Array, required: true },
  tags: { type: Array, default: () => [] },
})
const emit = defineEmits(['apply'])
</script>

<template>
  <div class="libf">
    <div class="libf__block">
      <h3 class="libf__title">分类</h3>
      <ul class="libf__cats">
        <li>
          <button :class="{ on: !filters.category }" @click="emit('apply', { category: '' })">全部作品</button>
        </li>
        <li v-for="c in categories" :key="c.slug">
          <button :class="{ on: filters.category === c.slug }" @click="emit('apply', { category: c.slug })">
            {{ c.label }}
          </button>
        </li>
      </ul>
    </div>

    <div class="libf__block">
      <h3 class="libf__title">连载状态</h3>
      <div class="libf__chips">
        <button
          v-for="c in completedOptions"
          :key="c.key"
          class="libf__chip"
          :class="{ on: filters.completed === c.key }"
          @click="emit('apply', { completed: c.key })"
        >
          {{ c.label }}
        </button>
      </div>
    </div>

    <div v-if="tags.length" class="libf__block">
      <h3 class="libf__title">热门标签</h3>
      <div class="libf__tags">
        <button
          v-for="t in tags"
          :key="t.name"
          class="libf__tag"
          :class="{ on: filters.tag === t.name }"
          @click="emit('apply', { tag: filters.tag === t.name ? '' : t.name })"
        >
          {{ t.name }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.libf {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-6);
}
.libf__title {
  font-size: var(--sos-text-sm);
  font-weight: 700;
  color: var(--sos-text-secondary);
  margin: 0 0 var(--sos-space-3);
  letter-spacing: var(--sos-tracking-wide);
}
.libf__cats {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.libf__cats button {
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 7px 12px;
  border-radius: var(--sos-radius-sm);
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
}
.libf__cats button:hover {
  background: var(--sos-bg-subtle);
}
.libf__cats button.on {
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
  font-weight: 600;
}
.libf__chips,
.libf__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.libf__chip,
.libf__tag {
  border: 1px solid var(--sos-border-default);
  background: var(--sos-bg-surface);
  cursor: pointer;
  padding: 4px 12px;
  border-radius: var(--sos-radius-full);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-secondary);
}
.libf__chip.on,
.libf__tag.on {
  border-color: var(--sos-accent);
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
}
</style>
