<script setup>
import { ref } from 'vue'
import { resolveUploadUrl } from '@haruhi/api-client'
import { SosModal } from '@haruhi/ui'

defineProps({
  items: { type: Array, default: () => [] },
})

const viewerOpen = ref(false)
const selected = ref(null)

function openPhoto(item) {
  selected.value = item
  viewerOpen.value = true
}

function formatDate(value) {
  return value?.replace('T', ' ').slice(0, 16) || ''
}
</script>

<template>
  <div class="event-album" role="list">
    <button
      v-for="item in items"
      :key="item.id"
      type="button"
      class="event-album__item"
      role="listitem"
      :aria-label="`查看照片：${item.title}`"
      @click="openPhoto(item)"
    >
      <img
        :src="resolveUploadUrl(item.imagePath)"
        :alt="item.title"
        loading="lazy"
        decoding="async"
      />
      <span class="event-album__caption">
        <strong>{{ item.title }}</strong>
        <time :datetime="item.happenedAt">{{ formatDate(item.happenedAt) }}</time>
      </span>
    </button>
  </div>

  <SosModal v-model:open="viewerOpen" :title="selected?.title" wide>
    <figure v-if="selected" class="event-album-viewer">
      <img :src="resolveUploadUrl(selected.imagePath)" :alt="selected.title" />
      <figcaption>
        <time :datetime="selected.happenedAt">{{ formatDate(selected.happenedAt) }}</time>
        <p v-if="selected.content">{{ selected.content }}</p>
      </figcaption>
    </figure>
  </SosModal>
</template>
