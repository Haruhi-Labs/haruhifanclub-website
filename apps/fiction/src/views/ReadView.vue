<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import DOMPurify from 'dompurify'
import CommentSection from '@/components/CommentSection.vue'
import { getChapter, saveProgress, session } from '@/api'
import { useReaderSettings, THEMES, themeOf, widthOf } from '@/lib/reader'
import { wordLabel } from '@/lib/format'

const route = useRoute()
const router = useRouter()
const s = useReaderSettings()

const storyId = computed(() => Number(route.params.id))
const chapterId = computed(() => Number(route.params.cid))

const loading = ref(true)
const notFound = ref(false)
const story = ref(null)
const chapter = ref(null)
const prev = ref(null)
const next = ref(null)
const toc = ref([])
const progress = ref(0)
const showToc = ref(false)
const showSettings = ref(false)

const safeHtml = computed(() =>
  chapter.value ? DOMPurify.sanitize(chapter.value.contentHtml || '') : '',
)
// 当前章在目录中的序号（供顶栏「第 N/总 章」显示）
const chapterIndex = computed(() => {
  const i = toc.value.findIndex((c) => c.id === chapterId.value)
  return i >= 0 ? i + 1 : 0
})
const theme = computed(() => themeOf(s.theme))
const rootStyle = computed(() => ({
  '--r-bg': theme.value.bg,
  '--r-text': theme.value.text,
  '--r-panel': theme.value.panel,
  '--r-width': widthOf(s.width),
  '--r-size': `${s.fontSize}px`,
  '--r-leading': s.lineHeight,
  '--r-font': s.fontFamily === 'serif' ? 'var(--sos-font-reading)' : 'var(--sos-font-sans)',
}))

async function load() {
  loading.value = true
  notFound.value = false
  showToc.value = false
  try {
    const r = await getChapter(storyId.value, chapterId.value)
    story.value = r.story
    chapter.value = r.chapter
    prev.value = r.prev
    next.value = r.next
    toc.value = r.chapters
    await nextTick()
    window.scrollTo({ top: 0 })
  } catch (e) {
    if (e.status === 404) notFound.value = true
  } finally {
    loading.value = false
  }
}

function go(target) {
  if (target) router.push(`/story/${storyId.value}/chapter/${target.id}`)
}

// 阅读进度 + 保存（登录时，节流 4s）
let saveTimer = null
function onScroll() {
  const doc = document.documentElement
  const max = doc.scrollHeight - doc.clientHeight
  const frac = max > 0 ? Math.min(1, doc.scrollTop / max) : 0
  progress.value = Math.round(frac * 100)
  if (session.state.user && !saveTimer) {
    saveTimer = setTimeout(() => {
      saveTimer = null
      saveProgress(storyId.value, { chapterId: chapterId.value, progress: frac }).catch(() => {})
    }, 4000)
  }
}

function onKey(e) {
  if (e.target.tagName === 'TEXTAREA' || e.target.tagName === 'INPUT') return
  if (e.key === 'ArrowLeft' && prev.value) go(prev.value)
  if (e.key === 'ArrowRight' && next.value) go(next.value)
}

onMounted(() => {
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('keydown', onKey)
})
onBeforeUnmount(() => {
  window.removeEventListener('scroll', onScroll)
  window.removeEventListener('keydown', onKey)
  if (saveTimer) clearTimeout(saveTimer)
})

watch([storyId, chapterId], load, { immediate: true })
</script>

<template>
  <div class="reader" :style="rootStyle">
    <div class="reader__progress" :style="{ width: progress + '%' }"></div>

    <header class="reader__bar">
      <RouterLink :to="`/story/${storyId}`" class="reader__iconbtn" title="返回作品">‹ 返回</RouterLink>
      <div class="reader__crumb">
        <span class="reader__crumb-title">{{ story?.title }}</span>
        <span v-if="chapterIndex" class="reader__crumb-sub">
          第 {{ chapterIndex }}/{{ toc.length }} 章 · 已读 {{ progress }}%
        </span>
      </div>
      <div class="reader__bar-right">
        <button class="reader__barbtn" title="章节目录" @click="showToc = true">
          <span class="reader__barbtn-ico">☰</span><span class="reader__barbtn-txt">目录</span>
        </button>
        <button
          class="reader__barbtn"
          :class="{ on: showSettings }"
          title="阅读设置"
          @click="showSettings = !showSettings"
        >
          <span class="reader__barbtn-ico">Aa</span><span class="reader__barbtn-txt">设置</span>
        </button>
      </div>
    </header>

    <div v-if="loading" class="reader__loading">正在翻开书页…</div>
    <div v-else-if="notFound" class="reader__missing">
      <p>章节不存在或未发布。</p>
      <RouterLink :to="`/story/${storyId}`" class="sos-button sos-button--primary">返回作品</RouterLink>
    </div>

    <template v-else-if="chapter">
      <article class="reader__content">
        <p class="reader__eyebrow">{{ story.title }}</p>
        <h1 class="reader__title">{{ chapter.title }}</h1>
        <p class="reader__meta">{{ story.authorName }} · {{ wordLabel(chapter.wordCount) }}</p>
        <!-- 正文为服务端 ammonia 白名单清洗后的 HTML，前端再经 DOMPurify 兜底 -->
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div class="reader__prose" v-html="safeHtml"></div>
        <div v-if="chapter.authorNote" class="reader__note">
          <strong>作者的话</strong>
          <p>{{ chapter.authorNote }}</p>
        </div>
      </article>

      <nav class="reader__pager">
        <button class="reader__pagerbtn" :disabled="!prev" @click="go(prev)">‹ 上一章</button>
        <RouterLink :to="`/story/${storyId}`" class="reader__pagerbtn reader__pagerbtn--toc">目录</RouterLink>
        <button class="reader__pagerbtn" :disabled="!next" @click="go(next)">下一章 ›</button>
      </nav>

      <div class="reader__comments">
        <CommentSection :story-id="storyId" :chapter-id="chapterId" />
      </div>
    </template>

    <!-- 设置面板 -->
    <div v-if="showSettings" class="reader__overlay" @click="showSettings = false"></div>
    <aside v-if="showSettings" class="reader__settings">
      <h3>阅读设置</h3>
      <div class="reader__set-row">
        <label>字号</label>
        <div class="reader__stepper">
          <button @click="s.fontSize = Math.max(15, s.fontSize - 1)">A-</button>
          <span>{{ s.fontSize }}</span>
          <button @click="s.fontSize = Math.min(26, s.fontSize + 1)">A+</button>
        </div>
      </div>
      <div class="reader__set-row">
        <label>行距</label>
        <div class="reader__stepper">
          <button @click="s.lineHeight = Math.max(1.5, +(s.lineHeight - 0.1).toFixed(1))">紧</button>
          <span>{{ s.lineHeight.toFixed(1) }}</span>
          <button @click="s.lineHeight = Math.min(2.4, +(s.lineHeight + 0.1).toFixed(1))">松</button>
        </div>
      </div>
      <div class="reader__set-row">
        <label>字体</label>
        <div class="reader__seg">
          <button :class="{ on: s.fontFamily === 'serif' }" @click="s.fontFamily = 'serif'">宋体</button>
          <button :class="{ on: s.fontFamily === 'sans' }" @click="s.fontFamily = 'sans'">黑体</button>
        </div>
      </div>
      <div class="reader__set-row">
        <label>栏宽</label>
        <div class="reader__seg">
          <button :class="{ on: s.width === 'narrow' }" @click="s.width = 'narrow'">窄</button>
          <button :class="{ on: s.width === 'normal' }" @click="s.width = 'normal'">中</button>
          <button :class="{ on: s.width === 'wide' }" @click="s.width = 'wide'">宽</button>
        </div>
      </div>
      <div class="reader__set-row">
        <label>背景</label>
        <div class="reader__themes">
          <button
            v-for="t in THEMES"
            :key="t.key"
            class="reader__theme"
            :class="{ on: s.theme === t.key }"
            :style="{ background: t.bg, color: t.text }"
            @click="s.theme = t.key"
          >
            {{ t.label }}
          </button>
        </div>
      </div>
    </aside>

    <!-- 目录抽屉 -->
    <div v-if="showToc" class="reader__overlay" @click="showToc = false"></div>
    <aside v-if="showToc" class="reader__toc">
      <div class="reader__toc-head">
        <h3>目录</h3>
        <button class="reader__iconbtn" @click="showToc = false">✕</button>
      </div>
      <ol>
        <li v-for="(c, i) in toc" :key="c.id">
          <RouterLink
            :to="`/story/${storyId}/chapter/${c.id}`"
            :class="{ on: c.id === chapterId }"
            @click="showToc = false"
          >
            <span class="reader__toc-no">{{ i + 1 }}</span>{{ c.title }}
          </RouterLink>
        </li>
      </ol>
    </aside>
  </div>
</template>

<style scoped>
.reader {
  min-height: 100vh;
  background: var(--r-bg);
  color: var(--r-text);
  transition: background 0.2s ease, color 0.2s ease;
  padding-bottom: var(--sos-space-10);
}
.reader__progress {
  position: fixed;
  top: 0;
  left: 0;
  height: 3px;
  background: var(--sos-accent);
  z-index: 40;
  transition: width 0.15s ease;
}
.reader__bar {
  position: sticky;
  top: 0;
  z-index: 30;
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  padding: var(--sos-space-3) var(--sos-space-5);
  background: color-mix(in srgb, var(--r-panel) 88%, transparent);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid rgba(120, 100, 80, 0.14);
}
.reader__crumb {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  line-height: 1.25;
}
.reader__crumb-title {
  max-width: 100%;
  font-size: var(--sos-text-sm);
  font-weight: 600;
  opacity: 0.85;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.reader__crumb-sub {
  font-size: var(--sos-text-xs);
  opacity: 0.55;
  font-variant-numeric: var(--sos-numeric-tabular);
  white-space: nowrap;
}
.reader__bar-right {
  display: flex;
  gap: var(--sos-space-2);
}
.reader__barbtn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border: 1px solid rgba(120, 100, 80, 0.22);
  background: transparent;
  color: inherit;
  cursor: pointer;
  border-radius: var(--sos-radius-full);
  padding: 5px 13px;
  font-size: var(--sos-text-sm);
}
.reader__barbtn-ico {
  font-size: 0.95em;
  font-weight: 700;
  line-height: 1;
}
.reader__barbtn:hover {
  border-color: var(--sos-accent);
  color: var(--sos-accent);
}
.reader__barbtn.on {
  background: var(--sos-accent);
  border-color: var(--sos-accent);
  color: #fff;
}
@media (max-width: 560px) {
  .reader__barbtn-txt {
    display: none;
  }
}
.reader__iconbtn {
  border: 1px solid rgba(120, 100, 80, 0.22);
  background: transparent;
  color: inherit;
  cursor: pointer;
  border-radius: var(--sos-radius-full);
  padding: 5px 14px;
  font-size: var(--sos-text-sm);
  text-decoration: none;
}
.reader__iconbtn:hover {
  border-color: var(--sos-accent);
  color: var(--sos-accent);
}
.reader__loading,
.reader__missing {
  text-align: center;
  padding: var(--sos-space-12) var(--sos-space-5);
  opacity: 0.8;
}
.reader__content {
  width: min(var(--r-width), 100% - 2 * var(--sos-space-5));
  margin: var(--sos-space-10) auto 0;
}
.reader__eyebrow {
  text-align: center;
  font-size: var(--sos-text-sm);
  opacity: 0.55;
  margin: 0 0 var(--sos-space-2);
}
.reader__title {
  font-family: var(--sos-font-reading);
  font-size: clamp(1.5rem, 4vw, 2rem);
  text-align: center;
  margin: 0 0 var(--sos-space-3);
  line-height: var(--sos-leading-snug);
}
.reader__meta {
  text-align: center;
  opacity: 0.55;
  font-size: var(--sos-text-sm);
  margin: 0 0 var(--sos-space-8);
}
.reader__prose {
  font-family: var(--r-font);
  font-size: var(--r-size);
  line-height: var(--r-leading);
  letter-spacing: 0.01em;
}
.reader__prose :deep(p) {
  margin: 0 0 1.1em;
  text-indent: 2em;
}
.reader__prose :deep(h2),
.reader__prose :deep(h3),
.reader__prose :deep(h4) {
  font-family: var(--sos-font-reading);
  margin: 1.6em 0 0.7em;
  text-align: center;
}
.reader__prose :deep(blockquote) {
  margin: 1.2em 0;
  padding-left: 1em;
  border-left: 3px solid var(--sos-accent);
  opacity: 0.85;
}
.reader__prose :deep(hr) {
  border: none;
  text-align: center;
  margin: 2em 0;
}
.reader__prose :deep(hr)::before {
  content: '❋';
  opacity: 0.5;
}
.reader__prose :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: var(--sos-radius-md);
  display: block;
  margin: 1.2em auto;
}
.reader__prose :deep(figure) {
  margin: 1.8em 0;
  text-align: center;
}
.reader__prose :deep(figure img) {
  margin: 0 auto;
}
.reader__prose :deep(figcaption) {
  margin-top: 0.7em;
  font-size: 0.85em;
  opacity: 0.6;
  text-indent: 0;
}
.reader__prose :deep(mark) {
  background: color-mix(in srgb, var(--sos-signal, #ffd666) 72%, transparent);
  color: inherit;
  border-radius: 2px;
  padding: 0 2px;
}
.reader__prose :deep(pre) {
  text-indent: 0;
  text-align: left;
  background: color-mix(in srgb, var(--r-text) 7%, transparent);
  border-radius: var(--sos-radius-md);
  padding: 1em 1.2em;
  overflow-x: auto;
  font-family: var(--sos-font-mono);
  font-size: 0.9em;
  line-height: 1.6;
  margin: 1.3em 0;
}
.reader__prose :deep(:not(pre) > code) {
  font-family: var(--sos-font-mono);
  font-size: 0.9em;
  background: color-mix(in srgb, var(--r-text) 8%, transparent);
  border-radius: 3px;
  padding: 1px 5px;
}
.reader__prose :deep(pre code) {
  background: none;
  padding: 0;
  font: inherit;
}
.reader__note {
  margin-top: var(--sos-space-8);
  padding: var(--sos-space-4) var(--sos-space-5);
  background: color-mix(in srgb, var(--r-panel) 70%, transparent);
  border-radius: var(--sos-radius-md);
  font-size: var(--sos-text-sm);
}
.reader__note strong {
  color: var(--sos-accent);
}
.reader__note p {
  margin: 6px 0 0;
  white-space: pre-wrap;
  opacity: 0.85;
}
.reader__pager {
  width: min(var(--r-width), 100% - 2 * var(--sos-space-5));
  margin: var(--sos-space-10) auto 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-3);
}
.reader__pagerbtn {
  flex: 1;
  border: 1px solid rgba(120, 100, 80, 0.22);
  background: color-mix(in srgb, var(--r-panel) 60%, transparent);
  color: inherit;
  cursor: pointer;
  border-radius: var(--sos-radius-md);
  padding: var(--sos-space-3);
  font-size: var(--sos-text-sm);
  text-decoration: none;
  text-align: center;
}
.reader__pagerbtn--toc {
  flex: 0 0 auto;
  padding-inline: var(--sos-space-6);
}
.reader__pagerbtn:disabled {
  opacity: 0.4;
  cursor: default;
}
.reader__pagerbtn:not(:disabled):hover {
  border-color: var(--sos-accent);
  color: var(--sos-accent);
}
/* 评论区脱离阅读主题，回到设计系统中性表面，保证任意背景（含夜间）下都清晰可读 */
.reader__comments {
  width: min(48rem, 100% - 2 * var(--sos-space-5));
  margin: var(--sos-space-10) auto 0;
  background: var(--sos-bg-surface);
  color: var(--sos-text-primary);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  padding: var(--sos-space-6);
}

/* 面板与抽屉 */
.reader__overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.28);
  z-index: 45;
}
.reader__settings,
.reader__toc {
  position: fixed;
  z-index: 46;
  background: var(--r-panel);
  color: var(--r-text);
  box-shadow: var(--sos-shadow-overlay);
}
.reader__settings {
  top: 64px;
  right: max(var(--sos-space-5), calc(50vw - 34rem));
  width: 300px;
  border-radius: var(--sos-radius-lg);
  padding: var(--sos-space-5);
}
.reader__settings h3 {
  margin: 0 0 var(--sos-space-4);
  font-size: var(--sos-text-md);
}
.reader__set-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-3);
  margin-bottom: var(--sos-space-4);
}
.reader__set-row > label {
  font-size: var(--sos-text-sm);
  opacity: 0.7;
}
.reader__stepper,
.reader__seg {
  display: flex;
  align-items: center;
  gap: 4px;
}
.reader__stepper button,
.reader__seg button,
.reader__theme {
  border: 1px solid rgba(120, 100, 80, 0.28);
  background: transparent;
  color: inherit;
  cursor: pointer;
  border-radius: var(--sos-radius-sm);
  padding: 5px 12px;
  font-size: var(--sos-text-sm);
}
.reader__stepper span {
  min-width: 2.4em;
  text-align: center;
  font-size: var(--sos-text-sm);
}
.reader__seg button.on {
  background: var(--sos-accent);
  color: #fff;
  border-color: var(--sos-accent);
}
.reader__themes {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.reader__theme {
  font-size: var(--sos-text-xs);
  padding: 6px 10px;
}
.reader__theme.on {
  outline: 2px solid var(--sos-accent);
  outline-offset: 1px;
}
.reader__toc {
  top: 0;
  left: 0;
  height: 100vh;
  width: min(340px, 82vw);
  padding: var(--sos-space-5);
  overflow-y: auto;
}
.reader__toc-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sos-space-4);
}
.reader__toc-head h3 {
  margin: 0;
}
.reader__toc ol {
  list-style: none;
  margin: 0;
  padding: 0;
}
.reader__toc a {
  display: flex;
  gap: var(--sos-space-2);
  padding: var(--sos-space-3);
  border-radius: var(--sos-radius-sm);
  text-decoration: none;
  color: inherit;
  font-size: var(--sos-text-sm);
}
.reader__toc a:hover {
  background: color-mix(in srgb, var(--sos-accent) 12%, transparent);
}
.reader__toc a.on {
  color: var(--sos-accent);
  font-weight: 600;
}
.reader__toc-no {
  opacity: 0.5;
  min-width: 1.8em;
}
</style>
