<template>
  <article
    class="news-card"
    role="button"
    tabindex="0"
    @click="$emit('click')"
    @keydown.enter.self="$emit('click')"
  >
    <div class="news-card__body">
      <div class="card-kicker-row">
        <span class="news-label">{{ article.type === 'news' ? 'NEWS' : 'POST' }}</span>
        <span v-if="article.isPinned" class="news-pin">置顶</span>
      </div>

      <div v-if="article.image" class="image-container">
        <img :src="article.image" :alt="article.title" class="card-image" />
      </div>

      <h2 class="card-title" v-html="highlight(article.title)"></h2>
      <div v-if="article.subtitle" class="card-subtitle" v-html="highlight(article.subtitle)"></div>

      <div
        v-if="article.type === 'news' && article.participants && article.participants.length"
        class="participants-box"
      >
        <div v-for="(p, idx) in article.participants" :key="idx" class="participant-row">
          <button
            class="participant-name"
            type="button"
            @click.stop="$router.push(`/participant/${p.name}`)"
          >
            {{ p.name }}
          </button>
          <span class="participant-detail"> — {{ p.role }} ({{ p.project }})</span>
        </div>
      </div>

      <div class="card-summary" v-html="highlight(previewText)"></div>
    </div>

    <footer class="card-footer">
      <div class="tags-container">
        <button
          v-for="tag in (article.tags || []).slice(0, 3)"
          :key="tag"
          type="button"
          @click.stop="$router.push(`/tag/${tag}`)"
          class="tag-item"
        >
          #{{ tag }}
        </button>
      </div>

      <div class="meta-info">
        <span v-if="article.type !== 'news'" class="author-section">
          作者：<button
            class="author-name"
            type="button"
            @click.stop="$router.push(`/author/${article.author || '凉宫春日应援团'}`)"
            v-html="highlight(article.author || '凉宫春日应援团')"
          ></button>
          <span class="meta-separator">·</span>
        </span>
        <time class="date-text">{{ article.date }}</time>
      </div>
    </footer>
  </article>
</template>

<script setup>
import { computed } from 'vue'
import { useMainStore } from '@/stores/main'

const props = defineProps(['article'])
const store = useMainStore()

// 直接使用后端生成的 preview 文本
const previewText = computed(() => {
  if (!props.article) return ''
  return props.article.preview || props.article.summary || ''
})

const escapeHtml = (str) => {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

// 解析 Markdown 样式
const parseInlineStyles = (html) => {
  return html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s>$1</s>')
    .replace(
      /\[([^\]]+)\]\(([^)]+)\)/g,
      '<a href="$2" target="_blank" class="inline-link" onclick="event.stopPropagation()">$1</a>'
    )
}

const highlight = (text) => {
  if (!text) return ''

  let escaped = escapeHtml(text.toString())
  escaped = parseInlineStyles(escaped)
  escaped = escaped.replace(/\n/g, '<br>')

  if (!store.searchQuery || store.searchQuery.trim() === '') {
    return escaped
  }

  const query = store.searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${query})`, 'gi')
  return escaped.replace(regex, '<span class="highlight-text">$1</span>')
}
</script>

<style scoped>
/* 团报原生卡片：纯白、无边框、hover 柔和投影、无圆角（与接入设计系统前一致）。 */
.news-card {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  transition:
    box-shadow 300ms ease,
    transform 200ms ease;
}

.news-card:hover,
.news-card:focus-visible {
  box-shadow:
    0 10px 15px rgba(0, 0, 0, 0.1),
    0 4px 6px rgba(0, 0, 0, 0.05);
}

.news-card__body {
  display: grid;
  align-content: start;
  gap: var(--sos-space-3);
  padding: var(--sos-space-5);
}

.card-kicker-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-2);
}

/* NEWS 标签：1px 黑色描边小框 + 黑色大写文字（与接入前一致）。 */
.news-label {
  display: inline-block;
  width: fit-content;
  border: 1px solid #000000;
  padding: 0.05rem 0.3rem;
  font-size: var(--sos-text-xs);
  font-weight: 700;
  color: #000000;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.news-pin {
  font-size: var(--sos-text-2xs);
  font-weight: 800;
  letter-spacing: 0.04em;
  color: var(--sos-amber-700);
}

.image-container {
  display: block;
  overflow: hidden;
  aspect-ratio: 4 / 3;
  margin-top: var(--sos-space-1);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-xs);
}

.card-image {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: cover;
  transition: transform var(--sos-duration-slow) var(--sos-ease-out);
}

.news-card:hover .card-image {
  transform: scale(1.03);
}

.card-title {
  margin: 0;
  color: var(--sos-text-primary);
  font-family: var(--sos-display-family);
  font-size: var(--sos-text-xl);
  font-weight: 850;
  line-height: 1.22;
  letter-spacing: 0;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  overflow: hidden;
}

.card-subtitle {
  margin: 0;
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  font-weight: 800;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.participants-box {
  display: grid;
  gap: var(--sos-space-2);
  border-left: 3px solid var(--sos-signal);
  background: var(--sos-bg-subtle);
  padding: var(--sos-space-3);
  font-size: var(--sos-text-xs);
  line-height: 1.45;
}

.participant-row {
  min-width: 0;
}

.participant-name,
.author-name {
  border: 0;
  background: transparent;
  padding: 0;
  color: var(--sos-text-primary);
  font: inherit;
  font-weight: 850;
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, var(--sos-signal) 80%, transparent);
  text-decoration-thickness: 0.18em;
  text-underline-offset: 0.18em;
}

.participant-detail {
  color: var(--sos-text-secondary);
}

.card-summary {
  margin: 0;
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  line-height: 1.72;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 6;
}

.card-footer {
  margin-top: auto;
  padding: var(--sos-space-4) var(--sos-space-5);
  flex-wrap: wrap;
  align-items: center;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}

.tag-item {
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  padding: 0.35rem 0.55rem;
  font-size: var(--sos-text-2xs);
  font-weight: 850;
  line-height: 1;
  transition:
    border-color var(--sos-duration-base) var(--sos-ease-standard),
    color var(--sos-duration-base) var(--sos-ease-standard),
    background-color var(--sos-duration-base) var(--sos-ease-standard);
}

.tag-item:hover {
  border-color: var(--sos-ink-950);
  background: var(--sos-ink-950);
  color: var(--sos-white);
}

.meta-info,
.author-section {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-1);
}

.meta-info {
  margin-left: auto;
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.meta-separator {
  color: var(--sos-border-strong);
  font-weight: 800;
}

:deep(.highlight-text) {
  border-radius: var(--sos-radius-xs);
  background: var(--sos-signal);
  color: var(--sos-ink-950);
  padding: 0 2px;
}

:deep(.inline-link) {
  color: var(--sos-link);
  text-decoration: underline;
}

:deep(b) {
  font-weight: 850;
}

:deep(i) {
  font-style: italic;
}

:deep(u) {
  text-decoration: underline;
}

:deep(s) {
  text-decoration: line-through;
}
</style>
