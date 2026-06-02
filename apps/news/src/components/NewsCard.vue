<template>
    <div class="news-card" @click="$emit('click')">

        <!-- News Label -->
        <div v-if="article.type === 'news'" class="news-label">NEWS</div>

        <!-- Image -->
        <div v-if="article.image" class="image-container">
            <img :src="article.image" :alt="article.title" class="card-image">
        </div>

        <!-- Title -->
        <h2 class="card-title" v-html="highlight(article.title)"></h2>

        <!-- Subtitle -->
        <div v-if="article.subtitle" class="card-subtitle" v-html="highlight(article.subtitle)"></div>

        <!-- Participants (News only) -->
        <div v-if="article.type === 'news' && article.participants && article.participants.length" class="participants-box">
            <div v-for="(p, idx) in article.participants" :key="idx" class="participant-row">
                <span class="participant-name" @click.stop="$router.push(`/participant/${p.name}`)">{{ p.name }}</span>
                <span class="participant-detail"> — {{ p.role }} ({{ p.project }})</span>
            </div>
        </div>

        <!-- Summary -->
        <div
            class="card-summary"
            v-html="highlight(previewText)"
        ></div>

        <!-- Footer: Tags (Left) + Meta (Right) -->
        <div class="card-footer">

            <!-- Tags -->
            <div class="tags-container">
                <span
                    v-for="tag in (article.tags || []).slice(0, 3)"
                    :key="tag"
                    @click.stop="$router.push(`/tag/${tag}`)"
                    class="tag-item"
                >
                    #{{ tag }}
                </span>
            </div>

            <!-- Author & Date -->
            <div class="meta-info">
                <span v-if="article.type !== 'news'" class="author-section">
                    <span>
                        作者：<span
                            class="author-name"
                            @click.stop="$router.push(`/author/${article.author || '凉宫春日应援团'}`)"
                            v-html="highlight(article.author || '凉宫春日应援团')"
                        ></span>
                    </span>
                    <span class="meta-separator">·</span>
                </span>
                <span class="date-text">{{ article.date }}</span>
            </div>

        </div>
    </div>
</template>

<script setup>
import { computed } from 'vue';
import { useMainStore } from '@/stores/main';

const props = defineProps(['article']);
const store = useMainStore();

// 直接使用后端生成的 preview 文本
const previewText = computed(() => {
    if (!props.article) return '';
    return props.article.preview || props.article.summary || '';
});

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
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="inline-link" onclick="event.stopPropagation()">$1</a>');
};

const highlight = (text) => {
    if (!text) return '';

    let escaped = escapeHtml(text.toString());
    escaped = parseInlineStyles(escaped);
    escaped = escaped.replace(/\n/g, '<br>');

    if (!store.searchQuery || store.searchQuery.trim() === '') {
        return escaped;
    }

    const query = store.searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${query})`, 'gi');
    return escaped.replace(regex, '<span class="highlight-text">$1</span>');
};
</script>

<style scoped>
/* Card Container */
.news-card {
    padding: 1.25rem;
    cursor: pointer;
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: #fff;
    transition-property: box-shadow;
    transition-duration: 300ms;
}

.news-card:hover {
    box-shadow: 0 10px 15px rgba(0, 0, 0, 0.1), 0 4px 6px rgba(0, 0, 0, 0.05);
}

/* News Label */
.news-label {
    font-size: 0.75rem;
    line-height: 1rem;
    font-weight: 700;
    margin-bottom: 0.5rem;
    color: #000000;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    border: 1px solid #000000;
    display: inline-block;
    padding-left: 0.25rem;
    padding-right: 0.25rem;
    width: fit-content;
}

/* Image Container */
.image-container {
    margin-bottom: 1rem;
    overflow: hidden;
    border: 1px solid #f3f4f6;
    flex-shrink: 0;
}

.card-image {
    width: 100%;
    height: 12rem;
    object-fit: cover;
    transition-property: transform;
    transition-duration: 700ms;
}

.news-card:hover .card-image {
    transform: scale(1.05);
}

/* Title */
.card-title {
    font-size: 1.25rem;
    line-height: 1.75rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
    line-height: 1.25;
    transition-property: color, background-color, border-color;
    transition-duration: 150ms;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    overflow: hidden;
}

@media (min-width: 768px) {
    .card-title {
        font-size: 1.5rem;
        line-height: 1.25;
    }
}

.news-card:hover .card-title {
    color: #1e3a8a;
}

/* Subtitle */
.card-subtitle {
    font-size: 0.875rem;
    line-height: 1.25rem;
    font-weight: 700;
    color: #6b7280;
    margin-bottom: 1rem;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    overflow: hidden;
}

/* Participants Box */
.participants-box {
    margin-bottom: 1rem;
    font-size: 0.75rem;
    line-height: 1rem;
    background-color: #f9fafb;
    padding: 0.5rem;
    border-left: 2px solid #e5e7eb;
    flex-shrink: 0;
}

.participant-row {
    margin-bottom: 0.25rem;
}

.participant-row:last-child {
    margin-bottom: 0;
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

/* Summary */
.card-summary {
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: #374151;
    text-align: justify;
    margin-bottom: 1rem;
    overflow: hidden;
    line-height: 1.7;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 6;
}

/* Footer */
.card-footer {
    margin-top: auto;
    padding-top: 0.75rem;
    border-top: 1px solid #f3f4f6;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    row-gap: 0.5rem;
    column-gap: 1rem;
    flex-shrink: 0;
}

/* Tags */
.tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.tag-item {
    font-size: 10px;
    font-weight: 700;
    border: 1px solid #d1d5db;
    padding-left: 0.375rem;
    padding-right: 0.375rem;
    padding-top: 0.125rem;
    padding-bottom: 0.125rem;
    color: #6b7280;
    transition: all 200ms;
}

.tag-item:hover {
    border-color: #000000;
    background-color: #000000;
    color: #ffffff;
}

/* Meta Info */
.meta-info {
    display: flex;
    align-items: center;
    font-size: 0.75rem;
    line-height: 1rem;
    color: #9ca3af;
    line-height: 1;
    white-space: nowrap;
    margin-left: auto;
}

.author-section {
    display: flex;
    align-items: center;
}

.author-name {
    font-weight: 700;
    color: #4b5563;
    cursor: pointer;
}

.author-name:hover {
    color: #000000;
    text-decoration: underline;
}

.meta-separator {
    color: #d1d5db;
    font-weight: 700;
    margin-left: 0.375rem;
    margin-right: 0.375rem;
}

.date-text {
    letter-spacing: -0.025em;
}

/* Highlight */
.highlight-text {
    background-color: #fef08a;
    color: black;
    padding: 0 2px;
    border-radius: 2px;
}

/* Inline link (generated by Markdown parser) */
:deep(.inline-link) {
    color: #2563eb;
}

:deep(.inline-link:hover) {
    text-decoration: underline;
}

:deep(b) { font-weight: bold; }
:deep(i) { font-style: italic; }
:deep(u) { text-decoration: underline; }
:deep(s) { text-decoration: line-through; }
</style>
