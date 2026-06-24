<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    size?: 'md' | 'lg'
    label?: string
  }>(),
  {
    size: 'md',
    label: '加载中',
  }
)

const classes = computed(() => ['sos-spinner', props.size === 'lg' ? 'sos-spinner--lg' : undefined])
</script>

<template>
  <span :class="classes" role="status" :aria-label="label" />
</template>
