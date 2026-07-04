<script setup>
import { fmtDate } from '@/lib/format'
import HighlightText from '@/components/HighlightText.vue'

defineProps({
  entry: { type: Object, required: true },
  query: { type: String, default: '' },
  // 搜索结果里展示所属分类路径，帮助定位
  showPath: { type: Boolean, default: false },
  // 是否展示摘要（浏览时不展示以保持索引密度；搜索时展示以提供命中上下文）
  showDesc: { type: Boolean, default: false },
})
</script>

<template>
  <a :href="entry.url" target="_blank" rel="noopener" class="dl-row">
    <span v-if="showPath && entry.path?.length" class="dl-row__path">{{ entry.path.join(' › ') }}</span>

    <span class="dl-row__line">
      <span class="dl-row__title"><HighlightText :text="entry.title" :query="query" /></span>
      <span class="dl-row__meta">
        <span class="dl-row__date">{{ fmtDate(entry.updatedAt) }}</span>
        <svg class="dl-row__go" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M7 17 17 7M9 7h8v8" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </span>
    </span>

    <span v-if="showDesc && entry.description" class="dl-row__desc">
      <HighlightText :text="entry.description" :query="query" />
    </span>
  </a>
</template>

<style scoped>
.dl-row {
  display: block;
  padding: var(--sos-space-2) var(--sos-space-4);
  text-decoration: none;
  color: inherit;
  border-top: 1px solid var(--sos-border-subtle);
  transition: background 0.13s ease;
}
.dl-row:first-child {
  border-top: none;
}
.dl-row:hover {
  background: var(--sos-accent-soft);
}
.dl-row__line {
  display: flex;
  align-items: baseline;
  gap: var(--sos-space-4);
}
.dl-row__title {
  flex: 1;
  min-width: 0;
  font-size: var(--sos-text-sm);
  font-weight: 600;
  line-height: var(--sos-leading-snug);
  color: var(--sos-text-primary);
  overflow-wrap: anywhere;
}
.dl-row:hover .dl-row__title {
  color: var(--sos-accent);
}
.dl-row__path {
  display: block;
  margin-bottom: 2px;
  font-size: var(--sos-text-2xs);
  font-weight: 600;
  color: var(--sos-accent);
}
.dl-row__desc {
  display: block;
  margin-top: 2px;
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  line-height: var(--sos-leading-snug);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 右侧：更新日期 + 跳转箭头 */
.dl-row__meta {
  flex: none;
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  font-variant-numeric: var(--sos-numeric-tabular);
  white-space: nowrap;
}
.dl-row__date {
  min-width: 5em;
  text-align: right;
}
.dl-row__go {
  width: 13px;
  height: 13px;
  flex: none;
  color: var(--sos-text-disabled);
}
.dl-row:hover .dl-row__go {
  color: var(--sos-accent);
}

@media (max-width: 560px) {
  .dl-row__line {
    gap: var(--sos-space-3);
  }
  .dl-row__date {
    min-width: 0;
  }
}
</style>
