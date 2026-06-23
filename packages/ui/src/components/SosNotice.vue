<script setup lang="ts">
import { computed } from 'vue'

type NoticeTone = 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(
  defineProps<{
    title?: string
    tone?: NoticeTone
  }>(),
  {
    title: undefined,
    tone: 'info',
  }
)

const classes = computed(() => [
  'sos-notice',
  props.tone !== 'info' ? `sos-notice--${props.tone}` : undefined,
])
</script>

<template>
  <aside :class="classes">
    <span class="sos-notice__icon" aria-hidden="true">
      <slot name="icon">i</slot>
    </span>
    <div>
      <h3 v-if="props.title" class="sos-notice__title">
        {{ props.title }}
      </h3>
      <div class="sos-notice__copy">
        <slot />
      </div>
    </div>
    <slot name="action" />
  </aside>
</template>
