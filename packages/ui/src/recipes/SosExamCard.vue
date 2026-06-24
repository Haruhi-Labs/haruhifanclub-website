<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string
    subject?: string
    score?: string
    meta?: string[]
    interactive?: boolean
  }>(),
  {
    subject: undefined,
    score: undefined,
    meta: () => [],
    interactive: true,
  }
)

const emit = defineEmits<{
  click: []
}>()
</script>

<template>
  <article
    class="sos-card sos-exam-card"
    :class="{ 'sos-card--interactive': interactive }"
    :role="interactive ? 'button' : undefined"
    :tabindex="interactive ? 0 : undefined"
    @click="interactive && emit('click')"
    @keydown.enter.self="interactive && emit('click')"
    @keydown.space.prevent.self="interactive && emit('click')"
  >
    <div class="sos-exam-card__paper">
      <span v-if="score" class="sos-exam-card__stamp">{{ score }}</span>
      <div class="sos-exam-card__content">
        <span v-if="subject" class="sos-exam-card__subject">{{ subject }}</span>
        <h3 class="sos-exam-card__title">{{ title }}</h3>
        <hr class="sos-exam-card__rule" />
        <div v-if="meta.length" class="sos-exam-card__meta">
          <span v-for="(item, index) in meta" :key="index">{{ item }}</span>
        </div>
        <slot />
      </div>
    </div>
  </article>
</template>
