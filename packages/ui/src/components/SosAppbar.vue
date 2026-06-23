<script setup lang="ts">
defineProps<{
  as?: string
}>()
</script>

<template>
  <component :is="as || 'header'" class="sos-appbar">
    <div class="sos-appbar__inner">
      <slot name="brand" />
      <slot />
      <div class="sos-inline sos-inline--tight">
        <slot name="actions" />
      </div>
    </div>
  </component>
</template>
