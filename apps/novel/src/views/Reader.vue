<!-- src/views/Reader.vue -->
<template>
  <div class="reader-root h-screen w-full flex flex-col md:flex-row">
    <!-- Loading 遮罩 -->
    <div
      v-if="isLoading"
      class="reader-loading fixed inset-0 z-50 flex flex-col items-center justify-center"
    >
      <div class="loader mb-4"></div>
      <p class="reader-loading__text text-sm animate-pulse">{{ loadingText }}</p>
    </div>

    <!-- 侧边栏目录 -->
    <aside
      class="reader-toc flex-shrink-0 transition-all duration-300 z-20 absolute md:relative h-full"
      :class="sidebarOpen ? 'w-72 translate-x-0 shadow-lg md:shadow-none' : 'w-72 -translate-x-full md:w-0 md:overflow-hidden'"
    >
      <div class="p-6 h-full flex flex-col">
        <div class="flex items-center justify-between mb-6">
          <h2 class="reader-toc__title">目录</h2>
          <button
            type="button"
            class="reader-icon-btn md:hidden"
            aria-label="关闭目录"
            @click="sidebarOpen = false"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>

        <!-- 章节 + 子锚点 列表 -->
        <div class="flex-1 overflow-y-auto pr-2 space-y-1 custom-scrollbar">
          <button
            v-for="item in sidebarItems"
            :key="item.key"
            type="button"
            class="reader-toc__item"
            :class="[
              item.level === 0 ? 'reader-toc__item--chapter' : 'reader-toc__item--sub',
              { 'is-active': isSidebarItemActive(item) },
            ]"
            :aria-current="isSidebarItemActive(item) ? 'true' : undefined"
            @click="onSidebarItemClick(item)"
          >
            <span class="truncate">{{ item.label }}</span>
          </button>
        </div>
      </div>
    </aside>

    <!-- 主内容 -->
    <main class="flex-1 flex flex-col h-full relative overflow-hidden">
      <!-- 顶部栏 -->
      <header class="reader-bar h-16 flex items-center justify-between gap-2 px-3 md:px-6">
        <div class="flex items-center gap-2 md:gap-4 min-w-0 flex-1">
          <button
            type="button"
            class="reader-icon-btn"
            aria-label="切换目录"
            @click="sidebarOpen = !sidebarOpen"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="3" y1="12" x2="21" y2="12"></line>
              <line x1="3" y1="6" x2="21" y2="6"></line>
              <line x1="3" y1="18" x2="21" y2="18"></line>
            </svg>
          </button>
          <div class="reader-titles">
            <span class="reader-titles__book">{{ bookTitle }}</span>
            <span class="reader-titles__chapter">{{ chapterTitle }}</span>
          </div>
        </div>

        <div class="flex items-center gap-1.5 md:gap-3 flex-shrink-0">
          <!-- 返回书架 -->
          <router-link to="/" class="reader-icon-btn" title="返回书架" aria-label="返回书架">
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
              <polyline points="9 22 9 12 15 12 15 22"></polyline>
            </svg>
          </router-link>

          <!-- 简繁切换 -->
          <div class="reader-seg" role="group" aria-label="文本字体">
            <button
              type="button"
              class="reader-seg__btn"
              :class="{ 'is-active': textVariant === 'original' }"
              :aria-pressed="textVariant === 'original'"
              @click="setTextVariant('original')"
            >
              原
            </button>
            <button
              type="button"
              class="reader-seg__btn"
              :class="{ 'is-active': textVariant === 'sc' }"
              :aria-pressed="textVariant === 'sc'"
              @click="setTextVariant('sc')"
            >
              简
            </button>
            <button
              type="button"
              class="reader-seg__btn"
              :class="{ 'is-active': textVariant === 'tc' }"
              :aria-pressed="textVariant === 'tc'"
              @click="setTextVariant('tc')"
            >
              繁
            </button>
          </div>

          <!-- 阅读模式切换 -->
          <div class="reader-seg" role="group" aria-label="阅读模式">
            <button
              type="button"
              class="reader-seg__btn"
              :class="{ 'is-active': readingMode === 'scroll' }"
              :aria-pressed="readingMode === 'scroll'"
              @click="setReadingMode('scroll')"
            >
              流式
            </button>
            <button
              type="button"
              class="reader-seg__btn"
              :class="{ 'is-active': readingMode === 'flip' }"
              :aria-pressed="readingMode === 'flip'"
              @click="setReadingMode('flip')"
            >
              翻页
            </button>
          </div>
        </div>
      </header>

      <!-- 内容区 -->
      <div class="reader-content flex-1 relative w-full overflow-hidden">
        <!-- 流式阅读 -->
        <div
          v-show="readingMode === 'scroll'"
          ref="scrollContainer"
          class="h-full overflow-y-auto px-4 md:px-12 py-8 scroll-smooth custom-scrollbar"
        >
          <div class="reader-paper max-w-3xl mx-auto min-h-[80vh] p-6 md:p-10">
            <div class="novel-content pb-20" v-html="safeContent"></div>
          </div>
          <div class="reader-chapnav max-w-3xl mx-auto mt-8 flex justify-between items-center pt-6 pb-24 px-4">
            <button
              type="button"
              class="reader-chapnav__btn"
              :disabled="currentChapterIndex <= 0"
              @click="prevChapter"
            >
              ← 上一章
            </button>
            <button
              type="button"
              class="reader-chapnav__btn"
              :disabled="currentChapterIndex >= chapters.length - 1"
              @click="nextChapter"
            >
              下一章 →
            </button>
          </div>
        </div>

        <!-- 翻页阅读 -->
        <div
          v-show="readingMode === 'flip'"
          class="h-full flex flex-col justify-center items-center select-none"
        >
          <!-- 中间书页卡片 -->
          <div
            class="w-full flex-1 flex items-center justify-center relative px-4 md:px-8"
          >
            <div ref="flipFrame" class="flip-page-frame">
              <div
                ref="flipColumns"
                class="flip-columns novel-content"
                :style="{ transform: pageTransform }"
                v-html="safeContent"
              ></div>
            </div>

            <!-- 左右点击区 -->
            <div
              class="absolute inset-y-0 left-0 w-1/3 md:w-1/4 cursor-pointer z-10"
              @click="prevPage"
            ></div>
            <div
              class="absolute inset-y-0 right-0 w-1/3 md:w-1/4 cursor-pointer z-10"
              @click="nextPage"
            ></div>
          </div>

          <!-- 页脚进度条 -->
          <div class="reader-progress h-12 w-full flex items-center justify-between px-6 z-20">
            <span class="reader-progress__chapter truncate max-w-[150px]">
              {{ chapterTitle }}
            </span>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="reader-icon-btn"
                aria-label="上一页"
                :disabled="currentPage <= 0"
                @click="prevPage"
              >
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="15 18 9 12 15 6"></polyline>
                </svg>
              </button>
              <div class="reader-progress__track w-20 md:w-32">
                <div
                  class="reader-progress__fill"
                  :style="{ width: `${((currentPage + 1) / (totalPages || 1)) * 100}%` }"
                ></div>
              </div>
              <button
                type="button"
                class="reader-icon-btn"
                aria-label="下一页"
                :disabled="currentPage >= totalPages - 1"
                @click="nextPage"
              >
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="9 18 15 12 9 6"></polyline>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup>
import {
  ref,
  onMounted,
  onBeforeUnmount,
  nextTick,
  watch,
  computed,
} from 'vue';
import { useRoute } from 'vue-router';
import axios from 'axios';
import ePub from 'epubjs';
import DOMPurify from 'dompurify';
import { resolveUploadUrl } from '@haruhi/api-client';

const route = useRoute();
// 统一后端：模块 API 走 /api/novel，静态文件走 /uploads
const API_BASE = '/api/novel';

// 小工具：兼容 CSS.escape
const cssEscape = (str) => {
  if (typeof CSS !== 'undefined' && CSS.escape) return CSS.escape(str);
  return str.replace(/([^a-zA-Z0-9_-])/g, '\\$1');
};

// ---- 锚点 ID 统一规则：文件名前缀 + 原始 ID ----
const normalizeHref = (href) => {
  if (!href) return '';
  const clean = href.split('#')[0].split('?')[0];
  return clean.replace(/\\/g, '/');
};

const fileKeyFromHref = (href) => {
  const n = normalizeHref(href);
  const name = n.split('/').pop() || n;
  // 去掉扩展名，并把奇怪字符转成下划线
  return name.replace(/\.[^./]+$/, '').replace(/[^a-zA-Z0-9_-]/g, '_');
};

const makeDomId = (fileHref, rawId) => {
  if (!rawId) return null;
  return `${fileKeyFromHref(fileHref)}__${rawId}`;
};

// ------- 状态 -------

const book = ref(null);

// 章节 + 子锚点目录
const chapters = ref([]);
const currentChapterIndex = ref(0);
const currentContent = ref('');
// EPUB 章节是外部上传的 HTML，渲染前用 DOMPurify 净化，剥离 script/事件处理器等，
// 保留正文排版标签；防止恶意 EPUB 对读者发起 XSS。
//
// 插图修复：processImages 把 epub 内的图片抽成 createObjectURL 的 blob: URL，
// 而 DOMPurify 默认 URI 白名单不含 blob:（也不含 data:），会把这些 src 整个剥掉，
// 导致全书插图/彩页裂图。此处精准放行——仅 <img> 的 src、且值为 blob: 或
// data:image/ 时保留；图片上下文不执行脚本，不放宽其它标签/属性的协议限制。
DOMPurify.addHook('uponSanitizeAttribute', (node, data) => {
  if (
    node.tagName === 'IMG' &&
    data.attrName === 'src' &&
    /^(?:blob:|data:image\/)/i.test(data.attrValue)
  ) {
    data.forceKeepAttr = true;
  }
});

const safeContent = computed(() =>
  DOMPurify.sanitize(currentContent.value, { USE_PROFILES: { html: true } }),
);
const bookTitle = ref('加载中...');

const sidebarOpen = ref(
  typeof window !== 'undefined' ? window.innerWidth >= 768 : true,
);

const readingMode = ref('scroll');
const textVariant = ref('original'); // 'original' | 'sc' | 'tc'
const isLoading = ref(true);
const loadingText = ref('正在打开书籍...');

// 当前目录高亮 key
const activeNavKey = ref(null);
// 待跳转锚点（已经是带前缀的 DOM id）
const pendingAnchor = ref(null);

// 翻页相关
const currentPage = ref(0);
const totalPages = ref(1);
const containerWidth = ref(0);

// epub zip 文件映射
const fileMap = ref({});

// DOM 引用
const flipFrame = ref(null);
const flipColumns = ref(null);
const scrollContainer = ref(null);

// 滚动模式下各锚点高度
const anchorPositions = ref([]);

// OpenCC 状态
const openccReady = ref(false);
let openccConverters = {
  sc: null, // 繁 -> 简
  tc: null, // 简 -> 繁
};

// key 生成
const makeChapterKey = (chapter, index) =>
  `c-${chapter?.id || index}`;
const makeAnchorKey = (chapter, chapterIndex, anchor, anchorIndex) =>
  `c-${chapter?.id || chapterIndex}-a-${anchor?.id || anchorIndex}`;

// 侧边栏拍扁后的数据
const sidebarItems = computed(() => {
  const items = [];
  chapters.value.forEach((chapter, chapterIndex) => {
    const cKey = makeChapterKey(chapter, chapterIndex);
    items.push({
      type: 'chapter',
      key: cKey,
      label: chapter.label || `第 ${chapterIndex + 1} 章`,
      chapterIndex,
      level: 0,
      anchorId: null,
    });

    (chapter.anchors || []).forEach((anchor, aIndex) => {
      const aKey = makeAnchorKey(chapter, chapterIndex, anchor, aIndex);
      items.push({
        type: 'anchor',
        key: aKey,
        label: anchor.label || anchor.anchor || `段落 ${aIndex + 1}`,
        chapterIndex,
        level: 1,
        anchorId: anchor.anchor || null, // 这里已经是带前缀的 DOM id
      });
    });
  });
  return items;
});

const chapterTitle = computed(
  () => chapters.value[currentChapterIndex.value]?.label || '',
);

// 翻页模式 transform
const pageTransform = computed(
  () => `translateX(-${currentPage.value * containerWidth.value}px)`,
);

// ------- 事件 -------

const handleResize = () => {
  calculatePagination();
};

const handleKeydown = (e) => {
  if (readingMode.value !== 'flip') return;
  if (e.key === 'ArrowRight' || e.key === 'PageDown' || e.key === ' ') {
    e.preventDefault();
    nextPage();
  } else if (e.key === 'ArrowLeft' || e.key === 'PageUp') {
    e.preventDefault();
    prevPage();
  }
};

// 滚动检测当前锚点（使用顶部位置 + 容差）
const handleScroll = () => {
  if (readingMode.value !== 'scroll') return;
  if (!scrollContainer.value) return;

  const list = anchorPositions.value;
  if (!list.length) return;

  const container = scrollContainer.value;
  const scrollTop = container.scrollTop;

  const TOP_MARGIN = 48;
  const pos = scrollTop + TOP_MARGIN;

  let activeIndex = 0;
  for (let i = 0; i < list.length; i++) {
    if (pos >= list[i].offset - TOP_MARGIN) {
      activeIndex = i;
    } else {
      break;
    }
  }

  const active = list[activeIndex];
  if (active && activeNavKey.value !== active.key) {
    activeNavKey.value = active.key;
  }
};

onMounted(async () => {
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', handleResize);
    window.addEventListener('keydown', handleKeydown);
  }

  await loadBookData(route.params.id);

  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.addEventListener('scroll', handleScroll, {
        passive: true,
      });
    }
  });
});

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', handleResize);
    window.removeEventListener('keydown', handleKeydown);
  }
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll);
  }
  if (book.value) book.value.destroy();
});

// ------- 数据加载 -------

const loadBookData = async (id) => {
  try {
    const metaRes = await axios.get(`${API_BASE}/books/${id}`);
    bookTitle.value = metaRes.data.title;

    const fileRes = await axios.get(resolveUploadUrl(metaRes.data.file_path), {
      responseType: 'arraybuffer',
      onDownloadProgress: (e) => {
        if (!e.total) return;
        const percent = Math.round((e.loaded * 100) / e.total);
        loadingText.value = `正在下载书籍 ${percent}%...`;
      },
    });

    loadingText.value = '正在解析 EPUB...';
    openBook(fileRes.data);
  } catch (e) {
    console.error('书籍加载失败', e);
    currentContent.value =
      `<div class='text-center p-10'>书籍加载失败<br><span class="text-xs text-gray-400">${e.message}</span></div>`;
    isLoading.value = false;
  }
};

const openBook = (arrayBuffer) => {
  if (book.value) book.value.destroy();
  book.value = ePub(arrayBuffer);

  book.value.ready
    .then(() => {
      const zip = book.value.archive.zip;
      fileMap.value = {};
      for (const path in zip.files) {
        if (!Object.prototype.hasOwnProperty.call(zip.files, path)) continue;
        if (!path.endsWith('/') && !path.includes('__MACOSX')) {
          const fileName = path.split('/').pop();
          fileMap.value[fileName] = path;
        }
      }

      book.value.loaded.navigation.then((nav) => {
        const chapterList = [];

        const walk = (items, parentChapter = null) => {
          items.forEach((item) => {
            const label = (item.label || '').trim();
            const href = item.href || '';
            const id =
              item.id || href || label || String(chapterList.length);
            const [fileHref, hash] = href.split('#');

            if (!parentChapter) {
              const chapter = {
                id,
                label,
                href,
                fileHref,
                anchors: [],
                rawHtml: null, // 原始 HTML（未做简繁转换）
              };
              chapterList.push(chapter);
              if (item.subitems && item.subitems.length) {
                walk(item.subitems, chapter);
              }
            } else {
              // 子锚点：注意它可能落在和父章节不同的文件里
              const fileForAnchor = fileHref || parentChapter.fileHref;
              const domId = hash ? makeDomId(fileForAnchor, hash) : null;

              const anchor = {
                id,
                label,
                href,
                fileHref: fileForAnchor,
                // 用于 DOM 中真正的 id（带文件名前缀）
                anchor: domId,
                // 原始锚点名（可选，调试用）
                rawAnchor: hash || null,
              };
              parentChapter.anchors.push(anchor);

              if (item.subitems && item.subitems.length) {
                walk(item.subitems, parentChapter);
              }
            }
          });
        };

        walk(nav.toc || [], null);
        chapters.value = chapterList;

        if (chapters.value.length > 0) {
          currentChapterIndex.value = 0;
          activeNavKey.value = makeChapterKey(chapters.value[0], 0);
          loadChapter(0);
        } else {
          currentContent.value = '<p>未检测到目录，请尝试手动翻阅。</p>';
          finishLoad();
        }
      });
    })
    .catch((err) => {
      console.error('EPUB 解析错误', err);
      currentContent.value = 'EPUB 解析错误';
      isLoading.value = false;
    });
};

// ------- 工具 -------

const findFileInZip = (href) => {
  if (!href) return null;
  const clean = href.split('#')[0];
  const fileName = clean.split('/').pop();
  return fileMap.value[fileName] || null;
};

const getSpineIndex = (href) => {
  if (!book.value || !book.value.spine) return -1;
  const item = book.value.spine.get(href);
  if (item) return item.index;

  const targetName = href.split('/').pop().split('#')[0];
  for (let i = 0; i < book.value.spine.items.length; i++) {
    if (book.value.spine.items[i].href.endsWith(targetName)) return i;
  }
  return -1;
};

const processImages = async (doc) => {
  const images = Array.from(doc.querySelectorAll('img, image'));
  const tasks = images.map(async (img) => {
    let src =
      img.getAttribute('src') ||
      img.getAttribute('xlink:href') ||
      img.getAttribute('href');
    if (!src) return;

    try {
      src = decodeURIComponent(src);
    } catch {}

    const realPath = findFileInZip(src);
    if (!realPath) return;

    const imgFile = book.value.archive.zip.file(realPath);
    if (!imgFile) return;
    const blob = await imgFile.async('blob');
    const url = URL.createObjectURL(blob);

    if (img.tagName.toLowerCase() === 'image') {
      const newImg = document.createElement('img');
      newImg.src = url;
      newImg.style.cssText =
        'display:block; margin:1em auto; max-width:100%; height:auto;';
      const svgParent = img.closest('svg');
      if (svgParent && svgParent.parentNode) {
        svgParent.parentNode.replaceChild(newImg, svgParent);
      } else if (img.parentNode) {
        img.parentNode.replaceChild(newImg, img);
      }
    } else {
      img.src = url;
    }
  });
  await Promise.all(tasks);
};

// 把一个 spine 文档里的 id / #id 全部加上文件名前缀，避免多文件冲突
const rebaseAnchorIds = (doc, spineHref) => {
  if (!doc || !doc.body) return;
  const fileHref = normalizeHref(spineHref);

  const idMap = new Map();

  // 1) 元素 id -> 新 id
  doc.body.querySelectorAll('[id]').forEach((el) => {
    const oldId = el.getAttribute('id');
    if (!oldId) return;
    const newId = makeDomId(fileHref, oldId);
    idMap.set(oldId, newId);
    el.setAttribute('id', newId);
  });

  // 2) 同一文件内的 <a href="#xxx"> 也要跟着改
  doc.body.querySelectorAll('a[href^="#"]').forEach((a) => {
    const href = a.getAttribute('href') || '';
    const raw = href.slice(1); // 去掉 #
    const mapped = idMap.get(raw);
    if (mapped) {
      a.setAttribute('href', `#${mapped}`);
    }
  });
};

// ------- OpenCC 简繁转换 -------

const ensureOpenCC = async () => {
  if (openccReady.value) return;

  if (typeof window === 'undefined' || !window.OpenCC) {
    console.warn(
      '[OpenCC] window.OpenCC 不存在，已回退为原文显示（请确认在 index.html 中通过 <script> 引入 opencc-js）',
    );
    return;
  }

  const { Converter } = window.OpenCC || {};
  if (!Converter) {
    console.warn('[OpenCC] OpenCC.Converter 不存在');
    return;
  }

  // 小工具：兼容同步 / 异步返回
  const makeConverter = async (optsList) => {
    for (const opts of optsList) {
      try {
        const maybe = Converter(opts);
        const fn =
          maybe && typeof maybe.then === 'function' ? await maybe : maybe;
        if (typeof fn === 'function') return fn;
      } catch (e) {
        console.warn(
          '[OpenCC] 转换器初始化方案失败',
          opts,
          e,
        );
      }
    }
    return null;
  };

  // 优先用 t / s，失败再尝试 tw / cn
  const scConv = await makeConverter([
    { from: 't', to: 's' },
    { from: 'tw', to: 'cn' },
  ]);
  const tcConv = await makeConverter([
    { from: 's', to: 't' },
    { from: 'cn', to: 'tw' },
  ]);

  if (scConv && tcConv) {
    openccConverters.sc = scConv;
    openccConverters.tc = tcConv;
    openccReady.value = true;
    console.info('[OpenCC] 简繁转换已就绪');
  } else {
    console.error('[OpenCC] 初始化失败，已回退为原文显示');
  }
};

// 按当前 textVariant + 当前章节 rawHtml 生成 currentContent
const applyTextVariant = async () => {
  const chapter = chapters.value[currentChapterIndex.value];
  if (!chapter || !chapter.rawHtml) return;

  const rawHtml = chapter.rawHtml;

  // 原文：不做转换
  if (textVariant.value === 'original') {
    currentContent.value = rawHtml;
    return;
  }

  if (typeof window === 'undefined' || !window.OpenCC) {
    console.warn('[OpenCC] 未找到 OpenCC，使用原文');
    currentContent.value = rawHtml;
    return;
  }

  await ensureOpenCC();

  const conv =
    textVariant.value === 'sc'
      ? openccConverters.sc
      : openccConverters.tc;

  if (!conv) {
    currentContent.value = rawHtml;
    return;
  }

  try {
    const result = conv(rawHtml);
    if (result && typeof result.then === 'function') {
      currentContent.value = await result;
    } else {
      currentContent.value = result;
    }
  } catch (e) {
    console.error('[OpenCC] 文本转换失败，回退原文', e);
    currentContent.value = rawHtml;
  }
};

// 按钮：切换字形模式
const setTextVariant = (variant) => {
  if (textVariant.value === variant) return;
  textVariant.value = variant;

  // 不重置滚动，不触发 finishLoad，只重新渲染文本 + 更新锚点布局
  nextTick(async () => {
    await applyTextVariant();
    nextTick(() => {
      calculatePagination();
      if (readingMode.value === 'scroll') {
        buildAnchorPositions();
        handleScroll();
      } else {
        updateActiveAnchorInFlip();
      }
    });
  });
};

// ------- 加载章节 -------

// 用于“找不到 spine 但能 load 单个文件”的情况
const updateContent = async (html, chapter) => {
  if (chapter) {
    chapter.rawHtml = html;
    chapter.content = html;
  }
  await applyTextVariant();
  finishLoad();
};

const loadChapter = async (index) => {
  if (index < 0 || index >= chapters.value.length) return;
  isLoading.value = true;
  loadingText.value = '正在渲染章节...';

  currentChapterIndex.value = index;
  const currentChapter = chapters.value[index];
  const nextChapter = chapters.value[index + 1];

  if (!pendingAnchor.value && currentChapter) {
    activeNavKey.value = makeChapterKey(currentChapter, index);
  }

  if (!currentChapter) {
    finishLoad();
    return;
  }

  // 已经加载过：直接用 rawHtml + 当前简繁模式
  if (currentChapter.rawHtml) {
    await applyTextVariant();
    finishLoad();
    return;
  }

  try {
    let startIndex = getSpineIndex(currentChapter.href);
    if (startIndex === -1) {
      console.warn('Spine index not found, trying direct load');
      const doc = await book.value.load(currentChapter.href);
      await processImages(doc);
      rebaseAnchorIds(doc, currentChapter.href);
      const html =
        doc.body?.innerHTML || '<p>本章内容为空</p>';
      await updateContent(html, currentChapter);
      return;
    }

    let endIndex = book.value.spine.items.length;
    if (nextChapter) {
      const nextIndex = getSpineIndex(nextChapter.href);
      if (nextIndex !== -1 && nextIndex > startIndex) endIndex = nextIndex;
    }

    const htmlParts = [];
    for (let i = startIndex; i < endIndex; i++) {
      const spineItem = book.value.spine.items[i];
      try {
        const doc = await book.value.load(spineItem.href);
        await processImages(doc);

        // 每个文件内的 id / #id 都做前缀处理
        rebaseAnchorIds(doc, spineItem.href);

        if (doc && doc.body) {
          const scripts = doc.body.querySelectorAll('script, style, link');
          scripts.forEach((el) => el.remove());
          htmlParts.push(doc.body.innerHTML);
        }
      } catch (err) {
        console.warn(`Failed to load spine item ${spineItem.href}`, err);
        const realPath = findFileInZip(spineItem.href);
        if (realPath) {
          const text = await book.value.archive.zip
            .file(realPath)
            .async('string');
          const bodyMatch = text.match(
            /<body[^>]*>([\s\S]*?)<\/body>/i,
          );
          if (bodyMatch) htmlParts.push(bodyMatch[1]);
        }
      }
    }

    const finalHtml =
      htmlParts.join(
        '<div class="chapter-divider" style="height: 50px;"></div>',
      ) || '<p>本章内容为空</p>';

    currentChapter.rawHtml = finalHtml;
    currentChapter.content = finalHtml;

    await applyTextVariant();
    finishLoad();
  } catch (err) {
    console.error('章节加载流程错误', err);
    currentContent.value =
      `<div class="p-10 text-center text-gray-500">加载异常: ${err.message}</div>`;
    finishLoad();
  }
};

const finishLoad = () => {
  isLoading.value = false;

  if (typeof window !== 'undefined' && window.innerWidth < 768) {
    sidebarOpen.value = false;
  }

  if (!pendingAnchor.value) {
    if (readingMode.value === 'scroll') {
      if (scrollContainer.value) scrollContainer.value.scrollTop = 0;
    } else {
      currentPage.value = 0;
    }
  }

  nextTick(() => {
    calculatePagination();

    if (readingMode.value === 'scroll') {
      buildAnchorPositions();
      handleScroll();
    } else {
      updateActiveAnchorInFlip();
    }

    if (pendingAnchor.value) {
      scrollToAnchor(pendingAnchor.value);
      pendingAnchor.value = null;
    }
  });
};

// ------- 锚点 / 高亮 -------

const onSidebarItemClick = (item) => {
  if (item.type === 'chapter') {
    pendingAnchor.value = null;
    activeNavKey.value = item.key;
    loadChapter(item.chapterIndex);
  } else if (item.type === 'anchor') {
    activeNavKey.value = item.key;
    if (currentChapterIndex.value === item.chapterIndex) {
      scrollToAnchor(item.anchorId);
    } else {
      pendingAnchor.value = item.anchorId;
      loadChapter(item.chapterIndex);
    }
  }
};

const isSidebarItemActive = (item) => {
  if (activeNavKey.value) return activeNavKey.value === item.key;
  return item.type === 'chapter' && item.chapterIndex === currentChapterIndex.value;
};

const buildAnchorPositions = () => {
  anchorPositions.value = [];
  if (readingMode.value !== 'scroll') return;
  if (!scrollContainer.value) return;

  const container = scrollContainer.value;
  const contentRoot = container.querySelector('.novel-content');
  if (!contentRoot) return;

  const chapter = chapters.value[currentChapterIndex.value];
  if (!chapter) return;

  const list = [];
  const baseKey = makeChapterKey(chapter, currentChapterIndex.value);
  list.push({ key: baseKey, offset: 0 });

  (chapter.anchors || []).forEach((anchor, index) => {
    if (!anchor.anchor) return;
    const selector = `#${cssEscape(anchor.anchor)}`;
    const el = contentRoot.querySelector(selector);
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    const offset =
      rect.top - containerRect.top + container.scrollTop;
    list.push({
      key: makeAnchorKey(chapter, currentChapterIndex.value, anchor, index),
      offset,
    });
  });

  list.sort((a, b) => a.offset - b.offset);
  anchorPositions.value = list;
};

const scrollToAnchor = (anchorId) => {
  if (!anchorId) {
    if (readingMode.value === 'scroll') {
      if (scrollContainer.value) {
        scrollContainer.value.scrollTo({ top: 0, behavior: 'smooth' });
      }
    } else if (readingMode.value === 'flip') {
      currentPage.value = 0;
    }
    return;
  }

  nextTick(() => {
    if (readingMode.value === 'scroll') {
      const container = scrollContainer.value;
      if (!container) return;
      const contentRoot = container.querySelector('.novel-content');
      if (!contentRoot) return;

      const selector = `#${cssEscape(anchorId)}`;
      const target = contentRoot.querySelector(selector);
      if (!target) return;

      const rect = target.getBoundingClientRect();
      const containerRect = container.getBoundingClientRect();
      const offset =
        rect.top - containerRect.top + container.scrollTop - 64;

      container.scrollTo({
        top: Math.max(offset, 0),
        behavior: 'smooth',
      });
    } else if (readingMode.value === 'flip') {
      if (!flipColumns.value || !containerWidth.value) return;
      const selector = `#${cssEscape(anchorId)}`;
      const target = flipColumns.value.querySelector(selector);
      if (!target) return;

      const colsRect = flipColumns.value.getBoundingClientRect();
      const targetRect = target.getBoundingClientRect();
      const relativeX = targetRect.left - colsRect.left;
      const page = Math.floor(relativeX / containerWidth.value);
      if (!Number.isNaN(page)) {
        currentPage.value = Math.min(
          Math.max(page, 0),
          totalPages.value - 1,
        );
      }
    }
  });
};

const updateActiveAnchorInFlip = () => {
  if (readingMode.value !== 'flip') return;
  const chapter = chapters.value[currentChapterIndex.value];
  if (!chapter || !flipFrame.value || !flipColumns.value) return;

  const frameRect = flipFrame.value.getBoundingClientRect();
  const centerX = (frameRect.left + frameRect.right) / 2;
  const cols = flipColumns.value;

  const candidates = [];

  const baseKey = makeChapterKey(chapter, currentChapterIndex.value);
  const colsRect = cols.getBoundingClientRect();
  const baseCenter = (colsRect.left + colsRect.right) / 2;
  candidates.push({
    key: baseKey,
    distance: Math.abs(baseCenter - centerX),
  });

  (chapter.anchors || []).forEach((anchor, index) => {
    if (!anchor.anchor) return;
    const selector = `#${cssEscape(anchor.anchor)}`;
    const el = cols.querySelector(selector);
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const elCenter = (rect.left + rect.right) / 2;
    const dist = Math.abs(elCenter - centerX);
    candidates.push({
      key: makeAnchorKey(chapter, currentChapterIndex.value, anchor, index),
      distance: dist,
    });
  });

  if (!candidates.length) {
    activeNavKey.value = baseKey;
  } else {
    const best = candidates.reduce(
      (min, c) => (c.distance < min.distance ? c : min),
      candidates[0],
    );
    activeNavKey.value = best.key;
  }
};

// ------- 翻页 / 章节 / 模式 -------

const calculatePagination = () => {
  if (readingMode.value !== 'flip' || !flipFrame.value || !flipColumns.value)
    return;

  const frame = flipFrame.value;
  const cols = flipColumns.value;

  const frameStyle = window.getComputedStyle(frame);
  const colsStyle = window.getComputedStyle(cols);

  const paddingLeft = parseFloat(frameStyle.paddingLeft) || 0;
  const paddingRight = parseFloat(frameStyle.paddingRight) || 0;

  const columnsPerPage = window.innerWidth >= 1024 ? 2 : 1;

  let colWidth = parseFloat(colsStyle.columnWidth);
  let colGap = parseFloat(colsStyle.columnGap);
  if (!Number.isFinite(colGap)) colGap = 32;

  if (
    !Number.isFinite(colWidth) ||
    colWidth <= 0 ||
    colsStyle.columnWidth === 'auto'
  ) {
    const inner = frame.clientWidth - paddingLeft - paddingRight;
    colWidth = (inner - (columnsPerPage - 1) * colGap) / columnsPerPage;
  }

  const viewWidth = columnsPerPage * colWidth + (columnsPerPage - 1) * colGap;
  const frameMaxWidth = viewWidth + paddingLeft + paddingRight;
  frame.style.maxWidth = `${frameMaxWidth}px`;

  const colsStyle2 = window.getComputedStyle(cols);
  const cw = parseFloat(colsStyle2.columnWidth) || colWidth;
  const cg = parseFloat(colsStyle2.columnGap) || colGap;

  const viewWidth2 = columnsPerPage * cw + (columnsPerPage - 1) * cg;

  const stepWidth = columnsPerPage * (cw + cg);
  containerWidth.value = stepWidth;

  const totalWidth = cols.scrollWidth;
  const maxOffset = Math.max(0, totalWidth - viewWidth2);

  totalPages.value = Math.max(
    1,
    Math.floor(maxOffset / stepWidth + 1e-3) + 1,
  );

  if (currentPage.value >= totalPages.value) {
    currentPage.value = totalPages.value - 1;
  }
};

const prevPage = () => {
  if (currentPage.value <= 0) return;
  currentPage.value -= 1;
};

const nextPage = () => {
  if (currentPage.value >= totalPages.value - 1) return;
  currentPage.value += 1;
};

const prevChapter = () => {
  const target = currentChapterIndex.value - 1;
  if (target < 0) return;
  pendingAnchor.value = null;
  const chapter = chapters.value[target];
  activeNavKey.value = makeChapterKey(chapter, target);
  loadChapter(target);
};

const nextChapter = () => {
  const target = currentChapterIndex.value + 1;
  if (target >= chapters.value.length) return;
  pendingAnchor.value = null;
  const chapter = chapters.value[target];
  activeNavKey.value = makeChapterKey(chapter, target);
  loadChapter(target);
};

const setReadingMode = (mode) => {
  if (readingMode.value === mode) return;
  readingMode.value = mode;

  if (mode === 'flip') {
    currentPage.value = 0;
  } else if (mode === 'scroll' && scrollContainer.value) {
    scrollContainer.value.scrollTop = 0;
  }

  nextTick(() => {
    calculatePagination();
    if (readingMode.value === 'scroll') {
      buildAnchorPositions();
      handleScroll();
    } else {
      updateActiveAnchorInFlip();
    }
  });
};

// ------- 监听 -------

watch([currentContent, readingMode], () => {
  nextTick(() => {
    calculatePagination();
    if (readingMode.value === 'scroll') {
      buildAnchorPositions();
      handleScroll();
    } else {
      updateActiveAnchorInFlip();
    }
  });
});

watch([currentPage, readingMode], () => {
  if (readingMode.value !== 'flip') return;
  nextTick(() => updateActiveAnchorInFlip());
});
</script>

<style scoped>
/* ===== 阅读器外壳：吃 library 表达 token；正文 .novel-content 维持站点自有阅读排版 ===== */
.reader-root {
  background: var(--sos-bg-page);
  color: var(--sos-text-primary);
}

.reader-loading {
  background: var(--sos-bg-page);
}
.reader-loading__text {
  color: var(--sos-text-tertiary);
}

.loader {
  border: 3px solid var(--sos-border-subtle);
  border-top-color: var(--sos-accent);
  border-radius: var(--sos-radius-full);
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

/* 目录侧栏 */
.reader-toc {
  background: var(--sos-bg-subtle);
  border-right: 1px solid var(--sos-border-subtle);
}
.reader-toc__title {
  margin: 0;
  font-family: var(--sos-display-family);
  font-size: var(--sos-text-lg);
  font-weight: var(--sos-weight-heavy);
  letter-spacing: var(--sos-tracking-wide);
  color: var(--sos-text-primary);
}
.reader-toc__item {
  display: flex;
  align-items: center;
  width: 100%;
  text-align: left;
  border-radius: var(--sos-radius-sm);
  color: var(--sos-text-secondary);
  background: transparent;
  transition:
    background-color var(--sos-duration-fast) var(--sos-ease-out),
    color var(--sos-duration-fast) var(--sos-ease-out);
}
.reader-toc__item--chapter {
  padding: var(--sos-space-2) var(--sos-space-3);
  margin-top: var(--sos-space-1);
  font-size: var(--sos-text-sm);
}
.reader-toc__item--sub {
  padding: var(--sos-space-1) var(--sos-space-3) var(--sos-space-1) var(--sos-space-7);
  font-size: var(--sos-text-xs);
}
.reader-toc__item:hover {
  background: var(--sos-bg-muted);
  color: var(--sos-text-primary);
}
.reader-toc__item.is-active {
  background: var(--sos-accent-soft);
  color: var(--sos-link);
  font-weight: var(--sos-weight-heavy);
}

/* 顶部工具栏 */
.reader-bar {
  flex-shrink: 0;
  border-bottom: 1px solid var(--sos-border-subtle);
  background: color-mix(in srgb, var(--sos-bg-page) 92%, transparent);
  backdrop-filter: blur(8px);
  z-index: var(--sos-z-raised);
}
.reader-titles {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.reader-titles__book {
  font-weight: var(--sos-weight-heavy);
  font-size: var(--sos-text-sm);
  color: var(--sos-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.reader-titles__chapter {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
@media (min-width: 768px) {
  .reader-titles__book {
    font-size: var(--sos-text-md);
  }
}

/* 通用图标按钮 */
.reader-icon-btn {
  display: inline-grid;
  place-items: center;
  padding: var(--sos-space-2);
  border-radius: var(--sos-radius-sm);
  color: var(--sos-text-secondary);
  background: transparent;
  transition:
    background-color var(--sos-duration-fast) var(--sos-ease-out),
    color var(--sos-duration-fast) var(--sos-ease-out);
}
.reader-icon-btn:hover:not(:disabled) {
  background: var(--sos-bg-muted);
  color: var(--sos-link);
}
.reader-icon-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

/* 分段切换（简繁 / 阅读模式）：奶纸底槽 + 选中浮起白面 */
.reader-seg {
  display: inline-flex;
  gap: 2px;
  padding: 3px;
  border-radius: var(--sos-radius-md);
  background: var(--sos-bg-muted);
  font-size: var(--sos-text-xs);
  font-weight: var(--sos-weight-heavy);
}
.reader-seg__btn {
  padding: var(--sos-space-1) var(--sos-space-2);
  border-radius: var(--sos-radius-sm);
  color: var(--sos-text-tertiary);
  background: transparent;
  white-space: nowrap;
  transition: all var(--sos-duration-fast) var(--sos-ease-out);
}
@media (min-width: 768px) {
  .reader-seg__btn {
    padding: var(--sos-space-1) var(--sos-space-3);
  }
}
.reader-seg__btn:hover {
  color: var(--sos-text-primary);
}
.reader-seg__btn.is-active {
  background: var(--sos-bg-surface);
  color: var(--sos-link);
  box-shadow: var(--sos-shadow-xs);
}

/* 内容区 */
.reader-content {
  background: var(--sos-bg-page);
}
/* 流式阅读纸面：玻璃奶纸承载，正文居中 */
.reader-paper {
  background: color-mix(in srgb, var(--sos-bg-surface) 55%, transparent);
  backdrop-filter: blur(4px);
  border-radius: var(--sos-radius-lg);
  box-shadow: var(--sos-shadow-sm);
}
.reader-chapnav {
  border-top: 1px solid var(--sos-border-subtle);
}
.reader-chapnav__btn {
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  transition: color var(--sos-duration-fast) var(--sos-ease-out);
}
.reader-chapnav__btn:hover:not(:disabled) {
  color: var(--sos-link);
}
.reader-chapnav__btn:disabled {
  opacity: 0.3;
  cursor: default;
}

/* 翻页页脚进度 */
.reader-progress {
  flex-shrink: 0;
  border-top: 1px solid var(--sos-border-subtle);
  background: var(--sos-bg-subtle);
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
}
.reader-progress__chapter {
  font-weight: var(--sos-weight-heavy);
  font-size: var(--sos-text-xs);
  color: var(--sos-link);
}
.reader-progress__track {
  height: 4px;
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-muted);
  overflow: hidden;
}
.reader-progress__fill {
  height: 100%;
  background: var(--sos-accent);
  transition: width var(--sos-duration-base) var(--sos-ease-out);
}

/* 翻页模式的书页卡片 */
.flip-page-frame {
  width: 100%;
  max-width: 64rem;
  height: calc(100vh - 160px);
  margin: 0 auto;
  padding: 2.5rem 3rem;
  background: color-mix(in srgb, var(--sos-bg-surface) 45%, transparent);
  backdrop-filter: blur(4px);
  border-radius: var(--sos-radius-md);
  box-shadow: var(--sos-shadow-sm);
  overflow: hidden;
}

/* 多栏内容 —— 会被整体左右平移 */
.flip-columns {
  height: 100%;
  column-width: 24rem;
  column-gap: 3rem;
  column-fill: auto;
  transition: transform 0.35s ease;
}

@media (max-width: 1024px) {
  .flip-page-frame {
    height: calc(100vh - 140px);
    padding: 2rem 1.75rem;
  }
  .flip-columns {
    column-width: 22rem;
    column-gap: 2rem;
  }
}

@media (max-width: 768px) {
  .flip-page-frame {
    height: calc(100vh - 130px);
    padding: 1.75rem 1.25rem;
  }
  .flip-columns {
    column-width: 18rem;
    column-gap: 1.5rem;
  }
}
</style>
