<template>
  <div class="shelf-view">
    <!-- 顶部导航：统一 SosAppbar 规范 + library 主题 -->
    <header class="sos-appbar shelf-appbar">
      <div class="sos-appbar__inner">
        <router-link to="/" class="sos-brand-lockup">
          <span class="sos-brand-lockup__mark">
            <img :src="logoSrc" alt="" />
          </span>
          <span class="sos-brand-lockup__text">
            <strong>长门有希的书架</strong>
            <small>凉宫春日应援团 · 书库</small>
          </span>
        </router-link>
        <nav class="sos-navlinks">
          <router-link to="/feedback" class="sos-navlink">
            同人投稿 &amp; 问题反馈
          </router-link>
        </nav>
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
// 页头品牌 logo（部署在 /library/ 子路径，需显式拼 BASE_URL）
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`;

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

/* 统一页头规范（.sos-appbar）：右侧给全局 fixed 账号菜单预留空档 */
.shelf-appbar {
  padding-right: 7.5rem;
}
/* library 品牌以衬线呈现、字重收敛（规范默认 black 偏重，这里走优雅一档） */
.shelf-appbar .sos-brand-lockup__text > strong {
  font-family: var(--sos-display-family);
  font-weight: var(--sos-weight-bold);
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
