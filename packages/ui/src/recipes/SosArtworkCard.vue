<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    title: string
    author?: string
    category?: string
    image?: string
    imageAlt?: string
    ratio?: '4:3' | '1:1' | '3:4' | '16:9'
    tags?: string[]
    likes?: number
  }>(),
  {
    author: undefined,
    category: undefined,
    image: undefined,
    imageAlt: '',
    ratio: '4:3',
    tags: () => [],
    likes: undefined,
  }
)

const emit = defineEmits<{
  click: []
}>()

const ratioValue = computed(() => props.ratio.replace(':', ' / '))
</script>

<template>
  <article
    class="sos-card sos-artwork-card sos-card--interactive"
    role="button"
    tabindex="0"
    @click="emit('click')"
    @keydown.enter.self="emit('click')"
  >
    <div class="sos-artwork-card__frame" :style="{ '--sos-artwork-ratio': ratioValue }">
      <img v-if="image" :src="image" :alt="imageAlt || title" />
    </div>
    <div class="sos-artwork-card__bar">
      <div class="sos-artwork-card__bar-main">
        <span v-if="category" class="sos-artwork-card__cat">{{ category }}</span>
        <h3 class="sos-artwork-card__title">{{ title }}</h3>
        <p v-if="author" class="sos-artwork-card__author">{{ author }}</p>
        <div v-if="tags.length" class="sos-artwork-card__tags">
          <span v-for="tag in tags" :key="tag" class="sos-artwork-card__tag">#{{ tag }}</span>
        </div>
      </div>
      <span v-if="likes !== undefined" class="sos-artwork-card__like">♥ {{ likes }}</span>
    </div>
  </article>
</template>
