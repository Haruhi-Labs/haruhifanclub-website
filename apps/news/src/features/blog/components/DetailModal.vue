<template>
  <Transition name="fade">
    <div
      v-if="article"
      class="modal-overlay"
      @click.self="store.closeModal"
    >
      <div class="modal-backdrop"></div>

      <Transition name="slide-up" appear>
        <div class="modal-container">
          <!-- Header -->
          <div class="modal-header">
            <div class="header-label">
              Preview Mode
            </div>
            <button
              @click="store.closeModal"
              class="header-close-btn"
            >
              <svg
                class="close-icon"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                ></path>
              </svg>
            </button>
          </div>

          <div class="modal-body">
            <!-- Content Header -->
            <div class="content-header">
              <div class="tags-row">
                <span
                  v-for="tag in (article.tags || [])"
                  :key="tag"
                  @click="goToTag(tag)"
                  class="tag-filled"
                >
                  {{ tag }}
                </span>
                <span
                  v-if="article.type === 'news'"
                  class="tag-outlined"
                >
                  News
                </span>
              </div>
              <h2 class="modal-title">
                {{ article.title }}
              </h2>
              <div class="article-meta">
                <span>{{ article.date }}</span>

                <span
                  v-if="article.type !== 'news'"
                  class="author-link"
                  @click="goToAuthor(article.author || '凉宫春日应援团')"
                  >By {{ article.author || '凉宫春日应援团' }}</span
                >
              </div>

              <!-- Participants (News) -->
              <div
                v-if="
                  article.type === 'news' &&
                  article.participants &&
                  article.participants.length
                "
                class="participants-section"
              >
                <p class="participants-label">PARTICIPANTS:</p>
                <div class="participants-list">
                  <div
                    v-for="(p, idx) in article.participants"
                    :key="idx"
                  >
                    <span
                      class="participant-name"
                      @click="goToParticipant(p.name)"
                      >{{ p.name }}</span
                    >
                    <span class="participant-detail">
                      — {{ p.role }}
                      <span v-if="p.project"> ({{ p.project }})</span>
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="article.image" class="article-image-wrapper">
              <img
                :src="article.image"
                class="article-image"
              />
            </div>

            <!-- Renderer -->
            <div
              class="content-renderer prose prose-lg"
            >
              <div v-for="(block, index) in modalBlocks" :key="index">
                <p
                  v-if="block.type === 'paragraph'"
                  v-html="formatParagraph(block.text)"
                ></p>

                <h3 v-else-if="block.type === 'heading'">
                  {{ block.text }}
                </h3>
                <div
                  v-else-if="block.type === 'math'"
                  class="math-block"
                >
                  $$ {{ block.expression }} $$
                </div>
                <div
                  v-else-if="block.type === 'image'"
                  class="image-block"
                >
                  <img
                    :src="block.src"
                    class="block-image"
                  />
                </div>
              </div>
              <p
                v-if="isTruncated"
                class="truncation-indicator"
              >
                ......
              </p>
            </div>

            <div class="footer-actions">
              <button
                v-if="isTruncated"
                @click="goToBlog"
                class="btn-primary"
              >
                <span>完整阅读</span>
                <svg class="btn-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path></svg>
              </button>
              <button
                v-else
                @click="goToBlog"
                class="btn-secondary"
              >
                <span>在新页面打开</span>
                <svg class="btn-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path></svg>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup>
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useMainStore } from '@/stores/main';

const store = useMainStore();
const router = useRouter();

const article = computed(() => store.selectedArticle);

const escapeHtml = (str) => {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
};

// 解析 Markdown 样式
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
  const escaped = escapeHtml(text.toString());
  const styled = parseInlineStyles(escaped);
  return styled.replace(/\n/g, '<br>');
};

/**
 * contentBlocks: 文章内容（数组）
 * isPreviewOnly: 是否为「仅预览内容」（列表接口返回）
 *  - 若是预览内容，不再根据长度截断，直接视为 truncated=true
 */
const processContent = (contentBlocks, isPreviewOnly = false) => {
  if (!contentBlocks || !Array.isArray(contentBlocks)) {
    return { blocks: [], truncated: false };
  }

  if (isPreviewOnly) {
    // 预览模式：后端已经做过截断，这里直接全部展示，同时视为"未完全展开"
    return {
      blocks: contentBlocks,
      truncated: true,
    };
  }

  // 原本的「根据字数截断」逻辑，保留用于完整内容
  const MAX_LENGTH = 150;
  let currentLength = 0;
  let blocks = [];
  let truncated = false;

  for (let i = 0; i < contentBlocks.length; i++) {
    const block = contentBlocks[i];
    blocks.push(block);
    if (block.text) currentLength += block.text.length;
    if (currentLength > MAX_LENGTH) {
      if (i < contentBlocks.length - 1) {
        truncated = true;
        while (
          blocks.length > 0 &&
          blocks[blocks.length - 1].type === 'heading'
        ) {
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

const modalState = computed(() => {
  const art = article.value;
  if (!art) return { blocks: [], truncated: false };

  const isPreviewOnly = art.isContentPreview === true;
  return processContent(art.content, isPreviewOnly);
});

const modalBlocks = computed(() => modalState.value.blocks);
const isTruncated = computed(() => modalState.value.truncated);

const goToBlog = () => {
  if (!article.value) return;
  router.push(`/blog/${article.value.id}`);
  store.closeModal();
};

const goToTag = (tag) => {
  router.push(`/tag/${tag}`);
  store.closeModal();
};

const goToParticipant = (name) => {
  router.push(`/participant/${name}`);
  store.closeModal();
};

const goToAuthor = (author) => {
  router.push(`/author/${author}`);
  store.closeModal();
};
</script>

<style scoped>
/* Modal Overlay */
.modal-overlay {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
}

/* Backdrop */
.modal-backdrop {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-color: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
}

/* Modal Container */
.modal-container {
    position: relative;
    background-color: #fff;
    width: 100%;
    max-width: 42rem;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 25px 50px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    border: 1px solid #000000;
}

/* Header */
.modal-header {
    position: sticky;
    top: 0;
    background-color: #fff;
    border-bottom: 1px solid #f3f4f6;
    padding: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 10;
}

.header-label {
    font-size: 0.75rem;
    line-height: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #9ca3af;
}

.header-close-btn {
    padding: 0.5rem;
    border-radius: 9999px;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
}

.header-close-btn:hover {
    background-color: #f3f4f6;
}

.close-icon {
    width: 1.5rem;
    height: 1.5rem;
}

/* Modal Body */
.modal-body {
    padding: 1.5rem;
}

@media (min-width: 768px) {
    .modal-body {
        padding: 2.5rem;
    }
}

/* Content Header */
.content-header {
    margin-bottom: 1.5rem;
}

/* Tags Row */
.tags-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
}

.tag-filled {
    display: inline-block;
    background-color: #000000;
    color: #ffffff;
    font-size: 0.75rem;
    line-height: 1rem;
    padding-left: 0.5rem;
    padding-right: 0.5rem;
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
    cursor: pointer;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
}

.tag-filled:hover {
    background-color: #1f2937;
}

.tag-outlined {
    display: inline-block;
    border: 1px solid #000000;
    color: #000000;
    font-size: 0.75rem;
    line-height: 1rem;
    padding-left: 0.5rem;
    padding-right: 0.5rem;
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
    text-transform: uppercase;
}

/* Modal Title */
.modal-title {
    font-size: 1.875rem;
    line-height: 2.25rem;
    font-weight: 700;
    margin-bottom: 1rem;
    line-height: 1.25;
}

@media (min-width: 768px) {
    .modal-title {
        font-size: 2.25rem;
        line-height: 1.25;
    }
}

/* Article Meta */
.article-meta {
    display: flex;
    align-items: center;
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #6b7280;
    gap: 1rem;
    margin-bottom: 0.5rem;
}

.author-link {
    color: #000000;
    font-weight: 700;
    cursor: pointer;
}

.author-link:hover {
    text-decoration: underline;
}

/* Participants Section */
.participants-section {
    font-size: 0.75rem;
    line-height: 1rem;
    background-color: #f9fafb;
    padding: 0.75rem;
    margin-bottom: 1rem;
    border-radius: 0.25rem;
}

.participants-label {
    font-weight: 700;
    color: #9ca3af;
    margin-bottom: 0.25rem;
}

.participants-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.participant-name {
    font-weight: 700;
    cursor: pointer;
    color: #1e3a8a;
}

.participant-name:hover {
    text-decoration: underline;
}

.participant-detail {
    color: #6b7280;
}

/* Article Image */
.article-image-wrapper {
    margin-bottom: 2rem;
}

.article-image {
    width: 100%;
    height: auto;
    transition: all 500ms;
}

/* Content Renderer */
.content-renderer {
    color: #1f2937;
    font-family: "Noto Serif SC", serif;
    line-height: 2;
    text-align: justify;
}

/* Math Block */
.math-block {
    margin-top: 1rem;
    margin-bottom: 1rem;
    padding: 1rem;
    background-color: #f9fafb;
    text-align: center;
}

/* Image Block */
.image-block {
    margin-top: 1rem;
    margin-bottom: 1rem;
}

.block-image {
    width: 100%;
    max-height: 12rem;
    object-fit: cover;
}

/* Truncation Indicator */
.truncation-indicator {
    color: #9ca3af;
    text-align: center;
    margin-top: 1rem;
}

/* Footer Actions */
.footer-actions {
    margin-top: 2.5rem;
    padding-top: 2rem;
    border-top: 1px solid #f3f4f6;
    display: flex;
    justify-content: center;
}

/* Primary Button (filled) */
.btn-primary {
    background-color: #000000;
    color: #ffffff;
    padding-left: 2rem;
    padding-right: 2rem;
    padding-top: 0.75rem;
    padding-bottom: 0.75rem;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.btn-primary:hover {
    background-color: #1f2937;
}

/* Secondary Button (outlined) */
.btn-secondary {
    border: 1px solid #000000;
    padding-left: 2rem;
    padding-right: 2rem;
    padding-top: 0.75rem;
    padding-bottom: 0.75rem;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.btn-secondary:hover {
    background-color: #f9fafb;
}

.btn-icon {
    width: 1rem;
    height: 1rem;
}

/* Inline link (generated by Markdown parser) */
:deep(.inline-link) {
    color: #2563eb;
}

:deep(.inline-link:hover) {
    text-decoration: underline;
}

.content-renderer :deep(b) { font-weight: bold; }
.content-renderer :deep(i) { font-style: italic; }
.content-renderer :deep(u) { text-decoration: underline; }
.content-renderer :deep(s) { text-decoration: line-through; }
</style>
