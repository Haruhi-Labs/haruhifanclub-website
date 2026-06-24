<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string
    author?: string
    cover?: string
    color?: string
    badge?: string
  }>(),
  {
    author: undefined,
    cover: undefined,
    color: undefined,
    badge: undefined,
  }
)

const emit = defineEmits<{
  click: []
}>()
</script>

<template>
  <article
    class="sos-card sos-book-card"
    role="button"
    tabindex="0"
    @click="emit('click')"
    @keydown.enter.self="emit('click')"
    @keydown.space.prevent.self="emit('click')"
  >
    <div class="sos-book-card__cover" :style="color ? { '--book-color': color } : undefined">
      <span v-if="badge" class="sos-book-card__badge sos-badge sos-badge--solid">{{ badge }}</span>
      <img v-if="cover" :src="cover" :alt="title" />
      <span v-else class="sos-book-card__vertical">{{ title }}</span>
    </div>
    <div>
      <h3 class="sos-book-card__title">{{ title }}</h3>
      <p v-if="author" class="sos-book-card__author">{{ author }}</p>
    </div>
  </article>
</template>
