<template>
  <div class="shelf-view">
    <!-- 书籍列表（页头由 App.vue 全局统一 SosAppbar 提供） -->
    <main class="shelf-main">
      <div v-if="loading" class="shelf-status" aria-live="polite">
        <span class="sos-spinner" aria-hidden="true"></span>
        <span>正在整理书架…</span>
      </div>

      <div
        v-else-if="books.length === 0"
        class="sos-empty-state sos-empty-state--center"
      >
        <div class="sos-empty-state__icon" aria-hidden="true">📚</div>
        <p class="sos-empty-state__title">书架空空如也</p>
        <p class="sos-empty-state__copy">
          还没有上架的书目。去后台添加几本，这里就会按栏目把它们陈列出来。
        </p>
      </div>

      <!-- 分栏书架 -->
      <div v-else class="shelf-sections">
        <section
          v-for="section in sections"
          :key="section.key"
          class="shelf-section"
        >
          <!-- 栏目标题行 -->
          <div class="shelf-section__head">
            <div class="shelf-section__title-row">
              <h2 class="shelf-section__title">{{ section.label }}</h2>
              <span class="shelf-section__count">{{ section.books.length }} 本</span>
            </div>

            <button
              v-if="section.books.length > section.previewCount"
              type="button"
              class="sos-button sos-button--ghost sos-button--sm"
              @click="toggleSection(section.key)"
            >
              <span>{{ isSectionExpanded(section.key) ? '收起' : '展开全部' }}</span>
              <svg
                class="shelf-chevron"
                viewBox="0 0 24 24"
                fill="none"
                aria-hidden="true"
              >
                <path
                  :d="
                    isSectionExpanded(section.key)
                      ? 'M6 15l6-6 6 6'
                      : 'M6 9l6 6 6-6'
                  "
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </div>

          <!-- 该栏目下的书 -->
          <div class="shelf-grid">
            <article
              v-for="book in getVisibleBooks(section)"
              :key="book.id"
              class="sos-card sos-book-card"
              role="button"
              :tabindex="0"
              :aria-label="`${book.title}${book.author ? ' · ' + book.author : ''}`"
              @click="openBook(book.id)"
              @keydown.enter.self="openBook(book.id)"
              @keydown.space.self.prevent="openBook(book.id)"
            >
              <div class="sos-book-card__cover">
                <img
                  v-if="book.cover_path"
                  :src="getCoverSrc(book)"
                  :alt="book.title"
                  loading="lazy"
                  decoding="async"
                  @error="onCoverError(book, $event)"
                />
                <span v-else class="sos-book-card__vertical" aria-hidden="true">{{
                  book.title.charAt(0)
                }}</span>
              </div>
              <div>
                <h3 class="sos-book-card__title">{{ book.title }}</h3>
                <p class="sos-book-card__author">{{ book.author || '佚名' }}</p>
              </div>
            </article>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';
import { resolveUploadUrl } from '@haruhi/api-client';

const router = useRouter();
const books = ref([]);
const loading = ref(true);
// 统一后端：模块 API 走 /api/novel，静态文件走 /uploads
const API_BASE = '/api/novel';

// === 书架分栏配置 ===
const CATEGORY_CONFIG = [
  { key: 'main', label: '正传小说', previewCount: 100 },
  { key: 'setting', label: '设定集', previewCount: 100 },
  { key: 'short', label: '官方短篇', previewCount: 100 },
  { key: 'fanfic', label: '社区同人', previewCount: 20 },
];

// 默认栏目
const DEFAULT_CATEGORY_KEY = 'main';

// 若后端暂无 category，可在这里按 id 映射
const BOOK_CATEGORY_MAP = {};

// 展开状态
const expandedKeys = ref(new Set());

// 书架封面走服务端缩略图：原图常达 2MB+，此处请求限宽 WebP，首屏不再下载原图。
const COVER_THUMB_WIDTH = 320;

// 原始封面 URL（仅作缩略图不可用时的兜底）。
const getCoverUrl = (path) => {
  if (!path) return '';
  return resolveUploadUrl(path);
};


// 模板用：返回服务端缩略图地址（限宽 WebP、磁盘缓存、immutable 强缓存）。
const getCoverSrc = (book) => {
  const p = book?.cover_path;
  if (!p) return '';
  return `${API_BASE}/thumb?path=${encodeURIComponent(p)}&w=${COVER_THUMB_WIDTH}`;
};

// 缩略图加载失败时回退原图（仅一次，防循环）。
const onCoverError = (book, e) => {
  const img = e?.target;
  if (!img || img.dataset.fellBack) return;
  const orig = getCoverUrl(book?.cover_path);
  if (orig) {
    img.dataset.fellBack = '1';
    img.src = orig;
  }
};

const fetchBooks = async () => {
  try {
    const res = await axios.get(`${API_BASE}/books`);
    books.value = res.data || [];
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
};

const openBook = (id) => {
  router.push(`/read/${id}`);
};

// 把 books 分桶到各个栏目
const sections = computed(() => {
  const buckets = {};

  // 先为已配置的栏目建空数组
  CATEGORY_CONFIG.forEach((cat) => {
    buckets[cat.key] = [];
  });

  // 把书放进对应栏目
  for (const book of books.value) {
    const explicitKey =
      book.category ||
      BOOK_CATEGORY_MAP[book.id] ||
      DEFAULT_CATEGORY_KEY;

    if (!buckets[explicitKey]) {
      buckets[explicitKey] = [];
    }
    buckets[explicitKey].push(book);
  }

  // 按配置顺序输出
  const ordered = [];

  CATEGORY_CONFIG.forEach((cat) => {
    const list = buckets[cat.key] || [];
    if (list.length) {
      ordered.push({ ...cat, books: list });
    }
  });

  // 把配置之外、但实际存在的类别追加到最后
  Object.entries(buckets).forEach(([key, list]) => {
    if (!list.length) return;
    const exists = CATEGORY_CONFIG.some((c) => c.key === key);
    if (!exists) {
      ordered.push({
        key,
        label: key,
        previewCount: 8,
        books: list,
      });
    }
  });

  return ordered;
});

const isSectionExpanded = (key) => expandedKeys.value.has(key);

const toggleSection = (key) => {
  const next = new Set(expandedKeys.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  expandedKeys.value = next;
};

const getVisibleBooks = (section) => {
  if (isSectionExpanded(section.key)) return section.books;
  return section.books.slice(0, section.previewCount || 8);
};

onMounted(fetchBooks);
</script>

<style scoped>
.shelf-view {
  min-height: 100%;
}

/* 内容区 */
.shelf-main {
  max-width: var(--sos-container-wide);
  margin-inline: auto;
  padding: var(--sos-space-8) var(--sos-page-gutter) var(--sos-space-12);
}
.shelf-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sos-space-3);
  padding-block: var(--sos-space-16);
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
}

.shelf-sections {
  display: grid;
  gap: var(--sos-space-12);
}
.shelf-section {
  display: grid;
  gap: var(--sos-space-5);
}
.shelf-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
}
/* 计数小号弱化、相对标题居中错开（沿用原版的层次感，不与标题等重对齐） */
.shelf-section__title-row {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.shelf-section__title {
  margin: 0;
  font-size: var(--sos-text-lg);
  font-weight: var(--sos-weight-semibold);
  letter-spacing: var(--sos-tracking-tight);
  color: var(--sos-text-primary);
}
.shelf-section__count {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}

/* 书架网格：一行 2/3/4/5 个大尺寸封面，留白克制优雅（沿用原版节奏） */
.shelf-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--sos-space-8) var(--sos-space-6);
  align-items: start;
}
@media (min-width: 640px) {
  .shelf-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}
@media (min-width: 960px) {
  .shelf-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}
@media (min-width: 1200px) {
  .shelf-grid {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
}

.shelf-chevron {
  width: 0.85rem;
  height: 0.85rem;
}
</style>
