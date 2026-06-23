<script setup lang="ts">
withDefaults(
  defineProps<{
    label?: string
    title: string
    subtitle?: string
    excerpt?: string
    image?: string
    imageAlt?: string
    date?: string
    author?: string
    tags?: string[]
    pinned?: boolean
    interactive?: boolean
  }>(),
  {
    label: undefined,
    subtitle: undefined,
    excerpt: undefined,
    image: undefined,
    imageAlt: '',
    date: undefined,
    author: undefined,
    tags: () => [],
    pinned: false,
    interactive: true,
  }
)

const emit = defineEmits<{
  click: []
  tag: [tag: string]
}>()
</script>

<template>
  <article
    class="sos-card sos-article-card"
    :class="{ 'sos-card--interactive': interactive }"
    :role="interactive ? 'button' : undefined"
    :tabindex="interactive ? 0 : undefined"
    @click="interactive && emit('click')"
    @keydown.enter.self="interactive && emit('click')"
  >
    <div v-if="image" class="sos-card__media sos-article-card__media">
      <img :src="image" :alt="imageAlt || title" />
    </div>
    <div class="sos-card__body">
      <div v-if="label || pinned" class="sos-article-card__head">
        <span v-if="label" class="sos-article-card__label">{{ label }}</span>
        <span v-if="pinned" class="sos-badge sos-badge--signal">置顶</span>
      </div>
      <h3 class="sos-card__heading sos-article-card__title">{{ title }}</h3>
      <p v-if="subtitle" class="sos-article-card__subtitle">{{ subtitle }}</p>
      <p v-if="excerpt" class="sos-card__excerpt sos-article-card__excerpt">{{ excerpt }}</p>
      <slot />
      <footer
        v-if="tags.length || date || author"
        class="sos-card__footer sos-article-card__footer"
      >
        <div v-if="tags.length" class="sos-card__tags">
          <button
            v-for="tag in tags"
            :key="tag"
            type="button"
            class="sos-card__tag"
            @click.stop="emit('tag', tag)"
          >
            #{{ tag }}
          </button>
        </div>
        <span v-if="date || author" class="sos-article-card__meta">
          <template v-if="author">{{ author }} · </template>{{ date }}
        </span>
      </footer>
    </div>
  </article>
</template>
