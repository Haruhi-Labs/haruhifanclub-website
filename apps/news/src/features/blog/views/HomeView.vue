<template>
  <div class="news-home-flow">
    <section v-if="viewType === 'home'" class="news-hero">
      <header class="sos-page-header news-page-header">
        <div class="sos-page-header__content">
          <p class="sos-eyebrow sos-page-header__eyebrow">News Desk</p>
          <h1 class="sos-page-header__title">春日团报</h1>
          <p class="sos-page-header__copy">
            记录社团项目、活动通知和成员投稿。标题、摘要、日期和来源优先，明黄色只做置顶和重点线索。
          </p>
          <p class="sos-page-header__meta">
            {{ filteredArticles.length }} 篇文章 · {{ store.allTags.length }} 个标签 · 发布时间倒序
          </p>
        </div>
        <div class="sos-page-header__actions">
          <button
            class="sos-button sos-button--secondary"
            type="button"
            @click="store.toggleSearch"
          >
            搜索团报
          </button>
          <router-link class="sos-button sos-button--primary" to="/submit">我要投稿</router-link>
        </div>
      </header>

      <div class="sos-toolbar sos-toolbar--surface news-toolbar">
        <div class="sos-toolbar__group">
          <span class="sos-badge sos-badge--signal">置顶优先</span>
          <span class="sos-badge sos-badge--outline">双栏阅读流</span>
          <span class="sos-badge sos-badge--outline">真实日期</span>
        </div>
        <div class="sos-toolbar__group news-toolbar__tags">
          <button
            v-for="t in store.allTags.slice(0, 5)"
            :key="t"
            class="news-chip"
            type="button"
            @click="$router.push(`/tag/${t}`)"
          >
            #{{ t }}
          </button>
        </div>
      </div>
    </section>

    <section
      v-else-if="viewType === 'author'"
      class="news-context-header news-context-header--author"
    >
      <img
        :src="getAvatarUrl(route.params.author)"
        :alt="`${route.params.author} 头像`"
        class="author-avatar"
      />
      <div>
        <p class="sos-eyebrow">Author</p>
        <h1>{{ route.params.author }}</h1>
        <p>{{ filteredArticles.length }} 篇文章 · 默认作者为凉宫春日应援团</p>
      </div>
    </section>

    <section v-else-if="viewType === 'search'" class="news-context-header">
      <div>
        <p class="sos-eyebrow">Search</p>
        <h1>搜索结果：“{{ store.searchQuery }}”</h1>
        <p>{{ filteredArticles.length }} 篇文章匹配当前关键词</p>
      </div>
    </section>

    <section v-else :class="headerClass" class="news-context-header">
      <div>
        <p class="sos-eyebrow">{{ viewType === 'tag' ? 'Tag' : 'Participant' }}</p>
        <h1>{{ headerTitle }}</h1>
        <p>{{ filteredArticles.length }} 篇相关文章</p>
      </div>
    </section>

    <div class="content-columns">
      <div class="column-left">
        <NewsCard
          v-for="article in leftCol"
          :key="article.id"
          :article="article"
          class="card-overlap"
          @click="store.openModal(article)"
        />
      </div>

      <div class="column-right">
        <NewsCard
          v-for="article in rightCol"
          :key="article.id"
          :article="article"
          class="card-overlap"
          @click="store.openModal(article)"
        />
      </div>
    </div>

    <nav class="pagination-bar" aria-label="团报分页">
      <div class="sort-label">
        <span>发布时间倒序</span>
        <span>{{ pageNum }} / {{ totalPages }}</span>
      </div>
      <div class="page-buttons">
        <button
          v-for="p in visiblePages"
          :key="p"
          @click="goPage(p)"
          :aria-current="pageNum === p ? 'page' : undefined"
          :class="{
            'pagination-active': pageNum === p,
            'page-inactive': pageNum !== p,
          }"
          class="page-btn"
        >
          <span v-if="pageNum === p">第 {{ p }} 页</span>
          <span v-else>{{ p }}</span>
        </button>
      </div>
    </nav>

    <section v-if="viewType === 'home'" class="tags-footer">
      <div class="tags-label">热门标签</div>
      <div class="tags-list">
        <button
          v-for="t in store.allTags"
          :key="t"
          @click="$router.push(`/tag/${t}`)"
          class="tag-item"
          type="button"
        >
          #{{ t }}
        </button>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useMainStore } from '@/stores/main'
import { buildMasonryPages } from '@/utils/masonry'
import NewsCard from '@/features/blog/components/NewsCard.vue'

const route = useRoute()
const store = useMainStore()

// 当前页码（从 1 开始）
const pageNum = ref(1)

// 判断视图类型
const viewType = computed(() => {
  if (route.name === 'tag') return 'tag'
  if (route.name === 'participant') return 'participant'
  if (route.name === 'author') return 'author'
  if (route.name === 'search') return 'search'
  return 'home'
})

// 获取头像 URL 的辅助函数
const getAvatarUrl = (authorName) => {
  const seed = authorName || 'default'
  return `https://api.dicebear.com/7.x/notionists/svg?seed=${seed}&backgroundColor=c0aede`
}

// 动态 Header 内容 (用于非作者页的普通Header)
const headerTitle = computed(() => {
  if (viewType.value === 'tag') return `# ${route.params.tag}`
  if (viewType.value === 'participant') return route.params.name
  return ''
})

// 动态 Header 样式
const headerClass = computed(() => {
  if (viewType.value === 'participant') return 'header-participant'
  return 'header-default'
})

// 排序逻辑（置顶优先）
const sortArticles = (list) => {
  return [...list].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1
    if (!a.isPinned && b.isPinned) return 1
    if (a.isPinned && b.isPinned) return (a.pinOrder || 0) - (b.pinOrder || 0)
    return 0
  })
}

// 过滤文章（标签 / 参与者 / 作者 / 搜索）
const filteredArticles = computed(() => {
  let list = store.allArticles

  if (viewType.value === 'tag') {
    list = list.filter((a) => a.tags && a.tags.includes(route.params.tag))
  } else if (viewType.value === 'participant') {
    list = list.filter(
      (a) => a.type === 'news' && a.participants?.some((p) => p.name === route.params.name)
    )
  } else if (viewType.value === 'author') {
    // --- 新增作者筛选逻辑 (已修改) ---
    // 目标作者
    const targetAuthor = route.params.author
    const defaultName = '凉宫春日应援团'

    list = list.filter((a) => {
      // 如果文章没有作者，则视为默认作者
      const articleAuthor = a.author || defaultName
      return articleAuthor === targetAuthor
    })
  } else if (viewType.value === 'search') {
    const q = store.searchQuery.toLowerCase()
    if (q) {
      list = list.filter((a) => {
        const inTitle = a.title.toLowerCase().includes(q)
        const inAuthor = (a.author || '凉宫春日应援团').toLowerCase().includes(q)
        const inParticipants =
          a.participants && a.participants.some((p) => p.name.toLowerCase().includes(q))
        return inTitle || inAuthor || inParticipants
      })
    }
  }

  return sortArticles(list)
})

// 用"高度 + 瀑布流"把所有文章拆成多页
const masonryPages = computed(() => {
  const list = filteredArticles.value
  if (!list || list.length === 0) return [{ left: [], right: [] }]

  return buildMasonryPages(list, {
    firstPageLeftOffset: 0,
    pageTargetHeight: 1300,
  })
})

const totalPages = computed(() => masonryPages.value.length)

const currentPage = computed(() => {
  const idx = Math.max(0, Math.min(pageNum.value - 1, totalPages.value - 1))
  return masonryPages.value[idx] || { left: [], right: [] }
})

const leftCol = computed(() => currentPage.value.left)
const rightCol = computed(() => currentPage.value.right)

const visiblePages = computed(() => {
  const p = pageNum.value
  const total = totalPages.value
  const pages = []

  if (total <= 5) {
    for (let i = 1; i <= total; i++) pages.push(i)
  } else {
    if (p <= 3) return [1, 2, 3, 4, 5]
    if (p >= total - 2) return [total - 4, total - 3, total - 2, total - 1, total]
    return [p - 2, p - 1, p, p + 1, p + 2]
  }

  return pages
})

const scrollToTop = () => window.scrollTo({ top: 0, behavior: 'smooth' })

const goPage = (page) => {
  pageNum.value = page
  scrollToTop()
}

// 路由变化时：重置页码
watch(
  () => route.path,
  () => {
    pageNum.value = 1
    scrollToTop()
    // 注意：这里不再清空 searchQuery，改为在 onMounted 中处理，或由 NavBar 控制
  }
)

// 增加 onMounted 钩子：组件挂载时如果不是搜索页，确保清空残留的搜索词
onMounted(() => {
  if (route.name !== 'search') {
    store.searchQuery = ''
  }
})

watch(
  () => totalPages.value,
  (tp) => {
    if (pageNum.value > tp) pageNum.value = tp || 1
  }
)
</script>

<style scoped>
.news-home-flow {
  display: grid;
  gap: var(--sos-space-8);
}

.news-hero,
.news-context-header {
  border: 1px solid var(--sos-border-strong);
  border-radius: var(--sos-radius-sm);
  background: var(--sos-bg-surface);
  box-shadow: var(--sos-shadow-hairline);
}

.news-hero {
  display: grid;
  gap: var(--sos-space-5);
  padding: var(--sos-space-6);
}

.news-page-header {
  padding-block: 0;
}

.news-toolbar {
  border-color: var(--sos-border-default);
  border-radius: var(--sos-radius-sm);
}

.news-toolbar__tags {
  justify-content: flex-end;
}

.news-chip,
.tag-item,
.page-btn {
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-surface);
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-xs);
  font-weight: 800;
  line-height: 1;
  transition:
    background-color var(--sos-duration-base) var(--sos-ease-standard),
    border-color var(--sos-duration-base) var(--sos-ease-standard),
    color var(--sos-duration-base) var(--sos-ease-standard),
    transform var(--sos-duration-fast) var(--sos-ease-out);
}

.news-chip,
.tag-item {
  padding: 0.45rem 0.7rem;
}

.news-chip:hover,
.tag-item:hover,
.page-btn:hover {
  border-color: var(--sos-ink-950);
  color: var(--sos-text-primary);
  transform: translateY(-1px);
}

.news-context-header {
  display: flex;
  align-items: center;
  gap: var(--sos-space-5);
  padding: var(--sos-space-6);
}

.news-context-header h1,
.news-context-header p {
  margin: 0;
}

.news-context-header h1 {
  margin-top: var(--sos-space-2);
  color: var(--sos-text-primary);
  font-family: var(--sos-display-family);
  font-size: var(--sos-text-3xl);
  font-weight: 850;
  line-height: 1.1;
}

.news-context-header p:last-child {
  margin-top: var(--sos-space-2);
  color: var(--sos-text-secondary);
}

.header-participant {
  border-left: 6px solid var(--sos-signal);
}

.author-avatar {
  width: 4.5rem;
  height: 4.5rem;
  flex: 0 0 auto;
  overflow: hidden;
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-subtle);
}

.content-columns {
  display: grid;
  grid-template-columns: 1fr;
  align-items: start;
  gap: 0;
}

@media (min-width: 768px) {
  .content-columns {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
}

.column-left,
.column-right {
  display: grid;
  gap: 0;
  min-width: 0;
}

@media (min-width: 768px) {
  .column-right {
    margin-left: -1px;
  }
}

.card-overlap {
  margin-top: -1px;
}

.pagination-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  border-top: 1px solid var(--sos-border-default);
  padding-top: var(--sos-space-5);
}

.sort-label {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--sos-space-2);
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  font-weight: 750;
}

.sort-label span:last-child {
  color: var(--sos-text-primary);
  font-variant-numeric: tabular-nums;
}

.page-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}

.page-btn {
  min-height: 2.25rem;
  padding: 0 var(--sos-space-3);
}

.pagination-active {
  border-color: var(--sos-signal);
  background: var(--sos-signal);
  color: var(--sos-ink-950);
}

.page-inactive {
  color: var(--sos-text-tertiary);
}

.tags-footer {
  display: grid;
  gap: var(--sos-space-3);
  border-top: 1px solid var(--sos-border-subtle);
  padding-top: var(--sos-space-5);
}

.tags-label {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-xs);
  font-weight: 800;
  text-transform: uppercase;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}

@media (max-width: 767px) {
  .news-hero,
  .news-context-header {
    padding: var(--sos-space-5);
  }

  .news-context-header {
    align-items: flex-start;
  }

  .news-context-header h1 {
    font-size: var(--sos-text-2xl);
  }
}
</style>
