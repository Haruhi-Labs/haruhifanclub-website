<template>
  <div class="min-h-screen bg-[#FAF9DE] text-[#4A3B32] font-sans">
    <!-- 顶部导航 -->
    <header
      class="h-16 px-6 flex items-center justify-between border-b border-[#E6DFD0] bg-[#FAF9DE]/90 backdrop-blur sticky top-0 z-10"
    >
      <div class="flex items-center gap-3">
        <span
          class="text-xl font-bold font-serif tracking-wider text-[#5C4B41]"
          >长门有希的书架</span
        >
      </div>
      <router-link
        to="/Feedback"
        class="text-sm text-[#8C7B70] hover:text-[#D97757] transition-colors"
      >
        同人投稿 & 问题反馈
      </router-link>
    </header>

    <!-- 书籍列表 -->
    <main class="p-6 max-w-7xl mx-auto">
      <div v-if="loading" class="text-center py-20 text-[#8C7B70]">
        正在整理书架...
      </div>

      <div v-else-if="books.length === 0" class="text-center py-20">
        <div class="text-6xl mb-4">📚</div>
        <p class="text-[#8C7B70]">书架空空如也，去后台添加几本吧</p>
      </div>

      <!-- 分栏书架 -->
      <div v-else class="space-y-10">
        <section
          v-for="section in sections"
          :key="section.key"
          class="space-y-4"
        >
          <!-- 栏目标题行 -->
          <div class="flex items-baseline justify-between">
            <div class="flex items-center gap-3">
              <h2 class="text-lg font-semibold text-[#4A3B32]">
                {{ section.label }}
              </h2>
              <span class="text-xs text-[#B0A090]">
                {{ section.books.length }} 本
              </span>
            </div>

            <button
              v-if="section.books.length > section.previewCount"
              @click="toggleSection(section.key)"
              class="text-xs text-[#8C7B70] hover:text-[#D97757] flex items-center gap-1"
            >
              <span>{{ isSectionExpanded(section.key) ? '收起' : '展开全部' }}</span>
              <svg
                class="w-3 h-3"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
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
          <div
            class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6"
          >
            <div
              v-for="book in getVisibleBooks(section)"
              :key="book.id"
              class="group cursor-pointer flex flex-col items-center"
              @click="openBook(book.id)"
            >
              <!-- 封面 -->
              <div
                class="w-full aspect-[2/3] bg-white rounded shadow-sm group-hover:shadow-md group-hover:-translate-y-1 transition-all duration-300 overflow-hidden border border-[#E6DFD0] relative"
              >
                <img
                  v-if="book.cover_path"
                  :src="getCoverUrl(book.cover_path)"
                  class="w-full h-full object-cover"
                  loading="lazy"
                />
                <div
                  v-else
                  class="w-full h-full flex items-center justify-center bg-[#F2EFE4] text-[#B0A090] text-4xl font-serif"
                >
                  {{ book.title[0] }}
                </div>

                <!-- 遮罩 -->
                <div
                  class="absolute inset-0 bg-black/0 group-hover:bg-black/5 transition-colors"
                ></div>
              </div>

              <!-- 信息 -->
              <div class="mt-3 text-center w-full">
                <h3
                  class="font-bold text-sm truncate px-1 group-hover:text-[#D97757] transition-colors"
                >
                  {{ book.title }}
                </h3>
                <p class="text-xs text-[#8C7B70] mt-1 truncate">
                  {{ book.author || '佚名' }}
                </p>
              </div>
            </div>
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
