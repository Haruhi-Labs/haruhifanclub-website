<template>
  <div class="article-tab">
    <table class="data-table">
      <thead class="article-thead">
        <tr class="article-thead-row">
          <th class="ath-id">ID / Type</th>
          <th class="ath-title">标题 & 摘要</th>
          <th class="ath-author">作者</th>
          <th class="ath-date">日期</th>
          <th class="ath-status">状态</th>
          <th class="ath-actions">操作</th>
        </tr>
      </thead>
      <tbody class="table-body">
        <tr v-for="article in currentList" :key="article.id" class="article-row group">
          <td class="atd-id">
            <div class="article-id">#{{ article.id }}</div>
            <span v-if="article.type === 'news'" class="type-badge-news">News</span>
            <span v-else class="type-badge-article">Article</span>
          </td>
          <td class="atd-title">
            <div class="article-title-text">{{ article.title }}</div>
            <div class="article-summary-text">{{ article.subtitle || article.summary || '无摘要...' }}</div>
          </td>
          <td class="atd-author">
            <div class="author-name">{{ article.author || '凉宫春日应援团' }}</div>
            <div class="author-participants" v-if="article.participants?.length">
              +{{ article.participants.length }} 位参与者
            </div>
          </td>
          <td class="atd-date">
            {{ article.date }}
          </td>
          <td class="atd-status">
            <div v-if="article.status === 'pending'" class="status-pending">
              <span class="status-dot-pending"></span>
              Pending
            </div>
            <div v-else class="status-published-col">
              <span class="status-published">
                <span class="status-dot-published"></span>
                Published
              </span>
              <span v-if="article.isPinned" class="pinned-label">
                <svg class="icon-xs" fill="currentColor" viewBox="0 0 20 20"><path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z"/></svg>
                PINNED ({{ article.pinOrder }})
              </span>
            </div>
          </td>
          <td class="atd-actions-cell">
            <div class="action-btns">
              <button @click="openPreview(article)" class="action-btn-preview" title="预览">
                <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
              </button>
              <div class="action-divider"></div>
              <button v-if="article.status === 'pending'" @click="approveArticle(article)" class="action-btn-approve" title="通过审核">通过</button>
              <button @click="editArticle(article.id)" class="action-btn-edit" title="编辑">编辑</button>
              <button @click="confirmDelete(article)" class="action-btn-delete" title="删除">删除</button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <Teleport to="body">
      <!-- ================= Preview Overlay (全功能预览) ================= -->
      <Transition name="fade">
        <div v-if="previewArticle" class="preview-overlay animate-fade-in-fast">
          <!-- Preview Header -->
          <div class="preview-header">
            <div class="preview-header-left">
              <div class="preview-header-info">
                <span class="preview-mode-label">Preview Mode</span>
                <h2 class="preview-title serif-font">{{ previewArticle.title }}</h2>
              </div>

              <!-- View Switcher -->
              <div class="preview-switcher">
                <button
                    @click="previewMode = 'card'"
                    :class="previewMode === 'card' ? 'preview-tab-active' : 'preview-tab-inactive'"
                    class="preview-tab-btn"
                >
                  NewsCard
                </button>
                <button
                    @click="previewMode = 'modal'"
                    :class="previewMode === 'modal' ? 'preview-tab-active' : 'preview-tab-inactive'"
                    class="preview-tab-btn"
                >
                  弹窗详情
                </button>
                <button
                    @click="previewMode = 'page'"
                    :class="previewMode === 'page' ? 'preview-tab-active' : 'preview-tab-inactive'"
                    class="preview-tab-btn"
                >
                  正文阅读页
                </button>
              </div>
            </div>

            <div class="preview-header-right">
              <span v-if="previewArticle.status === 'pending'" class="preview-pending-badge">当前状态: 待审核</span>
              <button @click="closePreview" class="preview-close-btn">
                <svg class="icon-md" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
              </button>
            </div>
          </div>

          <!-- Preview Body -->
          <div class="preview-body">

            <!-- 1. Card Mode -->
            <div v-if="previewMode === 'card'" class="preview-card-mode">
              <!-- 模拟 Masonry 列宽 -->
              <div class="preview-card-wrap">
                <NewsCard :article="previewArticle" class="preview-card-component" />
                <p class="preview-card-caption">Mobile Card View (Masonry)</p>
              </div>
            </div>

            <!-- 2. Modal Mode (复刻 DetailModal.vue) -->
            <div v-else-if="previewMode === 'modal'" class="preview-modal-mode">
              <div class="preview-modal-container">
                <!-- Modal Header -->
                <div class="preview-modal-header">
                  <div class="preview-modal-header-label">Preview Mode</div>
                  <div class="preview-modal-close-placeholder"><svg class="icon-md" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg></div>
                </div>
                <div class="preview-modal-body">
                  <div class="preview-modal-meta">
                    <div class="preview-modal-tags">
                      <span v-for="tag in (previewArticle.tags || [])" :key="tag" class="tag-filled">{{ tag }}</span>
                      <span v-if="previewArticle.type === 'news'" class="tag-outline">News</span>
                    </div>
                    <h2 class="preview-modal-article-title serif-font">{{ previewArticle.title }}</h2>
                    <div class="preview-modal-info-row">
                      <span>{{ previewArticle.date }}</span>
                      <span v-if="previewArticle.type !== 'news'" class="preview-modal-author">By {{ previewArticle.author || '凉宫春日应援团' }}</span>
                    </div>
                    <!-- Participants (News) -->
                    <div v-if="previewArticle.type === 'news' && previewArticle.participants?.length" class="preview-modal-participants">
                      <p class="participants-label">PARTICIPANTS:</p>
                      <div class="participants-list">
                        <div v-for="(p, idx) in previewArticle.participants" :key="idx">
                          <span class="participant-name">{{ p.name }}</span>
                          <span class="participant-role"> — {{ p.role }} <span v-if="p.project">({{ p.project }})</span></span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div v-if="previewArticle.image" class="preview-modal-image">
                    <img :src="previewArticle.image" class="preview-modal-img">
                  </div>

                  <!-- Renderer (带截断) -->
                  <div class="content-renderer prose prose-lg preview-modal-content">
                    <div v-for="(block, index) in modalBlocks" :key="index">
                      <p v-if="block.type === 'paragraph'" v-html="formatModalParagraph(block.text)"></p>
                      <h3 v-else-if="block.type === 'heading'">{{ block.text }}</h3>
                      <div v-else-if="block.type === 'math'" class="math-block-modal">$$ {{ block.expression }} $$</div>
                      <div v-else-if="block.type === 'image'" class="image-block-modal"><img :src="block.src" class="image-block-modal-img"></div>
                    </div>
                    <p v-if="isModalTruncated" class="truncation-indicator">......</p>
                  </div>

                  <div class="preview-modal-footer-actions">
                    <button v-if="isModalTruncated" class="btn-read-full">
                      <span>完整阅读</span>
                      <svg class="icon-sm" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path></svg>
                    </button>
                    <button v-else class="btn-open-page">
                      <span>在新页面打开</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 3. Page Mode (复刻 BlogDetailView.vue) -->
            <div v-else-if="previewMode === 'page'" class="preview-page-mode animate-fade-in">
              <!-- Header -->
              <div class="page-hero">
                 <!-- Cover -->
                 <div v-if="previewArticle.image" class="page-hero-cover">
                  <img :src="previewArticle.originalImage || previewArticle.image" class="page-hero-img">
                  <div class="page-hero-gradient-main"></div>
                  <div class="page-hero-overlay"></div>
                 </div>
                 <!-- No Cover Fallback -->
                 <div v-else class="page-hero-fallback">
                   <div class="page-hero-pattern" style="filter: grayscale(100%);">
                     <span v-for="n in 40" :key="n" class="page-hero-pattern-text">{{ n % 2 === 0 ? previewArticle.title : 'HIBIKILOGY' }}</span>
                   </div>
                   <div class="page-hero-gradient-fallback"></div>
                 </div>

                 <!-- Header Content -->
                 <div class="page-hero-content">
                   <div class="page-tags-desktop" style="left: 105px; top: 283px;">
                     <div class="page-tags-wrap">
                       <span v-for="tag in previewArticle.tags" :key="tag" class="page-tag-item">
                         <span class="page-tag-hash">#</span>
                         <span class="page-tag-text">{{ tag }}</span>
                       </span>
                     </div>
                     <span v-if="previewArticle.type === 'news'" class="page-news-badge">NEWS</span>
                   </div>

                   <div class="page-title-desktop" style="left: 105px; top: 329px; width: 1200px;">
                     <div>
                       <h1 class="page-main-title text-shadow">{{ previewArticle.title }}</h1>
                       <p v-if="previewArticle.subtitle" class="page-subtitle text-shadow">{{ previewArticle.subtitle }}</p>
                     </div>
                     <div class="page-author-row text-shadow">
                       <div class="page-author-info">
                         <span class="page-author-label">作者</span>
                         <span class="page-author-name">{{ previewArticle.author || '凉宫春日应援团' }}</span>
                       </div>
                       <div class="page-date-info">发表于 {{ previewArticle.date }}</div>
                     </div>
                   </div>

                   <!-- Mobile Header -->
                   <div class="page-mobile-header">
                     <div class="page-mobile-tags">
                       <span v-for="tag in previewArticle.tags" :key="tag" class="page-mobile-tag">#{{ tag }}</span>
                     </div>
                     <h1 class="page-mobile-title text-shadow">{{ previewArticle.title }}</h1>
                     <p class="page-mobile-subtitle text-shadow">{{ previewArticle.subtitle }}</p>
                   </div>
                 </div>
              </div>

              <!-- Content Body -->
               <div class="page-content-container">
                <div class="page-content-grid">
                  <!-- Main -->
                  <article class="page-article content-renderer prose prose-lg">
                    <div v-if="previewArticle.type === 'news' && previewArticle.participants?.length" class="page-participants-box">
                      <h2 class="page-participants-heading">参与者信息</h2>
                      <ul class="page-participants-list">
                        <li v-for="(p, idx) in previewArticle.participants" :key="idx" class="page-participant-item">
                          <span class="participant-name">{{ p.name }}</span>
                          <span class="page-participant-detail">{{ p.role }} <span v-if="p.project">· {{ p.project }}</span></span>
                        </li>
                      </ul>
                    </div>

                    <div v-for="(block, index) in previewArticle.content" :key="index" class="page-content-block">
                      <p v-if="block.type === 'paragraph'" class="article-paragraph page-paragraph" v-html="formatPageParagraph(block.text)"></p>
                      <h3 v-else-if="block.type === 'heading'" :id="'heading-' + index" class="page-heading">{{ block.text }}</h3>
                      <div v-else-if="block.type === 'math'" class="page-math-block">
                        <span class="page-math-expr">$$ {{ block.expression }} $$</span>
                        <p v-if="block.caption" class="page-math-caption">{{ block.caption }}</p>
                      </div>
                      <figure v-else-if="block.type === 'image'" class="page-image-figure">
                        <div class="page-image-wrap">
                          <img :src="block.src" class="page-image">
                        </div>
                        <figcaption v-if="block.caption" class="page-image-caption">
                          <span class="caption-dot"></span>{{ block.caption }}
                        </figcaption>
                      </figure>
                    </div>
                    <div class="page-fin">
                      <span class="fin-line"></span><span class="fin-text">Fin</span><span class="fin-line"></span>
                    </div>
                  </article>
                  <!-- Sidebar -->
                  <aside class="page-sidebar">
                     <div class="page-sidebar-inner">
                       <h4 class="page-sidebar-title">Catalog</h4>
                       <nav v-if="toc.length > 0">
                         <ul class="page-toc-list">
                           <li v-for="(item, idx) in toc" :key="idx" class="page-toc-item">
                             <span class="page-toc-dot"></span>
                             <a :href="'#heading-' + item.index" @click.prevent="scrollToHeading(item.index)" class="page-toc-link">{{ item.text }}</a>
                           </li>
                         </ul>
                       </nav>
                       <p v-else class="page-toc-empty">本文无小标题</p>
                       <div class="page-sidebar-stats">
                         <div class="page-sidebar-stats-inner">
                           <p>字数统计: <span class="page-stat-val">{{ wordCount }}</span> 字</p>
                           <p>预计阅读: <span class="page-stat-val">{{ Math.ceil(wordCount / 400) }}</span> 分钟</p>
                         </div>
                       </div>
                     </div>
                  </aside>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useMainStore } from '@/stores/main';
import NewsCard from '@/features/blog/components/NewsCard.vue';

// mode: 'pending' | 'published'（由后台壳的 tab 传入），决定文章列表过滤
const props = defineProps({
    mode: { type: String, default: 'pending' }
});

const store = useMainStore();
const router = useRouter();

// Preview State
const previewArticle = ref(null);
const previewMode = ref('card');

const currentList = computed(() => {
    if (props.mode === 'pending') {
        return store.adminArticles.filter(a => a.status === 'pending');
    } else {
        return store.adminArticles.filter(a => !a.status || a.status === 'published');
    }
});

// Article Actions
const editArticle = (id) => {
    router.push({ path: '/submit', query: { id } });
};

const approveArticle = async (article) => {
    if (confirm(`确认发布文章 "${article.title}" 吗？`)) {
        await store.updateArticle(article.id, { status: 'published' });
    }
};

const confirmDelete = async (article) => {
    const actionName = article.status === 'pending' ? '拒绝' : '删除';
    if (confirm(`确定要${actionName}文章 "${article.title}" 吗？此操作无法撤销。`)) {
        await store.deleteArticle(article.id);
    }
};

// Preview Logic
const openPreview = (article) => {
    previewArticle.value = article;
    previewMode.value = 'card';
    document.body.style.overflow = 'hidden';
};

const closePreview = () => {
    previewArticle.value = null;
    document.body.style.overflow = '';
};

// --- Helper Functions for Rendering ---

// 1. Text Formatters
const escapeHtml = (str) => str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const parseInlineStyles = (html) => html
    .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>')
    .replace(/\*(.*?)\*/g, '<i>$1</i>')
    .replace(/__(.*?)__/g, '<u>$1</u>')
    .replace(/~~(.*?)~~/g, '<s>$1</s>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="inline-link">$1</a>');

// For Modal View (Standard <br>)
const formatModalParagraph = (text) => {
  if (!text) return '';
  const escaped = escapeHtml(text.toString());
  const styled = parseInlineStyles(escaped);
  return styled.replace(/\n/g, '<br>'); // Simple break
};

// For Page View (Indented Paragraphs)
const formatPageParagraph = (text) => {
  if (!text) return '';
  const lines = text.toString().split('\n').map(line => line.replace(/^[\s　]+/, ''));
  return lines.map(line => {
    const escaped = escapeHtml(line);
    const styled = parseInlineStyles(escaped);
    return `<span class="para-line">${styled || '&nbsp;'}</span>`;
  }).join('');
};

// For DetailModal Logic (Truncation)
const processContent = (contentBlocks) => {
  const MAX_LENGTH = 150;
  let currentLength = 0;
  let blocks = [];
  let truncated = false;
  if (!contentBlocks) return { blocks: [], truncated: false };

  for (let i = 0; i < contentBlocks.length; i++) {
    const block = contentBlocks[i];
    blocks.push(block);
    if (block.text) currentLength += block.text.length;
    if (currentLength > MAX_LENGTH) {
      if (i < contentBlocks.length - 1) {
        truncated = true;
        while (blocks.length > 0 && blocks[blocks.length - 1].type === 'heading') {
          blocks.pop();
        }
      }
      break;
    }
  }
  if (!truncated && blocks.length < contentBlocks.length) {
    truncated = true;
  }
  return { blocks, truncated };
};
const modalState = computed(() => processContent(previewArticle.value?.content));
const modalBlocks = computed(() => modalState.value.blocks);
const isModalTruncated = computed(() => modalState.value.truncated);

// For Page View (Indented Paragraphs + TOC)
const toc = computed(() => {
  if (!previewArticle.value || !previewArticle.value.content) return [];
  return previewArticle.value.content.map((block, index) => ({ ...block, index })).filter((block) => block.type === 'heading');
});
const wordCount = computed(() => {
  if (!previewArticle.value || !previewArticle.value.content) return 0;
  return previewArticle.value.content.reduce((acc, block) => acc + (block.text ? block.text.length : 0), 0);
});
const scrollToHeading = (index) => {
    const el = document.getElementById(`heading-${index}`);
    if (el) el.scrollIntoView({ behavior: 'smooth' });
};
</script>

<style scoped>
/* ==================== Icons ==================== */
.icon-xs {
  width: 0.75rem;
  height: 0.75rem;
}

.icon-sm {
  width: 1rem;
  height: 1rem;
}

.icon-md {
  width: 1.5rem;
  height: 1.5rem;
}

.data-table {
  width: 100%;
  text-align: left;
  border-collapse: collapse;
}

.table-body {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

/* ==================== Article Tab (Tab 2 & 3) ==================== */
.article-tab {
  overflow-x: auto;
  flex: 1;
}

.article-thead {
  background-color: rgba(249,250,251,0.5);
}

.article-thead-row {
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--sos-text-tertiary);
}

.ath-id { padding: 1rem 1rem 1rem 2rem; font-weight: 700; }

.ath-title { padding: 1rem; width: 33.333%; font-weight: 700; }

.ath-author { padding: 1rem; font-weight: 700; }

.ath-date { padding: 1rem; font-weight: 700; }

.ath-status { padding: 1rem; font-weight: 700; }

.ath-actions { padding: 1rem 2rem; text-align: right; font-weight: 700; }

.article-row {
  border-bottom: 1px solid var(--sos-bg-muted);
  transition: color 150ms, background-color 150ms, border-color 150ms;
}

.article-row:hover {
  background-color: rgba(249,250,251,0.8);
}

.article-row:last-child {
  border-bottom: 0;
}

.atd-id {
  padding: 1.25rem 1rem 1.25rem 2rem;
  vertical-align: top;
}

.article-id {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: var(--sos-text-tertiary);
  font-size: 0.75rem;
  line-height: 1rem;
  margin-bottom: 0.25rem;
}

.type-badge-news {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 10px;
  font-weight: 700;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.type-badge-article {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 10px;
  font-weight: 700;
  background-color: var(--sos-border-default);
  color: var(--sos-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.atd-title {
  padding: 1.25rem 1rem;
  vertical-align: top;
}

.article-title-text {
  font-weight: 700;
  font-size: 1rem;
  line-height: 1.5rem;
  color: var(--sos-text-primary);
  margin-bottom: 0.25rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}

.article-row:hover .article-title-text {
  color: #1e3a8a;
}

.article-summary-text {
  color: var(--sos-text-tertiary);
  font-size: 0.75rem;
  line-height: 1rem;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-height: 1.625;
  max-width: 28rem;
}

.atd-author {
  padding: 1.25rem 1rem;
  vertical-align: top;
}

.author-name {
  font-weight: 500;
  color: var(--sos-text-primary);
}

.author-participants {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-tertiary);
  margin-top: 0.125rem;
}

.atd-date {
  padding: 1.25rem 1rem;
  vertical-align: top;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  color: var(--sos-text-secondary);
  font-size: 0.75rem;
  line-height: 1rem;
}

.atd-status {
  padding: 1.25rem 1rem;
  vertical-align: top;
}

.status-pending {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: #a16207;
  background-color: #fefce8;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  border: 1px solid #fef9c3;
}

.status-dot-pending {
  width: 0.375rem;
  height: 0.375rem;
  border-radius: 9999px;
  background-color: #eab308;
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-published-col {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}

.status-published {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: #15803d;
  background-color: #f0fdf4;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  border: 1px solid #dcfce7;
}

.status-dot-published {
  width: 0.375rem;
  height: 0.375rem;
  border-radius: 9999px;
  background-color: #22c55e;
}

.pinned-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.atd-actions-cell {
  padding: 1.25rem 2rem;
  vertical-align: top;
  text-align: right;
}

.action-btns {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  opacity: 1;
  transition: opacity 150ms;
}

@media (min-width: 768px) {
  .action-btns {
    opacity: 0;
  }
  .article-row:hover .action-btns {
    opacity: 1;
  }
}

.action-btn-preview {
  padding: 0.5rem;
  color: var(--sos-text-tertiary);
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
}

.action-btn-preview:hover {
  color: var(--sos-text-primary);
  background-color: var(--sos-bg-muted);
}

.action-divider {
  width: 1px;
  height: 1rem;
  background-color: var(--sos-border-default);
  margin: 0 0.25rem;
}

.action-btn-approve {
  padding: 0.5rem;
  color: var(--sos-success);
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.action-btn-approve:hover {
  background-color: #f0fdf4;
}

.action-btn-edit {
  padding: 0.5rem;
  color: #2563eb;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.action-btn-edit:hover {
  background-color: #eff6ff;
}

.action-btn-delete {
  padding: 0.5rem;
  color: #f87171;
  border-radius: 0.5rem;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1rem;
}

.action-btn-delete:hover {
  color: var(--sos-danger);
  background-color: #fef2f2;
}

/* ==================== Preview Overlay ==================== */
.preview-overlay {
  position: fixed;
  top: 0; right: 0; bottom: 0; left: 0;
  z-index: 60;
  background-color: var(--sos-bg-muted);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-header {
  background-color: var(--sos-bg-surface);
  border-bottom: 1px solid var(--sos-border-default);
  padding: 0.75rem 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  z-index: 10;
  flex-shrink: 0;
}

.preview-header-left {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.preview-header-info {
  display: flex;
  flex-direction: column;
}

.preview-mode-label {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  color: var(--sos-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.preview-title {
  font-weight: 700;
  font-size: 1.125rem;
  line-height: 1.25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 28rem;
}

.preview-switcher {
  display: flex;
  background-color: var(--sos-bg-muted);
  padding: 0.25rem;
  border-radius: 0.5rem;
}

.preview-tab-btn {
  padding: 0.375rem 1rem;
  border-radius: 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  transition: all 150ms;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.preview-tab-active {
  background-color: var(--sos-bg-surface);
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  color: var(--sos-text-primary);
}

.preview-tab-inactive {
  color: var(--sos-text-secondary);
}

.preview-tab-inactive:hover {
  color: var(--sos-text-secondary);
}

.preview-header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.preview-pending-badge {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  background-color: #fef9c3;
  color: #a16207;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}

.preview-close-btn {
  padding: 0.5rem;
  border-radius: 9999px;
  transition: color 150ms, background-color 150ms, border-color 150ms;
  color: var(--sos-text-secondary);
}

.preview-close-btn:hover {
  background-color: var(--sos-bg-muted);
  color: var(--sos-text-primary);
}

.preview-body {
  flex: 1;
  overflow-y: auto;
  background-color: var(--sos-bg-muted);
  position: relative;
}

/* Card Mode */
.preview-card-mode {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.preview-card-wrap {
  width: 380px;
}

.preview-card-component {
  pointer-events: none;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  background-color: var(--sos-bg-surface);
}

.preview-card-caption {
  text-align: center;
  color: var(--sos-text-tertiary);
  font-size: 0.75rem;
  line-height: 1rem;
  margin-top: 1.5rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

/* Modal Mode */
.preview-modal-mode {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

@media (min-width: 768px) {
  .preview-modal-mode {
    padding: 2.5rem;
  }
}

.preview-modal-container {
  position: relative;
  background-color: var(--sos-bg-surface);
  width: 100%;
  max-width: 42rem;
  box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  display: flex;
  flex-direction: column;
  border: 1px solid var(--sos-text-primary);
  max-height: 90vh;
  overflow-y: auto;
}

.preview-modal-header {
  position: sticky;
  top: 0;
  background-color: var(--sos-bg-surface);
  border-bottom: 1px solid var(--sos-bg-muted);
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.preview-modal-header-label {
  font-size: 0.75rem;
  line-height: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--sos-text-tertiary);
}

.preview-modal-close-placeholder {
  padding: 0.5rem;
  opacity: 0.5;
}

.preview-modal-body {
  padding: 1.5rem;
}

@media (min-width: 768px) {
  .preview-modal-body {
    padding: 2.5rem;
  }
}

.preview-modal-meta {
  margin-bottom: 1.5rem;
}

.preview-modal-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.tag-filled {
  display: inline-block;
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  font-size: 0.75rem;
  line-height: 1rem;
  padding: 0.25rem 0.5rem;
}

.tag-outline {
  display: inline-block;
  border: 1px solid var(--sos-text-primary);
  color: var(--sos-text-primary);
  font-size: 0.75rem;
  line-height: 1rem;
  padding: 0.25rem 0.5rem;
  text-transform: uppercase;
}

.preview-modal-article-title {
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
  margin-bottom: 1rem;
  line-height: 1.25;
}

@media (min-width: 768px) {
  .preview-modal-article-title {
    font-size: 2.25rem;
    line-height: 2.5rem;
  }
}

.preview-modal-info-row {
  display: flex;
  align-items: center;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.preview-modal-author {
  color: var(--sos-text-primary);
  font-weight: 700;
}

.preview-modal-participants {
  font-size: 0.75rem;
  line-height: 1rem;
  background-color: var(--sos-bg-subtle);
  padding: 0.75rem;
  margin-bottom: 1rem;
  border-radius: 0.25rem;
}

.participants-label {
  font-weight: 700;
  color: var(--sos-text-tertiary);
  margin-bottom: 0.25rem;
}

.participants-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.participant-name {
  font-weight: 700;
  color: #1e3a8a;
}

.participant-role {
  color: var(--sos-text-secondary);
}

.preview-modal-image {
  margin-bottom: 2rem;
}

.preview-modal-img {
  width: 100%;
  height: auto;
  transition: all 500ms;
}

.preview-modal-content {
  color: var(--sos-text-primary);
  font-family: "Noto Serif SC", serif;
  line-height: 2;
  text-align: justify;
}

.math-block-modal {
  margin: 1rem 0;
  padding: 1rem;
  background-color: var(--sos-bg-subtle);
  text-align: center;
}

.image-block-modal {
  margin: 1rem 0;
}

.image-block-modal-img {
  width: 100%;
  max-height: 12rem;
  object-fit: cover;
}

.truncation-indicator {
  color: var(--sos-text-tertiary);
  text-align: center;
  margin-top: 1rem;
}

.preview-modal-footer-actions {
  margin-top: 2.5rem;
  padding-top: 2rem;
  border-top: 1px solid var(--sos-bg-muted);
  display: flex;
  justify-content: center;
}

.btn-read-full {
  background-color: var(--sos-text-primary);
  color: var(--sos-bg-surface);
  padding: 0.75rem 2rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: not-allowed;
  opacity: 0.8;
}

.btn-open-page {
  border: 1px solid var(--sos-text-primary);
  padding: 0.75rem 2rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: not-allowed;
  opacity: 0.5;
}

/* ==================== Page Mode (BlogDetail replica) ==================== */
.preview-page-mode {
  background-color: var(--sos-bg-surface);
  min-height: 100vh;
  font-family: "Noto Sans SC", sans-serif;
  color: var(--sos-text-primary);
  max-width: 100%;
  overflow-x: hidden;
}

.page-hero {
  position: relative;
  width: 100%;
  height: 600px;
  background-color: var(--sos-text-tertiary);
  overflow: hidden;
  user-select: none;
}

.page-hero-cover {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
}

.page-hero-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.page-hero-gradient-main {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to bottom, rgba(0,0,0,0.3), transparent, rgba(0,0,0,0.6));
}

.page-hero-overlay {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background-color: rgba(0,0,0,0.1);
}

.page-hero-fallback {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  width: 100%;
  height: 100%;
  background-color: var(--sos-text-tertiary);
  overflow: hidden;
}

.page-hero-pattern {
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

.page-hero-pattern-text {
  font-size: 4.5rem;
  line-height: 1;
  font-weight: 900;
  color: var(--sos-text-primary);
  white-space: nowrap;
  font-family: "Noto Sans SC", sans-serif;
}

.page-hero-gradient-fallback {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.063), transparent);
}

.page-hero-content {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  top: 0;
  width: 100%;
  max-width: 1600px;
  height: 100%;
  z-index: 10;
}

.page-tags-desktop {
  display: none;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}

@media (min-width: 768px) {
  .page-tags-desktop {
    display: flex;
  }
}

.page-tags-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.page-tag-item {
  display: flex;
  align-items: center;
}

.page-tag-hash {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: rgba(255,255,255,0.5);
  margin-right: 0.25rem;
}

.page-tag-text {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 36px;
  color: var(--sos-bg-surface);
  text-decoration: underline;
  text-decoration-thickness: 2px;
  text-underline-offset: 4px;
}

.page-news-badge {
  margin-left: 0.5rem;
  border: 1px solid rgba(255,255,255,0.6);
  padding: 0.125rem 0.5rem;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.1em;
}

.page-title-desktop {
  display: none;
  flex-direction: column;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
  position: absolute;
}

@media (min-width: 768px) {
  .page-title-desktop {
    display: flex;
  }
}

.page-main-title {
  font-family: "Noto Serif SC", serif;
  font-weight: 900;
  font-size: 80px;
  line-height: 90px;
  letter-spacing: 0;
  margin-bottom: 0;
  color: var(--sos-bg-surface);
}

.page-subtitle {
  margin-top: 6px;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 27px;
  line-height: 45px;
  color: var(--sos-bg-surface);
}

.page-author-row {
  margin-top: 1rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  column-gap: 1rem;
  row-gap: 0.25rem;
}

.page-author-info {
  font-family: "Noto Sans SC", sans-serif;
  font-size: 24px;
  line-height: 45px;
  color: var(--sos-bg-surface);
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.page-author-label {
  font-weight: 400;
  opacity: 0.8;
}

.page-author-name {
  font-weight: 700;
}

.page-date-info {
  font-family: "Noto Sans SC", sans-serif;
  font-size: 21px;
  line-height: 45px;
  color: rgba(255,255,255,0.9);
}

/* Mobile Header */
.page-mobile-header {
  position: absolute;
  left: 1rem;
  right: 1rem;
  bottom: 2.5rem;
  color: var(--sos-bg-surface);
  filter: drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));
}

@media (min-width: 768px) {
  .page-mobile-header {
    display: none;
  }
}

.page-mobile-tags {
  margin-bottom: 0.5rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.page-mobile-tag {
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-bg-surface);
}

.page-mobile-title {
  font-family: "Noto Serif SC", serif;
  font-weight: 900;
  font-size: 1.5rem;
  line-height: 1.375;
}

.page-mobile-subtitle {
  margin-top: 0.25rem;
  font-family: "Noto Sans SC", sans-serif;
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

/* Page Content Body */
.page-content-container {
  max-width: 1600px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
  position: relative;
  margin-top: 61px;
}

@media (min-width: 768px) {
  .page-content-container {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

@media (min-width: 1024px) {
  .page-content-container {
    padding-left: 0;
    padding-right: 0;
  }
}

.page-content-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 3rem;
  padding-bottom: 5rem;
}

@media (min-width: 1024px) {
  .page-content-grid {
    grid-template-columns: repeat(12, minmax(0, 1fr));
    padding-left: 105px;
    padding-right: 2.5rem;
  }
}

.page-article {
  padding-top: 0;
  padding-bottom: 3rem;
  max-width: none;
}

@media (min-width: 768px) {
  .page-article {
    padding-top: 0;
    padding-bottom: 5rem;
  }
}

@media (min-width: 1024px) {
  .page-article {
    grid-column: span 9;
  }
}

/* Prose overrides for page article */
.page-article :deep(h1),
.page-article :deep(h2),
.page-article :deep(h3),
.page-article :deep(h4),
.page-article :deep(h5),
.page-article :deep(h6) {
  font-family: "Noto Serif SC", serif;
}

.page-article :deep(p) {
  color: var(--sos-text-secondary);
}

.page-article :deep(img) {
  border-radius: 0.125rem;
}

/* Page Participants */
.page-participants-box {
  margin-bottom: 2.5rem;
  border-radius: 0.5rem;
  border: 1px solid var(--sos-bg-muted);
  background-color: rgba(249,250,251,0.8);
  padding: 1.25rem 1.5rem;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}

.page-participants-heading {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  letter-spacing: 0.25em;
  color: var(--sos-text-secondary);
  text-transform: uppercase;
  margin-bottom: 0.75rem;
}

.page-participants-list {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
}

.page-participants-list > * + * {
  margin-top: 0.5rem;
}

.page-participant-item {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  column-gap: 0.5rem;
  row-gap: 0.125rem;
}

.page-participant-detail {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-secondary);
}

/* Page Content Blocks */
.page-content-block {
  margin-bottom: 2rem;
}

.page-paragraph {
  line-height: 1.625;
  text-align: justify;
  font-size: 22px;
  letter-spacing: 0.025em;
  color: var(--sos-text-primary);
}

@media (min-width: 768px) {
  .page-paragraph {
    font-size: 23px;
  }
}

.page-heading {
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  margin-top: 4rem;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e5e5;
  color: var(--sos-text-primary);
  position: relative;
}

.page-math-block {
  margin: 2.5rem 0;
  padding: 2rem;
  background-color: #f9f9f9;
  border-left: 4px solid var(--sos-text-tertiary);
  text-align: center;
  overflow-x: auto;
}

.page-math-expr {
  font-family: "Noto Serif SC", serif;
  font-size: 1.25rem;
  line-height: 1.75rem;
}

.page-math-caption {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  margin-top: 1rem;
  font-style: normal;
  font-family: "Noto Sans SC", sans-serif;
}

.page-image-figure {
  margin: 3rem 0;
}

.page-image-wrap {
  position: relative;
  overflow: hidden;
}

.page-image {
  width: 100%;
  height: auto;
  display: block;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  border: 1px solid var(--sos-bg-muted);
}

.page-image-caption {
  text-align: center;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-secondary);
  margin-top: 0.75rem;
  font-family: "Noto Sans SC", sans-serif;
  letter-spacing: 0.025em;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.25rem;
}

.caption-dot {
  width: 0.25rem;
  height: 0.25rem;
  background-color: var(--sos-text-tertiary);
  border-radius: 9999px;
}

/* Fin marker */
.page-fin {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin: 5rem 0;
  opacity: 0.3;
}

.fin-line {
  height: 1px;
  width: 3rem;
  background-color: var(--sos-text-primary);
}

.fin-text {
  font-family: "Noto Serif SC", serif;
  font-style: italic;
  font-size: 1.125rem;
  line-height: 1.75rem;
}

/* Sidebar */
.page-sidebar {
  display: none;
  padding-left: 2rem;
  border-left: 1px solid var(--sos-bg-muted);
  background-color: rgba(255,255,255,0.5);
}

@media (min-width: 1024px) {
  .page-sidebar {
    display: block;
    grid-column: span 3;
  }
}

.page-sidebar-inner {
  position: sticky;
  top: 8rem;
  padding-top: 3rem;
  padding-bottom: 3rem;
}

.page-sidebar-title {
  font-weight: 700;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  color: var(--sos-text-tertiary);
  margin-bottom: 2rem;
}

.page-toc-list {
  border-left: 2px solid var(--sos-bg-muted);
  margin-left: 0.25rem;
  padding-left: 1.25rem;
  position: relative;
}

.page-toc-list > * + * {
  margin-top: 1.25rem;
}

.page-toc-item {
  position: relative;
}

.page-toc-dot {
  position: absolute;
  left: -23px;
  top: 6px;
  width: 6px;
  height: 6px;
  border-radius: 9999px;
  background-color: var(--sos-border-strong);
  transition: color 150ms, background-color 150ms, border-color 150ms;
}

.page-toc-item:hover .page-toc-dot {
  background-color: var(--sos-text-primary);
}

.page-toc-link {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-text-secondary);
  transition: all 150ms;
  display: block;
  line-height: 1.625;
  font-family: "Noto Sans SC", sans-serif;
}

.page-toc-link:hover {
  color: var(--sos-text-primary);
  font-weight: 700;
}

.page-toc-empty {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--sos-border-strong);
  font-style: italic;
  padding-left: 1.5rem;
}

.page-sidebar-stats {
  margin-top: 4rem;
  padding-top: 2rem;
  border-top: 1px solid var(--sos-bg-muted);
}

.page-sidebar-stats-inner {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--sos-text-tertiary);
  font-family: "Noto Sans SC", sans-serif;
}

.page-sidebar-stats-inner > * + * {
  margin-top: 0.5rem;
}

.page-stat-val {
  color: var(--sos-text-primary);
}

/* ==================== Existing Preserved Styles ==================== */

/* Scoped styles for typography in preview */
:deep(b) { font-weight: bold; }

:deep(i) { font-style: italic; }

:deep(u) { text-decoration: underline; }

:deep(s) { text-decoration: line-through; }

:deep(.inline-link) {
  color: #2563eb;
  word-break: break-all;
}

:deep(.inline-link:hover) {
  text-decoration: underline;
}

/* Keep the old preview-overlay entrance timing (200ms) */
.animate-fade-in-fast {
  animation: anim-fade-in 0.2s ease both;
}

/* Text Shadow for Page Header */
.text-shadow {
  text-shadow: 0px 6px 22.5px rgba(0, 0, 0, 0.3);
}

/* Article Paragraph Indentation for Page View */
.article-paragraph {
  margin: 0;
}

.article-paragraph :deep(.para-line) {
  display: block;
  text-indent: 2em;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0,0,0,0.05);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.2);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(0,0,0,0.4);
}

/* Fade transition for Vue <Transition name="fade"> */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
