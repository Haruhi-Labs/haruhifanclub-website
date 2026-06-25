<script setup lang="ts">
type MediaRatio = '1:1' | '4:3' | '3:4' | '2:3'

const props = withDefaults(
  defineProps<{
    as?: string
    ratio?: MediaRatio
  }>(),
  {
    as: 'div',
    ratio: '4:3',
  }
)
</script>

<template>
  <component :is="props.as" class="sos-media-frame" :data-ratio="props.ratio">
    <slot />
  </component>
</template>
