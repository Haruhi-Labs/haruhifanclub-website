<script setup lang="ts">
defineProps<{
  label: string
}>()
</script>

<template>
  <span class="sos-tooltip">
    <slot />
    <span class="sos-tooltip__bubble" role="tooltip">{{ label }}</span>
  </span>
</template>
