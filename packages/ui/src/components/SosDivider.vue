<script setup lang="ts">
defineProps<{
  label?: string
}>()
</script>

<template>
  <div v-if="label" class="sos-divider sos-divider--labeled" role="separator">
    {{ label }}
  </div>
  <hr v-else class="sos-divider" />
</template>
