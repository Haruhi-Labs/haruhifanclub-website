<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    title: string
    author?: string
    image?: string
    imageAlt?: string
    ratio?: '4:3' | '1:1' | '3:4' | '16:9'
    likes?: number
    views?: number
  }>(),
  {
    author: undefined,
    image: undefined,
    imageAlt: '',
    ratio: '4:3',
    likes: undefined,
    views: undefined,
  }
)

const emit = defineEmits<{
  click: []
}>()

const ratioValue = computed(() => props.ratio.replace(':', ' / '))
</script>

<template>
  <article
    class="sos-card sos-artwork-card"
    role="button"
    tabindex="0"
    @click="emit('click')"
    @keydown.enter.self="emit('click')"
  >
    <div class="sos-artwork-card__frame" :style="{ '--sos-artwork-ratio': ratioValue }">
      <img v-if="image" :src="image" :alt="imageAlt || title" />
      <div class="sos-artwork-card__veil">
        <h3 class="sos-artwork-card__veil-title">{{ title }}</h3>
        <span v-if="author" class="sos-artwork-card__veil-author">{{ author }}</span>
      </div>
    </div>
    <div class="sos-artwork-card__caption">
      <div>
        <h3 class="sos-artwork-card__title">{{ title }}</h3>
        <p v-if="author" class="sos-artwork-card__author">{{ author }}</p>
      </div>
      <div v-if="likes !== undefined || views !== undefined" class="sos-artwork-card__stats">
        <span v-if="likes !== undefined" class="sos-stat">♥ {{ likes }}</span>
        <span v-if="views !== undefined" class="sos-stat">◉ {{ views }}</span>
      </div>
    </div>
  </article>
</template>
