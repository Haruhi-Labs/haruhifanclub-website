<template>
  <div class="shelf-view">
    <!-- 顶部导航：奶油纸编辑台气质，标题用 library 衬线 -->
    <header class="shelf-header">
      <div class="shelf-header__inner">
        <router-link to="/" class="shelf-brand">
          <span class="shelf-brand__name">长门有希的书架</span>
        </router-link>
        <router-link to="/feedback" class="shelf-header__link">
          同人投稿 &amp; 问题反馈
        </router-link>
      </div>
    </header>

    <!-- 书籍列表 -->
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
              <h2 class="sos-title shelf-section__title">{{ section.label }}</h2>
              <span class="sos-badge">{{ section.books.length }} 本</span>
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
                />
                <span v-else class="sos-book-card__vertical">{{
                  book.title
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

// 用于缓存每本书压缩后的 webp 封面：{ [bookId]: dataUrl }
const compressedCovers = ref({});

// ------------ 工具方法：原始封面 URL ------------
// 注意：不要把扩展名改写成 .webp——服务器上没有同名 webp 副本（旧站遗留假设），
// 改写会让全部 png/jpeg 封面 404。展示侧的体积优化由下方 canvas 压缩缓存承担。
const getCoverUrl = (path) => {
  if (!path) return '';
  return resolveUploadUrl(path);
};


// ------------ 工具方法：压缩图片为 webp ------------
const compressImageToWebp = (url, quality = 0.8) => {
  return new Promise((resolve, reject) => {
    try {
      const img = new Image();

      // 如前后端是不同域名，且服务器允许跨域，这个有利于避免 canvas 污染
      img.crossOrigin = 'anonymous';

      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          const { naturalWidth, naturalHeight } = img;

          // 书架封面一般不需要原始超大分辨率，可在这里做一次简单缩放
          const maxWidth = 800; // 控制最大宽度，防止特别大的图
          let targetWidth = naturalWidth;
          let targetHeight = naturalHeight;

          if (naturalWidth > maxWidth) {
            const scale = maxWidth / naturalWidth;
            targetWidth = maxWidth;
            targetHeight = Math.round(naturalHeight * scale);
          }

          canvas.width = targetWidth;
          canvas.height = targetHeight;

          ctx.drawImage(img, 0, 0, targetWidth, targetHeight);

          // 使用 toBlob 得到 webp，再转成 dataURL
          canvas.toBlob(
            (blob) => {
              if (!blob) {
                // 兜底：如果失败，就用原图
                console.warn('WebP 压缩失败，使用原图显示:', url);
                resolve(url);
                return;
              }
              const reader = new FileReader();
              reader.onloadend = () => {
                resolve(reader.result); // dataURL
              };
              reader.readAsDataURL(blob);
            },
            'image/webp',
            quality
          );
        } catch (err) {
          console.error('压缩图片时出错:', err);
          resolve(url); // 出错时退回原图
        }
      };

      img.onerror = (e) => {
        console.error('图片加载失败:', url, e);
        resolve(url); // 加载失败也退回原图
      };

      img.src = url;
    } catch (err) {
      console.error('compressImageToWebp 异常:', err);
      resolve(url);
    }
  });
};

// ------------ 在获取到书本数据后，批量预压缩封面 ------------
const precompressCovers = async (bookList) => {
  const cache = { ...compressedCovers.value };

  const tasks = bookList
    .filter((b) => b.cover_path && !cache[b.id])
    .map(async (b) => {
      const url = getCoverUrl(b.cover_path);
      const webpDataUrl = await compressImageToWebp(url, 0.7);
      cache[b.id] = webpDataUrl;
    });

  if (tasks.length) {
    await Promise.allSettled(tasks);
    compressedCovers.value = cache;
  }
};

// 供模板使用：优先返回 webp 封面
const getCoverSrc = (book) => {
  if (book?.id && compressedCovers.value[book.id]) {
    return compressedCovers.value[book.id];
  }
  // 压缩过程未完成或者失败时，先用原图兜底
  return getCoverUrl(book.cover_path);
};

const fetchBooks = async () => {
  try {
    const res = await axios.get(`${API_BASE}/books`);
    books.value = res.data || [];
    // 拿到 books 后，开始预压缩封面
    precompressCovers(books.value);
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

/* 顶栏：半透明奶油纸 + 模糊，sticky 不抢视线 */
.shelf-header {
  position: sticky;
  top: 0;
  z-index: var(--sos-z-sticky);
  border-bottom: 1px solid var(--sos-border-subtle);
  background: color-mix(in srgb, var(--sos-bg-page) 88%, transparent);
  backdrop-filter: blur(8px);
}
.shelf-header__inner {
  max-width: var(--sos-container-wide);
  min-height: 4rem;
  margin-inline: auto;
  /* 右侧预留固定宽度账号菜单的位置（账号菜单是全局 fixed），避免反馈链接被遮挡；
     窄屏放不下时整体换行，标题独占一行、账号菜单浮于其右侧空档。 */
  padding: var(--sos-space-3) 7.5rem var(--sos-space-3) var(--sos-page-gutter);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-2) var(--sos-space-4);
}
.shelf-brand {
  display: inline-flex;
  align-items: center;
  text-decoration: none;
}
.shelf-brand__name {
  font-family: var(--sos-display-family);
  font-size: var(--sos-text-xl);
  font-weight: var(--sos-weight-heavy);
  letter-spacing: var(--sos-tracking-wide);
  color: var(--sos-text-primary);
  white-space: nowrap;
}
.shelf-brand:hover .shelf-brand__name {
  color: var(--sos-link);
}
.shelf-header__link {
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
  text-decoration: none;
  white-space: nowrap;
  transition: color var(--sos-duration-fast) var(--sos-ease-out);
}
.shelf-header__link:hover {
  color: var(--sos-link);
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
.shelf-section__title-row {
  display: inline-flex;
  align-items: baseline;
  gap: var(--sos-space-3);
}
/* 栏目标题比页面主标题小一档，保持层级 */
.shelf-section__title {
  font-size: var(--sos-text-xl);
}

/* 书架网格：随容器自动密排，少量书目也左对齐不撑大 */
.shelf-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(8.5rem, 1fr));
  gap: var(--sos-space-6) var(--sos-space-5);
  align-items: start;
}

.shelf-chevron {
  width: 0.85rem;
  height: 0.85rem;
}

/* 长标题占位封面：竖排书脊保持在框内，溢出收口 */
.sos-book-card__vertical {
  max-block-size: calc(100% - var(--sos-space-6));
  overflow: hidden;
}

@media (max-width: 640px) {
  .shelf-grid {
    grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr));
  }
}
</style>
