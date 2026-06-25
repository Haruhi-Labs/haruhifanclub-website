<script setup lang="ts">
import { computed } from 'vue'

type PageContained = 'none' | 'content' | 'wide' | 'reading'
type PageGap = 'default' | 'tight' | 'loose'

const props = withDefaults(
  defineProps<{
    as?: string
    site?: string
    density?: string
    contained?: PageContained
    gap?: PageGap
    scoped?: boolean
  }>(),
  {
    as: 'main',
    site: undefined,
    density: undefined,
    contained: 'content',
    gap: 'default',
    scoped: true,
  }
)

const classes = computed(() => [
  props.scoped ? 'sos-scope' : undefined,
  'sos-page',
  props.contained !== 'none'
    ? props.contained === 'content'
      ? 'sos-page--contained'
      : `sos-page--${props.contained}`
    : undefined,
  props.gap !== 'default' ? `sos-page--${props.gap}` : undefined,
])
</script>

<template>
  <component
    :is="props.as"
    :class="classes"
    :data-sos-site="props.site"
    :data-sos-density="props.density"
  >
    <slot />
  </component>
</template>
