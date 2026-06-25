<script setup lang="ts">
withDefaults(
  defineProps<{
    as?: string
    plain?: boolean
  }>(),
  {
    as: 'p',
    plain: false,
  }
)
</script>

<template>
  <component :is="as" class="sos-eyebrow" :class="{ 'sos-eyebrow--plain': plain }">
    <slot />
  </component>
</template>
