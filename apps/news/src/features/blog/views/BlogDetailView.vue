<template>
  <div
    v-if="article"
    class="animate-fade-in blog-detail-root"
  >
    <!-- ================= HEADER SECTION ================= -->
    <div
      class="hero-container"
    >
      <!-- CASE A: 有头图 (优先使用 originalImage) -->
      <div v-if="article.image" class="hero-image-wrapper">
        <img
          :src="article.originalImage || article.image"
          class="hero-image"
          :style="heroTransformStyle"
        />
        <div
          class="hero-gradient-overlay"
        ></div>
        <div class="hero-dark-overlay"></div>
      </div>

      <!-- CASE B: 无头图 -->
      <div
        v-else
        class="hero-no-image"
      >
        <div
          class="hero-text-pattern"
          style="filter: grayscale(100%);"
        >
          <span
            v-for="n in 40"
            :key="n"
            class="hero-pattern-text"
          >
            {{ n % 2 === 0 ? article.title : (article.tags?.[0] || 'HIBIKILOGY')
            }}
          </span>
        </div>
        <div
          class="hero-no-image-gradient"
        ></div>
      </div>

      <div
        class="hero-content-positioner"
      >
        <!-- 标签行 -->
        <div
          class="hero-tags-row"
          style="left: 105px; top: 283px;"
        >
          <div class="hero-tags-list">
            <span
              v-for="tag in article.tags"
              :key="tag"
              @click.stop="$router.push(`/tag/${tag}`)"
              class="hero-tag-item"
            >
              <span
                class="hero-tag-hash"
                >#</span
              >
              <span
                class="hero-tag-name"
              >
                {{ tag }}
              </span>
            </span>
          </div>
          <span
            v-if="article.type === 'news'"
            class="hero-news-badge"
          >
            NEWS
          </span>
        </div>

        <!-- 标题 + 副标题 + 作者块 -->
        <div
          class="hero-title-block"
          style="left: 105px; top: 329px; width: 1200px;"
        >
          <div>
            <h1
              class="hero-title text-shadow"
            >
              {{ article.title }}
            </h1>

            <p
              v-if="article.subtitle"
              class="hero-subtitle text-shadow"
            >
              {{ article.subtitle }}
            </p>
          </div>

          <div
            class="hero-author-row text-shadow"
          >
            <div
              class="hero-author-info"
            >
              <span class="hero-author-label">作者</span>

              <span
                class="hero-author-name"
                @click="$router.push(article.authorUserId && article.type !== 'news' ? `/author/u${article.authorUserId}` : `/author/${article.author || '凉宫春日应援团'}`)"
              >
                {{ article.author || '凉宫春日应援团' }}
              </span>
            </div>
            <div class="hero-date-info">
              发表于 {{ article.date }}
            </div>
          </div>
        </div>

        <!-- ========= 移动端：自适应布局 ========= -->
        <div
          class="hero-mobile-block"
        >
          <div class="hero-mobile-tags">
            <div class="hero-mobile-tags-list">
              <span
                v-for="tag in article.tags"
                :key="tag"
                @click.stop="$router.push(`/tag/${tag}`)"
                class="hero-mobile-tag-item"
              >
                <span
                  class="hero-mobile-tag-hash"
                  >#</span
                >
                <span
                  class="hero-mobile-tag-name"
                >
                  {{ tag }}
                </span>
              </span>
            </div>
          </div>
          <h1 class="hero-mobile-title text-shadow">
            {{ article.title }}
          </h1>
          <p
            v-if="article.subtitle"
            class="hero-mobile-subtitle text-shadow"
          >
            {{ article.subtitle }}
          </p>

          <div
            class="hero-mobile-meta text-shadow"
          >
            <span class="hero-mobile-meta-label">作者</span>

            <span
                class="hero-mobile-author-name"
                @click="$router.push(article.authorUserId && article.type !== 'news' ? `/author/u${article.authorUserId}` : `/author/${article.author || '凉宫春日应援团'}`)"
            >{{ article.author || '凉宫春日应援团' }}</span>
            <span class="hero-mobile-date">· 发表于 {{ article.date }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- ================= CONTENT BODY ================= -->
    <div
      class="content-outer"
    >
      <div
        class="content-grid"
      >
        <!-- Main Content (Left) -->
        <article
          class="content-main content-renderer prose prose-lg"
        >
          <!-- 参与者信息 -->
          <div
            v-if="
              article.type === 'news' &&
              article.participants &&
              article.participants.length
            "
            class="participants-box"
          >
            <h2
              class="participants-heading"
            >
              参与者信息
            </h2>
            <ul class="participants-list">
              <li
                v-for="(p, idx) in article.participants"
                :key="idx"
                class="participant-item"
              >
                <span class="participant-name" @click="$router.push(`/participant/${p.name}`)">{{ p.name }}</span>
                <span class="participant-role">
                  {{ p.role }}
                  <span v-if="p.project"> · {{ p.project }}</span>
                </span>
              </li>
            </ul>
          </div>

          <!-- 正文内容 -->
          <div
            v-for="(block, index) in article.content"
            :key="index"
            class="content-block"
          >
            <p
              v-if="block.type === 'paragraph'"
              class="article-paragraph"
              v-html="formatParagraph(block.text)"
            ></p>

            <h3
              v-else-if="block.type === 'heading'"
              :id="'heading-' + index"
              class="content-heading"
            >
              {{ block.text }}
            </h3>

            <div
              v-else-if="block.type === 'math'"
              class="math-block"
            >
              <span class="math-expression">$$ {{ block.expression }} $$</span>
              <p
                v-if="block.caption"
                class="math-caption"
              >
                {{ block.caption }}
              </p>
            </div>

            <figure v-else-if="block.type === 'image'" class="image-figure">
              <div class="image-wrapper">
                <img
                  :src="block.src"
                  :alt="block.caption"
                  class="image-content"
                />
              </div>
              <figcaption
                v-if="block.caption"
                class="image-caption"
              >
                <span class="image-caption-dot"></span>
                {{ block.caption }}
              </figcaption>
            </figure>
          </div>

          <div
            class="fin-divider"
          >
            <span class="fin-line"></span>
            <span class="fin-text">Fin</span>
            <span class="fin-line"></span>
          </div>
        </article>

        <!-- Sidebar / TOC -->
        <aside
          class="sidebar"
        >
          <div class="sidebar-inner">
            <h4
              class="sidebar-title"
            >
              Catalog
            </h4>

            <nav v-if="toc.length > 0">
              <ul
                class="toc-list"
              >
                <li
                  v-for="(item, idx) in toc"
                  :key="idx"
                  class="toc-item"
                >
                  <span
                    class="toc-dot"
                  ></span>
                  <a
                    :href="'#heading-' + item.index"
                    @click.prevent="scrollToHeading(item.index)"
                    class="toc-link"
                  >
                    {{ item.text }}
                  </a>
                </li>
              </ul>
            </nav>
            <p v-else class="toc-empty">
              本文无小标题
            </p>

            <div class="sidebar-stats">
              <div class="sidebar-stats-inner">
                <p>
                  字数统计:
                  <span class="sidebar-stats-value">{{ wordCount }}</span> 字
                </p>
                <p>
                  预计阅读:
                  <span class="sidebar-stats-value">
                    {{ Math.ceil(wordCount / 400) }}
                  </span>
                  分钟
                </p>
              </div>
            </div>
          </div>
        </aside>
      </div>
    </div>

    <!-- Footer -->
    <div class="blog-footer">
      <div
        class="blog-footer-inner"
      >
        <button
          @click="$router.push('/')"
          class="footer-back-btn"
        >
          <span
            class="footer-back-icon"
          >
            &larr;
          </span>
          <div class="footer-back-text">
            <div
              class="footer-back-label"
            >
              Return
            </div>
            <div
              class="footer-back-title"
            >
              首页
            </div>
          </div>
        </button>

        <div
          class="footer-actions"
        >
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { usePageMeta, canonicalUrl, absoluteUrl } from '@haruhi/seo';
import { useMainStore } from '@/stores/main';

const route = useRoute();
const store = useMainStore();

/**
 * 这里不再直接从 store.allArticles 里拿，
 * 而是本地维护一个 article ref，并确保它是「完整正文」。
 */
const article = ref(null);

// 目录
const toc = computed(() => {
  if (!article.value || !article.value.content) return [];
  return article.value.content
    .map((block, index) => ({ ...block, index }))
    .filter((block) => block.type === 'heading');
});

// 字数统计
const wordCount = computed(() => {
  if (!article.value || !article.value.content) return 0;
  return article.value.content.reduce((acc, block) => {
    return acc + (block.text ? block.text.length : 0);
  }, 0);
});

// 使用裁切时记录的几何中心，作为完整头图缩放的锚点
const heroTransformStyle = computed(() => {
  if (!article.value) return {};

  const clamp01 = (v, fallback = 0.5) => {
    const n = Number(v);
    if (!isFinite(n)) return fallback;
    return Math.min(1, Math.max(0, n));
  };

  const x = clamp01(article.value.coverFocalX, 0.5);
  const y = clamp01(article.value.coverFocalY, 0.5);

  const xPercent = x * 100;
  const yPercent = y * 100;

  return {
    objectPosition: `${xPercent}% ${yPercent}%`,
    transformOrigin: `${xPercent}% ${yPercent}%`,
  };
});

// 滚动到标题
const scrollToHeading = (index) => {
  const el = document.getElementById(`heading-${index}`);
  if (el) {
    const y = el.getBoundingClientRect().top + window.pageYOffset - 100;
    window.scrollTo({ top: y, behavior: 'smooth' });
  }
};

const escapeHtml = (str) => {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
};

// 解析 Markdown 样式的简单函数
const parseInlineStyles = (html) => {
  return html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s>$1</s>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="inline-link">$1</a>');
};

const formatParagraph = (text) => {
  if (!text) return '';

  const lines = text
    .toString()
    .split('\n')
    .map((line) => line.replace(/^[\s\u3000]+/, ''));

  const htmlLines = lines.map((line) => {
    const escaped = escapeHtml(line);
    const styled = parseInlineStyles(escaped);

    return `<span class="para-line">${
      styled === '' ? '&nbsp;' : styled
    }</span>`;
  });

  return htmlLines.join('');
};

/**
 * 核心：加载当前路由对应文章
 * 1. 先从 store 里找（如果已经是完整内容 `!isContentPreview` 直接用）
 * 2. 如果只有预览版，先用预览填充画面，再请求完整版替换
 * 3. 如果 store 里根本没有，就直接请求完整版
 */
const loadArticle = async () => {
  const id = route.params.id;
  if (!id) return;

  // 尝试从列表缓存中获取
  const cached = store.allArticles.find((a) => a.id == id);

  if (cached && cached.isContentPreview === false) {
    // 已经是完整版本
    article.value = cached;
    return;
  }

  if (cached) {
    // 只有预览版本：先用预览占位
    article.value = cached;
  } else {
    article.value = null;
  }

  // 无论如何，再从后端拉一次完整文章
  const full = await store.fetchArticleById(id);
  if (full) {
    article.value = full;
  }
};

onMounted(loadArticle);

// 路由切换到其它文章时，重新加载
watch(
  () => route.params.id,
  () => {
    article.value = null;
    loadArticle();
  }
);

// 描述截断至约 160 字（meta description 长度惯例）
const truncate = (text, max = 160) => {
  if (!text) return '';
  const s = String(text).replace(/\s+/g, ' ').trim();
  return s.length > max ? s.slice(0, max - 1) + '…' : s;
};

// 页面 meta：文章数据就绪后设置（加载前返回 null，保留静态兜底标签）
usePageMeta(() =>
  article.value
    ? {
        title: `${article.value.title} · 春日团报`,
        description: truncate(article.value.summary || article.value.subtitle) || undefined,
        canonical: canonicalUrl(`/blog/${route.params.id}`),
        ogType: 'article',
        ogImage: article.value.image ? absoluteUrl(article.value.image) : undefined,
        jsonLd: {
          '@context': 'https://schema.org',
          '@type': 'NewsArticle',
          headline: article.value.title,
          datePublished: article.value.date || article.value.created_at || undefined,
          author: { '@type': 'Person', name: article.value.author || '凉宫春日应援团' },
          ...(article.value.image ? { image: absoluteUrl(article.value.image) } : {}),
        },
      }
    : null,
);
</script>

<style scoped>
/* ===== Preserved original styles ===== */
.text-shadow {
  text-shadow: 0px 6px 22.5px rgba(0, 0, 0, 0.3);
}

.article-paragraph {
  margin: 0;
  line-height: 1.625;
  text-align: justify;
  font-size: 22px;
  letter-spacing: 0.025em;
  color: var(--sos-text-primary);
}

.article-paragraph :deep(.para-line) {
  display: block;
  text-indent: 2em;
}

.article-paragraph :deep(b) { font-weight: bold; }
.article-paragraph :deep(i) { font-style: italic; }
.article-paragraph :deep(u) { text-decoration: underline; }
.article-paragraph :deep(s) { text-decoration: line-through; }
.article-paragraph :deep(.inline-link) {
  color: #2563eb;
}
.article-paragraph :deep(.inline-link:hover) {
  text-decoration: underline;
}

/* ===== Root ===== */
.blog-detail-root {
  background-color: var(--sos-bg-surface);
  min-height: 100vh;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  color: var(--sos-text-primary);
}

/* ===== Hero Section ===== */
.hero-container {
  position: relative;
  width: 100%;
  height: 600px;
  background-color: var(--sos-text-tertiary);
  overflow: hidden;
  user-select: none;
}

.hero-image-wrapper {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
}

.hero-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 3s ease-out;
}

.hero-image:hover {
  transform: scale(1.05);
}

.hero-gradient-overlay {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to bottom, rgba(0,0,0,0.3), transparent, rgba(0,0,0,0.6));
}

.hero-dark-overlay {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background-color: rgba(0,0,0,0.1);
}

/* Hero - No Image */
.hero-no-image {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
  background-color: var(--sos-text-tertiary);
  overflow: hidden;
}

.hero-text-pattern {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  justify-content: center;
  gap: 2rem;
  opacity: 0.05;
  pointer-events: none;
  transform: rotate(-12deg) scale(1.25);
}

.hero-pattern-text {
  font-size: 4.5rem;
  font-weight: 900;
  color: var(--sos-text-primary);
  white-space: nowrap;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
}

.hero-no-image-gradient {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.063), transparent);
}

/* Hero Content Positioner */
.hero-content-positioner {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  top: 0;
  width: 100%;
  max-width: 1600px;
  height: 100%;
  z-index: 10;
}

/* Hero Tags Row (Desktop) */
.hero-tags-row {
  display: none;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}

.hero-tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.hero-tag-item {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.hero-tag-hash {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: rgba(255,255,255,0.5);
  margin-right: 0.25rem;
}

.hero-tag-name {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: var(--sos-bg-surface);
}

.hero-tag-name:hover {
  text-decoration: underline;
  text-decoration-thickness: 2px;
  text-underline-offset: 4px;
}

.hero-news-badge {
  margin-left: 0.5rem;
  border: 1px solid rgba(255,255,255,0.6);
  padding: 0.125rem 0.5rem;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.1em;
}

/* Hero Title Block (Desktop) */
.hero-title-block {
  display: none;
  flex-direction: column;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}

.hero-title {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
  font-weight: 900;
  font-size: 80px;
  line-height: 90px;
  letter-spacing: 0;
  margin-bottom: 0;
  color: var(--sos-bg-surface);
}

.hero-subtitle {
  margin-top: 6px;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 27px;
  line-height: 45px;
  color: var(--sos-bg-surface);
}

.hero-author-row {
  margin-top: 1rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  column-gap: 1rem;
  row-gap: 0.25rem;
}

.hero-author-info {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-size: 24px;
  line-height: 45px;
  color: var(--sos-bg-surface);
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.hero-author-label {
  font-weight: 400;
  opacity: 0.8;
}

.hero-author-name {
  font-weight: 700;
  cursor: pointer;
}

.hero-author-name:hover {
  text-decoration: underline;
}

.hero-date-info {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-size: 21px;
  line-height: 45px;
  color: rgba(255,255,255,0.9);
}

/* Hero Mobile Block */
.hero-mobile-block {
  position: absolute;
  left: 1rem;
  right: 1rem;
  bottom: 2.5rem;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
}

.hero-mobile-tags {
  margin-bottom: 0.5rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.hero-mobile-tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.hero-mobile-tag-item {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.hero-mobile-tag-hash {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: rgba(255,255,255,0.5);
  margin-right: 0.25rem;
}

.hero-mobile-tag-name {
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-bg-surface);
}

.hero-mobile-tag-name:hover {
  text-decoration: underline;
}

.hero-mobile-title {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
  font-weight: 900;
  font-size: 1.5rem;
  line-height: 1.375;
}

.hero-mobile-subtitle {
  margin-top: 0.25rem;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.hero-mobile-meta {
  margin-top: 0.75rem;
  font-size: 0.75rem;
  line-height: 1.25rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  column-gap: 0.5rem;
  row-gap: 0.25rem;
}

.hero-mobile-meta-label {
  opacity: 0.8;
}

.hero-mobile-author-name {
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
}

.hero-mobile-date {
  opacity: 0.8;
}

/* ===== Content Body ===== */
.content-outer {
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  position: relative;
  margin-top: 61px;
}

.content-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 3rem;
}

/* Main Article Content */
.content-main {
  padding-top: 0;
  padding-bottom: 3rem;
  max-width: none;
}

/* prose modifier overrides (scoped) */
.content-main :deep(h1),
.content-main :deep(h2),
.content-main :deep(h3),
.content-main :deep(h4),
.content-main :deep(h5),
.content-main :deep(h6) {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
}

.content-main :deep(p) {
  color: var(--sos-text-secondary);
}

.content-main :deep(img) {
  border-radius: 0.125rem;
}

/* Participants Box */
.participants-box {
  margin-bottom: 2.5rem;
  border-radius: 0.5rem;
  border: 1px solid var(--sos-bg-muted);
  background-color: rgba(249,250,251,0.8);
  padding: 1.5rem;
  padding-top: 1.25rem;
  padding-bottom: 1.25rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}

.participants-heading {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  letter-spacing: 0.25em;
  color: var(--sos-text-secondary);
  text-transform: uppercase;
  margin-bottom: 0.75rem;
}

.participants-list {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
}

.participants-list > * + * {
  margin-top: 0.5rem;
}

.participant-item {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  column-gap: 0.5rem;
  row-gap: 0.125rem;
}

.participant-name {
  font-weight: 600;
  cursor: pointer;
  color: #1e3a8a;
}

.participant-name:hover {
  text-decoration: underline;
}

.participant-role {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-secondary);
}

/* Content Blocks */
.content-block {
  margin-bottom: 2rem;
}

.content-heading {
  font-size: 1.5rem;
  font-weight: 700;
  margin-top: 4rem;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e5e5;
  scroll-margin-top: 8rem;
  color: var(--sos-text-primary);
  position: relative;
}

/* Math Block */
.math-block {
  margin-top: 2.5rem;
  margin-bottom: 2.5rem;
  padding: 2rem;
  background-color: #f9f9f9;
  border-left: 4px solid var(--sos-text-tertiary);
  text-align: center;
  overflow-x: auto;
}

.math-expression {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
  font-size: 1.25rem;
}

.math-caption {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  margin-top: 1rem;
  font-style: normal;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
}

/* Image Figure */
.image-figure {
  margin-top: 3rem;
  margin-bottom: 3rem;
}

.image-wrapper {
  position: relative;
  overflow: hidden;
}

.image-content {
  width: 100%;
  height: auto;
  display: block;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  border: 1px solid var(--sos-bg-muted);
}

.image-caption {
  text-align: center;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-secondary);
  margin-top: 0.75rem;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  letter-spacing: 0.025em;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.25rem;
}

.image-caption-dot {
  width: 0.25rem;
  height: 0.25rem;
  background-color: var(--sos-text-tertiary);
  border-radius: 9999px;
}

/* Fin Divider */
.fin-divider {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-top: 5rem;
  margin-bottom: 5rem;
  opacity: 0.3;
}

.fin-line {
  height: 1px;
  width: 3rem;
  background-color: var(--sos-text-primary);
}

.fin-text {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
  font-style: italic;
  font-size: 1.125rem;
}

/* ===== Sidebar / TOC ===== */
.sidebar {
  display: none;
}

.sidebar-inner {
  position: sticky;
  top: 8rem;
  padding-top: 3rem;
  padding-bottom: 3rem;
}

.sidebar-title {
  font-weight: 700;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  color: var(--sos-text-tertiary);
  margin-bottom: 2rem;
}

.toc-list {
  border-left: 2px solid var(--sos-bg-muted);
  margin-left: 0.25rem;
  padding-left: 1.25rem;
  position: relative;
}

.toc-list > * + * {
  margin-top: 1.25rem;
}

.toc-item {
  position: relative;
}

.toc-dot {
  position: absolute;
  left: -23px;
  top: 6px;
  width: 6px;
  height: 6px;
  border-radius: 9999px;
  background-color: var(--sos-border-strong);
  transition: background-color 0.15s;
}

.toc-item:hover .toc-dot {
  background-color: var(--sos-text-primary);
}

.toc-link {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  display: block;
  line-height: 1.625;
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
  transition: all 0.15s;
}

.toc-link:hover {
  color: var(--sos-text-primary);
  font-weight: 700;
}

.toc-empty {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-border-strong);
  font-style: italic;
  padding-left: 1.5rem;
}

.sidebar-stats {
  margin-top: 4rem;
  padding-top: 2rem;
  border-top: 1px solid var(--sos-bg-muted);
}

.sidebar-stats-inner {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-tertiary);
  font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif);
}

.sidebar-stats-inner > * + * {
  margin-top: 0.5rem;
}

.sidebar-stats-value {
  color: var(--sos-text-primary);
}

/* ===== Footer ===== */
.blog-footer {
  border-top: 1px solid var(--sos-text-primary);
  background-color: var(--sos-bg-surface);
  padding-top: 4rem;
  padding-bottom: 4rem;
}

.blog-footer-inner {
  max-width: 1200px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-back-btn {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.footer-back-icon {
  width: 3rem;
  height: 3rem;
  border: 1px solid var(--sos-border-default);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s;
}

.footer-back-btn:hover .footer-back-icon {
  background-color: var(--sos-text-primary);
  border-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
}

.footer-back-text {
  text-align: left;
}

.footer-back-label {
  font-size: 10px;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.footer-back-title {
  font-family: var(--font-serif, ui-serif, Georgia, serif);
  font-weight: 700;
  font-size: 1.125rem;
  color: var(--sos-text-primary);
}

.footer-back-btn:hover .footer-back-title {
  text-decoration: underline;
  text-decoration-thickness: 1px;
  text-underline-offset: 4px;
}

.footer-actions {
  display: flex;
  gap: 0.75rem;
  opacity: 0.5;
  transition: opacity 0.15s;
}

.footer-actions:hover {
  opacity: 1;
}

/* ===== Responsive: md (min-width: 768px) ===== */
@media (min-width: 768px) {
  .hero-tags-row {
    display: flex;
  }

  .hero-title-block {
    display: flex;
  }

  .hero-mobile-block {
    display: none;
  }

  .content-outer {
    padding-left: 2rem;
    padding-right: 2rem;
  }

  .content-main {
    padding-top: 0;
    padding-bottom: 5rem;
  }

  .article-paragraph {
    font-size: 23px;
  }

  .blog-footer-inner {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

/* ===== Responsive: lg (min-width: 1024px) ===== */
@media (min-width: 1024px) {
  .content-outer {
    padding-left: 0;
    padding-right: 0;
  }

  .content-grid {
    grid-template-columns: repeat(12, minmax(0, 1fr));
    padding-left: 105px;
    padding-right: 2.5rem;
  }

  .content-main {
    grid-column: span 9 / span 9;
  }

  .sidebar {
    display: block;
    grid-column: span 3 / span 3;
    padding-left: 2rem;
    border-left: 1px solid var(--sos-bg-muted);
  }
}
</style>
