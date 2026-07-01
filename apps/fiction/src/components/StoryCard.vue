<script setup>
import { computed } from 'vue'
import CoverImage from '@/components/CoverImage.vue'
import { compact, wordLabel, categoryLabel } from '@/lib/format'

const props = defineProps({
  story: { type: Object, required: true },
  // compact: 横向紧凑排版（排行榜/侧栏）；默认纵向卡片
  layout: { type: String, default: 'grid' },
})

const s = computed(() => props.story)
const to = computed(() => `/story/${s.value.id}`)
</script>

<template>
  <RouterLink :to="to" class="fiction-card" :class="`fiction-card--${layout}`">
    <div class="fiction-card__cover">
      <CoverImage :path="s.coverPath" :title="s.title" :category="s.category" />
      <span v-if="s.isCompleted" class="fiction-card__flag fiction-card__flag--done">完结</span>
      <span v-else class="fiction-card__flag">连载</span>
    </div>
    <div class="fiction-card__body">
      <div class="fiction-card__meta">
        <span class="fiction-card__cat">{{ categoryLabel(s.category) }}</span>
        <span v-if="s.contentRating === 'mature'" class="fiction-card__r18">限</span>
      </div>
      <h3 class="fiction-card__title">{{ s.title }}</h3>
      <p class="fiction-card__author">{{ s.authorName || '佚名' }}</p>
      <p v-if="layout === 'grid'" class="fiction-card__summary">{{ s.summary || '暂无简介' }}</p>
      <ul v-if="layout === 'grid' && s.tags && s.tags.length" class="fiction-card__tags">
        <li v-for="t in s.tags.slice(0, 3)" :key="t">{{ t }}</li>
      </ul>
      <div class="fiction-card__stats">
        <span>{{ wordLabel(s.wordCount) }}</span>
        <span>·</span>
        <span>{{ compact(s.viewCount) }} 阅</span>
        <span>·</span>
        <span>{{ compact(s.likeCount) }} 赞</span>
      </div>
    </div>
  </RouterLink>
</template>
