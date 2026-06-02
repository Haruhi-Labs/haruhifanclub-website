<template>
  <div class="animate-fade-in">

    <!-- ============================================== -->
    <!-- CASE A: 作者页专用头部 -->
    <!-- ============================================== -->
    <div v-if="viewType === 'author'" class="author-header">
        <!-- 装饰背景 -->
        <div class="decorative-overlay">
             <div class="radial-gradient-bg"></div>
        </div>

        <!-- 头像 (占位符) -->
        <div class="author-avatar">
            <!-- 使用封装的函数获取头像 -->
            <img :src="getAvatarUrl(route.params.author)" class="avatar-img">
        </div>

        <!-- 作者名 -->
        <h1 class="author-name">
            <span class="author-label">作者</span>
            <span>{{ route.params.author }}</span>
        </h1>

        <!-- 统计与链接 -->
        <div class="author-stats">
            <div class="stat-item">
                <span class="stat-number">{{ filteredArticles.length }}</span>
                <span>篇文章</span>
            </div>
        </div>
    </div>


    <!-- ============================================== -->
    <!-- CASE B: 标签/参与者/普通页 头部 -->
    <!-- ============================================== -->
    <div
      v-else-if="viewType !== 'home' && viewType !== 'search'"
      :class="headerClass"
      class="section-header"
    >
      <h1
        class="section-title serif-font"
      >
        {{ headerTitle }}
      </h1>
      <p
        class="section-subtitle"
      >
        {{ filteredArticles.length }} 篇相关文章
      </p>
      <div
        class="section-bg-pattern"
      >
        <span
          v-for="n in 20"
          :key="n"
          class="bg-pattern-text"
        >
          {{ headerTitle }}
        </span>
      </div>
    </div>

    <!-- ============================================== -->
    <!-- CASE C: 搜索页 头部 -->
    <!-- ============================================== -->
    <div
      v-else-if="viewType === 'search'"
      class="search-header"
    >
      <h1 class="search-title serif-font">
        搜索结果: "{{ store.searchQuery }}"
      </h1>
      <span class="search-count">
        {{ filteredArticles.length }} 篇文章
      </span>
    </div>

    <!-- 列表内容区 -->
    <div class="content-columns">
      <!-- 左侧列 (包含 Banner) -->
      <div class="column-left">
        <!-- 首页 Banner -->
        <div
          v-if="viewType === 'home'"
          class="home-banner"
        >
          <!-- 背景噪点与光影 -->
          <div class="banner-bg">
            <div
              class="banner-radial-gradient"
            ></div>
            <svg
              class="banner-noise-svg"
            >
              <filter id="noiseFilter">
                <feTurbulence
                  type="fractalNoise"
                  baseFrequency="0.8"
                  numOctaves="3"
                  stitchTiles="stitch"
                />
              </filter>
              <rect
                width="100%"
                height="100%"
                filter="url(#noiseFilter)"
              />
            </svg>
          </div>

          <!-- [修改] Logo 图片区域 -->
          <div class="banner-logo-wrapper">
             <!-- 使用 object-contain 确保 Logo 完整显示且不变形，增加 drop-shadow 提升层次感 -->
             <img src="/春日团报白.png" alt="春日团报 Logo" class="banner-logo-img">
          </div>
        </div>

        <NewsCard
          v-for="article in leftCol"
          :key="article.id"
          :article="article"
          class="card-overlap"
          @click="store.openModal(article)"
        />
      </div>

      <!-- 右侧列 -->
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

    <!-- Pagination -->
    <!-- [修改] 移除了 border-t border-black -->
    <div
      class="pagination-bar"
    >
      <div
        class="sort-label"
      >
        <span>发布时间倒序</span>
      </div>
      <div class="page-buttons">
        <button
          v-for="p in visiblePages"
          :key="p"
          @click="pageNum = p; scrollToTop()"
          :class="{
            'pagination-active': pageNum === p,
            'page-inactive': pageNum !== p
          }"
          class="page-btn"
        >
          <span v-if="pageNum === p">第 {{ p }} 页</span>
          <span v-else>{{ p }}</span>
        </button>
      </div>
    </div>

    <!-- Tags Footer (仅首页显示) -->
    <!-- [修改] 移除了 border-t-4 border-black -->
    <div
      v-if="viewType === 'home'"
      class="tags-footer"
    >
      <div class="tags-label">
        热门标签
      </div>

      <div
        class="tags-list"
      >
        <span
          v-for="t in store.allTags"
          :key="t"
          @click="$router.push(`/tag/${t}`)"
          class="tag-item"
        >
          #{{ t }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useMainStore } from '@/stores/main';
import { buildMasonryPages } from '@/utils/masonry';
import NewsCard from '@/components/NewsCard.vue';

const route = useRoute();
const store = useMainStore();

// 当前页码（从 1 开始）
const pageNum = ref(1);

// 判断视图类型
const viewType = computed(() => {
  if (route.name === 'tag') return 'tag';
  if (route.name === 'participant') return 'participant';
  if (route.name === 'author') return 'author';
  if (route.name === 'search') return 'search';
  return 'home';
});

// 获取头像 URL 的辅助函数
const getAvatarUrl = (authorName) => {
    const seed = authorName || 'default';
     return `https://api.dicebear.com/7.x/notionists/svg?seed=${seed}&backgroundColor=c0aede`;
};

// 动态 Header 内容 (用于非作者页的普通Header)
const headerTitle = computed(() => {
  if (viewType.value === 'tag') return `# ${route.params.tag}`;
  if (viewType.value === 'participant') return route.params.name;
  return '';
});

// 动态 Header 样式
const headerClass = computed(() => {
  if (viewType.value === 'participant') return 'header-participant';
  return 'header-default';
});

// 排序逻辑（置顶优先）
const sortArticles = (list) => {
  return [...list].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1;
    if (!a.isPinned && b.isPinned) return 1;
    if (a.isPinned && b.isPinned) return (a.pinOrder || 0) - (b.pinOrder || 0);
    return 0;
  });
};

// 过滤文章（标签 / 参与者 / 作者 / 搜索）
const filteredArticles = computed(() => {
  let list = store.allArticles;

  if (viewType.value === 'tag') {
    list = list.filter(
      (a) => a.tags && a.tags.includes(route.params.tag)
    );
  } else if (viewType.value === 'participant') {
    list = list.filter(
      (a) =>
        a.type === 'news' &&
        a.participants?.some((p) => p.name === route.params.name)
    );
  } else if (viewType.value === 'author') {
    // --- 新增作者筛选逻辑 (已修改) ---
    // 目标作者
    const targetAuthor = route.params.author;
    const defaultName = '凉宫春日应援团';

    list = list.filter((a) => {
        // 如果文章没有作者，则视为默认作者
        const articleAuthor = a.author || defaultName;
        return articleAuthor === targetAuthor;
    });

  } else if (viewType.value === 'search') {
    const q = store.searchQuery.toLowerCase();
    if (q) {
      list = list.filter((a) => {
        const inTitle = a.title.toLowerCase().includes(q);
        const inAuthor =
          (a.author || '凉宫春日应援团').toLowerCase().includes(q);
        const inParticipants =
          a.participants &&
          a.participants.some((p) =>
            p.name.toLowerCase().includes(q)
          );
        return inTitle || inAuthor || inParticipants;
      });
    }
  }

  return sortArticles(list);
});

// 用"高度 + 瀑布流"把所有文章拆成多页
const masonryPages = computed(() => {
  const list = filteredArticles.value;
  if (!list || list.length === 0)
    return [{ left: [], right: [] }];

  // 首页左侧第一个块被 Banner 占用，所以有 offset
  const firstPageLeftOffset =
    viewType.value === 'home' ? 170 : 0;

  return buildMasonryPages(list, {
    firstPageLeftOffset,
    pageTargetHeight: 1300,
  });
});

const totalPages = computed(
  () => masonryPages.value.length
);

const currentPage = computed(() => {
  const idx = Math.max(
    0,
    Math.min(pageNum.value - 1, totalPages.value - 1)
  );
  return (
    masonryPages.value[idx] || { left: [], right: [] }
  );
});

const leftCol = computed(() => currentPage.value.left);
const rightCol = computed(() => currentPage.value.right);

const visiblePages = computed(() => {
  const p = pageNum.value;
  const total = totalPages.value;
  const pages = [];

  if (total <= 5) {
    for (let i = 1; i <= total; i++) pages.push(i);
  } else {
    if (p <= 3) return [1, 2, 3, 4, 5];
    if (p >= total - 2)
      return [
        total - 4,
        total - 3,
        total - 2,
        total - 1,
        total,
      ];
    return [p - 2, p - 1, p, p + 1, p + 2];
  }

  return pages;
});

const scrollToTop = () =>
  window.scrollTo({ top: 0, behavior: 'smooth' });

// 路由变化时：重置页码
watch(
  () => route.path,
  () => {
    pageNum.value = 1;
    scrollToTop();
    // 注意：这里不再清空 searchQuery，改为在 onMounted 中处理，或由 NavBar 控制
  }
);

// 增加 onMounted 钩子：组件挂载时如果不是搜索页，确保清空残留的搜索词
onMounted(() => {
    if (route.name !== 'search') {
        store.searchQuery = '';
    }
});

watch(
  () => totalPages.value,
  (tp) => {
    if (pageNum.value > tp) pageNum.value = tp || 1;
  }
);
</script>

<style scoped>
/* ============================================= */
/* CASE A: Author Header                         */
/* ============================================= */

.author-header {
  width: 100%;
  background-color: #222;
  color: #fff;
  padding-top: 4rem;
  padding-bottom: 4rem;
  margin-bottom: 3rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  box-shadow: 0 20px 25px rgba(0, 0, 0, 0.1), 0 8px 10px rgba(0, 0, 0, 0.04);
  border-radius: 0.125rem;
}

.decorative-overlay {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  opacity: 0.1;
  pointer-events: none;
}

.radial-gradient-bg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: radial-gradient(circle at center, #6b7280, #222, #222);
}

.author-avatar {
  position: relative;
  z-index: 10;
  width: 7rem;
  height: 7rem;
  margin-bottom: 1.5rem;
  border-radius: 9999px;
  overflow: hidden;
  border: 4px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.25);
  background-color: #4b5563;
}

@media (min-width: 768px) {
  .author-avatar {
    width: 8rem;
    height: 8rem;
  }
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.author-name {
  position: relative;
  z-index: 10;
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
  font-family: "Noto Sans SC", sans-serif;
  margin-bottom: 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

@media (min-width: 768px) {
  .author-name {
    font-size: 2.25rem;
  }
}

.author-label {
  opacity: 0.6;
  font-family: "Noto Serif SC", serif;
  font-style: italic;
  font-size: 1.25rem;
}

.author-stats {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 1.5rem;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  color: rgba(255, 255, 255, 0.8);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 1.5rem;
}

@media (min-width: 768px) {
  .author-stats {
    font-size: 0.875rem;
  }
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.stat-number {
  font-size: 1.25rem;
  font-family: "Noto Serif SC", serif;
  color: #fff;
}

/* ============================================= */
/* CASE B: Section Header (Tag / Participant)    */
/* ============================================= */

.section-header {
  width: 100%;
  padding: 3rem;
  margin-bottom: 2rem;
  position: relative;
  overflow: hidden;
  border: 1px solid #000;
}

.header-default {
  background-color: #222;
  color: #fff;
}

.header-participant {
  background-color: #1e3a8a;
  color: #fff;
}

.section-title {
  font-size: 2.25rem;
  font-weight: 900;
  position: relative;
  z-index: 10;
  text-align: center;
}

@media (min-width: 768px) {
  .section-title {
    font-size: 3.75rem;
  }
}

.section-subtitle {
  text-align: center;
  margin-top: 0.5rem;
  position: relative;
  z-index: 10;
  font-family: "Noto Serif SC", serif;
  font-style: italic;
  opacity: 0.8;
  color: #fff;
}

.section-bg-pattern {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  opacity: 0.1;
  pointer-events: none;
  display: flex;
  flex-wrap: wrap;
  align-content: center;
  justify-content: center;
  gap: 1rem;
  transform: rotate(12deg) scale(1.5);
}

.bg-pattern-text {
  font-size: 2.25rem;
  font-family: "Noto Serif SC", serif;
  color: #fff;
}

/* ============================================= */
/* CASE C: Search Header                         */
/* ============================================= */

.search-header {
  border-bottom: 2px solid #000;
  padding-bottom: 1rem;
  margin-bottom: 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.search-title {
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
}

.search-count {
  color: #6b7280;
  font-family: "Noto Serif SC", serif;
  font-style: italic;
}

/* ============================================= */
/* Content Columns (Masonry Layout)              */
/* ============================================= */

.content-columns {
  display: flex;
  flex-direction: column;
  gap: 0;
  align-items: flex-start;
  margin-bottom: 3rem;
}

@media (min-width: 768px) {
  .content-columns {
    flex-direction: row;
  }
}

.column-left {
  width: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 10;
}

@media (min-width: 768px) {
  .column-left {
    width: 50%;
  }
}

.column-right {
  width: 100%;
  display: flex;
  flex-direction: column;
}

@media (min-width: 768px) {
  .column-right {
    width: 50%;
    margin-left: -1px;
  }
}

.card-overlap {
  margin-top: -1px;
}

/* ============================================= */
/* Home Banner                                   */
/* ============================================= */

.home-banner {
  width: 100%;
  position: relative;
  overflow: hidden;
  cursor: default;
  height: 10rem;
  background-color: #1a1a1a;
  border-style: double;
  border-color: #4a4a4a;
  box-shadow: 0 10px 15px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
}

@media (min-width: 768px) {
  .home-banner {
    height: 12rem;
  }
}

.banner-bg {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background-color: #171717;
}

.banner-radial-gradient {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  opacity: 0.2;
  background: radial-gradient(circle at center, #374151, #000, #000);
}

.banner-noise-svg {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0.2;
  pointer-events: none;
  mix-blend-mode: overlay;
}

.banner-logo-wrapper {
  position: relative;
  z-index: 10;
  width: 100%;
  height: 100%;
  padding: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.banner-logo-img {
  width: auto;
  height: 100%;
  max-width: 100%;
  object-fit: contain;
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.5));
}

/* ============================================= */
/* Pagination                                    */
/* ============================================= */

.pagination-bar {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  padding-top: 1rem;
  margin-bottom: 4rem;
  font-size: 0.875rem;
  font-family: "Noto Serif SC", serif;
}

@media (min-width: 768px) {
  .pagination-bar {
    flex-direction: row;
  }
}

.sort-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  margin-bottom: 1rem;
}

.sort-label:hover {
  color: #4b5563;
}

@media (min-width: 768px) {
  .sort-label {
    margin-bottom: 0;
  }
}

.page-buttons {
  display: flex;
  gap: 1rem;
}

.page-btn {
  transition: color 150ms;
}

.page-btn:hover {
  color: #000;
}

.page-inactive {
  color: #9ca3af;
}

/* ============================================= */
/* Tags Footer                                   */
/* ============================================= */

.tags-footer {
  padding-top: 2rem;
}

.tags-label {
  margin-bottom: 1rem;
  font-size: 0.875rem;
  color: #9ca3af;
  font-family: "Noto Serif SC", serif;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  column-gap: 1rem;
  row-gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 700;
  line-height: 2;
}

@media (min-width: 768px) {
  .tags-list {
    font-size: 1.125rem;
  }
}

.tag-item {
  cursor: pointer;
  color: #6b7280;
  transition: color 200ms;
}

.tag-item:hover {
  color: #000;
}
</style>
