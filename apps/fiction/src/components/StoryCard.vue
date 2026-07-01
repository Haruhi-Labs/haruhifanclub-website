<script setup>
import { computed } from 'vue'
import { compact, wordLabel, categoryLabel } from '@/lib/format'

const props = defineProps({
  story: { type: Object, required: true },
  // compact: 排行/侧栏用的精简文本行；默认 list 为完整文本卡
  layout: { type: String, default: 'list' },
})

const s = computed(() => props.story)
const to = computed(() => `/story/${s.value.id}`)
const isCompact = computed(() => props.layout === 'compact')
</script>

<template>
  <RouterLink :to="to" class="fic-item" :class="`fic-item--${layout}`">
    <div class="fic-item__top">
      <h3 class="fic-item__title">{{ s.title }}</h3>
      <span class="fic-item__flag" :class="{ 'is-done': s.isCompleted }">
        {{ s.isCompleted ? '完结' : '连载' }}
      </span>
    </div>

    <div class="fic-item__by">
      <span class="fic-item__cat">{{ categoryLabel(s.category) }}</span>
      <span class="fic-item__dot">·</span>
      <span>{{ s.authorName || '佚名' }}</span>
      <span v-if="s.contentRating === 'mature'" class="fic-item__r18">限</span>
    </div>

    <p v-if="!isCompact" class="fic-item__summary">{{ s.summary || '暂无简介' }}</p>

    <div class="fic-item__foot">
      <ul v-if="!isCompact && s.tags && s.tags.length" class="fic-item__tags">
        <li v-for="t in s.tags.slice(0, 3)" :key="t">{{ t }}</li>
      </ul>
      <span class="fic-item__stats">
        {{ wordLabel(s.wordCount) }} · {{ s.chapterCount }} 章 · {{ compact(s.viewCount) }} 阅 · {{ compact(s.likeCount) }} 赞
      </span>
    </div>
  </RouterLink>
</template>
