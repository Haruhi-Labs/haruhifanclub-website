<template>
  <div class="news-home-flow">
    <section
      v-if="viewType === 'author'"
      class="news-context-header news-context-header--author"
    >
      <img
        :src="getAvatarUrl(authorDisplayName)"
        :alt="`${authorDisplayName} 头像`"
        class="author-avatar"
      />
      <div>
        <p class="sos-eyebrow">作者</p>
        <h1>{{ authorDisplayName }}</h1>
        <p>{{ filteredArticles.length }} 篇文章</p>
      </div>
    </section>

    <section v-else-if="viewType === 'search'" class="news-context-header">
      <div>
        <h1>搜索结果: "{{ store.searchQuery }}"</h1>
        <p>{{ filteredArticles.length }} 篇文章</p>
      </div>
    </section>

    <section v-else-if="viewType !== 'home'" :class="headerClass" class="news-context-header">
      <div>
        <h1>{{ headerTitle }}</h1>
        <p>{{ filteredArticles.length }} 篇相关文章</p>
      </div>
    </section>

    <!-- 不等高双列瀑布流：保留宝贵的 Grid Lanes 气质，仅去掉重叠、加间距。
         报头在左列单列内，其高度一同计入瀑布流平衡（见 firstPageLeftOffset）。 -->
    <div class="content-columns">
      <div class="column-left">
        <div v-if="viewType === 'home'" class="home-banner">
          <div class="banner-bg">
            <div class="banner-radial-gradient"></div>
            <svg class="banner-noise-svg">
              <filter id="noiseFilter">
                <feTurbulence
                  type="fractalNoise"
                  baseFrequency="0.8"
                  numOctaves="3"
                  stitchTiles="stitch"
                />
              </filter>
              <rect width="100%" height="100%" filter="url(#noiseFilter)" />
            </svg>
          </div>
          <div class="banner-logo-wrapper">
            <img :src="bannerLogo" alt="春日团报 Logo" class="banner-logo-img" />
          </div>
        </div>

        <NewsCard
          v-for="article in leftCol"
          :key="article.id"
          :article="article"
          @click="store.openModal(article)"
        />
      </div>
      <div class="column-right">
        <NewsCard
          v-for="article in rightCol"
          :key="article.id"
          :article="article"
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
          class="tag-item"
          type="button"
          @click="$router.push(`/tag/${t}`)"
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

// public 资源需带应用 base（/news/），否则绝对路径 /春日团报白.png 会 404
const bannerLogo = `${import.meta.env.BASE_URL}春日团报白.png`

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
  return `https://api.dicebear.com/7.x/notionists/svg?seed=${encodeURIComponent(seed)}&backgroundColor=c0aede`
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
    const targetAuthor = route.params.author
    const defaultName = '凉宫春日应援团'
    // 身份关联：param 形如 u{id} → 按发布者账号(authorUserId)归属（仅投稿，新闻除外）；
    // 无 author_user_id 的旧文章 / 新闻 → 回退按作者字符串匹配（保持原逻辑）。
    const idMatch = /^u(\d+)$/.exec(targetAuthor || '')
    if (idMatch) {
      const uid = Number(idMatch[1])
      list = list.filter((a) => a.authorUserId === uid && a.type !== 'news')
    } else {
      list = list.filter((a) => (a.author || defaultName) === targetAuthor)
    }
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

// 作者页标题/头像显示名：身份模式(param=u{id})取匹配文章的作者(随昵称同步)，否则取字符串本身
const authorDisplayName = computed(() => {
  const p = route.params.author || ''
  // 空态（该作者暂无可见的非 news 文章）回退到占位，避免再次暴露 u{id} 形态
  if (/^u\d+$/.test(p)) return filteredArticles.value[0]?.author || '该作者'
  return p
})

// 不等高双列瀑布流（CSS Grid Lanes 式）：按高度把文章分配到左右两列，保留这一宝贵气质；
// 仅去掉卡片重叠、改为有间距，让有厚度的卡片各自完整呈现。
// 报头在左列：高度（约 10rem + 列内间距）一同计入，让两列高度平衡
const BANNER_OFFSET = 184
const masonryPages = computed(() => {
  const list = filteredArticles.value
  if (!list || list.length === 0) return [{ left: [], right: [] }]
  return buildMasonryPages(list, {
    firstPageLeftOffset: viewType.value === 'home' ? BANNER_OFFSET : 0,
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
    // 非搜索页清空残留搜索词，避免组件复用时 NewsCard.highlight() 继续高亮旧关键词
    if (route.name !== 'search') store.searchQuery = ''
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

.news-context-header {
  border: 1px solid var(--sos-border-strong);
  border-radius: var(--sos-radius-sm);
  background: var(--sos-bg-surface);
  box-shadow: var(--sos-shadow-hairline);
}

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

.tag-item {
  padding: 0.45rem 0.7rem;
}

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

.home-banner {
  position: relative;
  display: flex;
  width: 100%;
  height: 10rem;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 1px solid var(--sos-border-strong);
  border-radius: var(--sos-radius-sm);
  background: var(--sos-ink-950);
  box-shadow: var(--sos-shadow-hairline);
}

.banner-bg,
.banner-radial-gradient,
.banner-noise-svg {
  position: absolute;
  inset: 0;
}

.banner-bg {
  background: var(--sos-text-primary);
}

.banner-radial-gradient {
  opacity: 0.2;
  background: radial-gradient(circle at center, var(--sos-text-secondary), var(--sos-text-primary), var(--sos-text-primary));
}

.banner-noise-svg {
  width: 100%;
  height: 100%;
  opacity: 0.2;
  pointer-events: none;
  mix-blend-mode: overlay;
}

.banner-logo-wrapper {
  position: relative;
  z-index: 1;
  display: flex;
  width: 100%;
  height: 100%;
  align-items: center;
  justify-content: center;
  padding: var(--sos-space-4);
}

.banner-logo-img {
  width: auto;
  height: 100%;
  max-width: 100%;
  object-fit: contain;
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.5));
}

/* 不等高双列瀑布流（Grid Lanes）：按高度分配到左右两列，保留这一宝贵气质。
   去掉旧的 card-overlap(-1px) 密集重叠，改为列间 + 列内卡片都有间距，
   让有厚度的 recipe 卡片各自完整、不挤压。 */
.content-columns {
  display: grid;
  grid-template-columns: 1fr;
  align-items: start;
  gap: var(--sos-space-6);
}

@media (min-width: 768px) {
  .content-columns {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
}

.column-left,
.column-right {
  display: grid;
  align-content: start;
  gap: var(--sos-space-6);
  min-width: 0;
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
