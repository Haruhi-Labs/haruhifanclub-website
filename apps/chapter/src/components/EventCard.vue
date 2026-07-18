<script setup>
import { RouterLink } from 'vue-router'
import { resolveUploadUrl } from '@haruhi/api-client'

defineProps({
  item: { type: Object, required: true },
  showBranch: { type: Boolean, default: false },
})

function formatLabel(value) {
  return value === 'online' ? '线上' : value === 'hybrid' ? '线上线下混合' : '线下'
}
</script>

<template>
  <RouterLink
    :to="`/branches/${item.branchSlug}/events/${item.slug}`"
    class="chapter-card feed-card event-card"
  >
    <img
      v-if="item.coverPath"
      class="event-card__cover"
      :src="resolveUploadUrl(item.coverPath)"
      :alt="item.title"
    />
    <div class="event-card__body">
      <p v-if="showBranch" class="sos-eyebrow">{{ item.branchName }}</p>
      <time :datetime="item.startsAt">{{ item.startsAt?.replace('T', ' ').slice(0, 16) }}</time>
      <h2>{{ item.title }}</h2>
      <p>{{ item.summary }}</p>
      <footer>
        <span>{{ item.venueName || (item.format === 'online' ? '线上活动' : '地点待定') }}</span>
        <span>{{ item.eventType }} · {{ formatLabel(item.format) }}</span>
      </footer>
      <strong class="event-card__action">查看活动 →</strong>
    </div>
  </RouterLink>
</template>
