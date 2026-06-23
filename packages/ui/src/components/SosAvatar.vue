<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    src?: string
    alt?: string
    name?: string
    size?: 'sm' | 'md' | 'lg'
    square?: boolean
  }>(),
  {
    src: undefined,
    alt: undefined,
    name: undefined,
    size: 'md',
    square: false,
  }
)

const classes = computed(() => [
  'sos-avatar',
  props.size !== 'md' ? `sos-avatar--${props.size}` : undefined,
  props.square ? 'sos-avatar--square' : undefined,
])

const initials = computed(() => {
  if (!props.name) return ''
  return props.name.trim().slice(0, 2)
})
</script>

<template>
  <span :class="classes">
    <img v-if="src" :src="src" :alt="alt || name || ''" />
    <slot v-else>{{ initials }}</slot>
  </span>
</template>
