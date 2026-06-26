<template>
  <article
    class="sos-card sos-article-card sos-card--interactive news-card"
    role="button"
    tabindex="0"
    @click="$emit('click')"
    @keydown.enter.self="$emit('click')"
  >
    <div v-if="article.image" class="sos-card__media sos-article-card__media">
      <img :src="article.image" :alt="article.title" />
    </div>

    <div class="sos-card__body">
      <div class="sos-article-card__head">
        <span class="sos-article-card__label">{{ article.type === 'news' ? 'NEWS' : 'POST' }}</span>
        <span v-if="article.isPinned" class="sos-badge sos-badge--signal">置顶</span>
      </div>

      <h2 class="sos-card__heading sos-article-card__title" v-html="highlight(article.title)"></h2>
      <div v-if="article.subtitle" class="sos-article-card__subtitle" v-html="highlight(article.subtitle)"></div>

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

      <div class="sos-card__excerpt sos-article-card__excerpt news-card__summary" v-html="highlight(previewText)"></div>

      <footer class="sos-card__footer sos-article-card__footer">
        <div class="sos-card__tags">
          <button
            v-for="tag in (article.tags || []).slice(0, 3)"
            :key="tag"
            type="button"
            @click.stop="$router.push(`/tag/${tag}`)"
            class="sos-card__tag"
          >
            #{{ tag }}
          </button>
        </div>

        <span class="sos-article-card__meta meta-info">
          <span v-if="article.type !== 'news'" class="author-section">
            作者：<button
              class="author-name"
              type="button"
              @click.stop="$router.push(article.authorUserId ? `/author/u${article.authorUserId}` : `/author/${article.author || '凉宫春日应援团'}`)"
              v-html="highlight(article.author || '凉宫春日应援团')"
            ></button>
            <span class="meta-separator">·</span>
          </span>
          <time class="date-text">{{ article.date }}</time>
        </span>
      </footer>
    </div>
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
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_match, label, url) => {
      // 协议白名单：拦截 javascript: 等可执行 URL；并补 rel 隔离新标签页的 opener
      const safe = /^(?:https?:|mailto:)/i.test(url) || /^[/#]/.test(url) ? url : '#'
      return `<a href="${safe}" target="_blank" rel="noopener noreferrer" class="inline-link" onclick="event.stopPropagation()">${label}</a>`
    })
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
/* 团报卡基于设计系统 .sos-article-card recipe（顶部信号线 + 墨底白字 NEWS 标签
   + 衬线标题 + 摘要 + 描边）。这里仅补：参与者框、署名/日期、行内高亮、摘要行数。 */

/* 厚卡片：正文摘要多展示几行 */
.news-card__summary {
  -webkit-line-clamp: 4;
  line-height: 1.7;
}

/* 参与者框：信号色左边线 + 弱底，编辑部特色 */
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
  cursor: pointer;
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, var(--sos-signal) 80%, transparent);
  text-decoration-thickness: 0.18em;
  text-underline-offset: 0.18em;
}

.participant-detail {
  color: var(--sos-text-secondary);
}

/* 署名 / 日期 */
.meta-info,
.author-section {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-1);
}
.meta-info {
  margin-left: auto;
  color: var(--sos-text-tertiary);
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
