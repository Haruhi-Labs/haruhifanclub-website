<script setup lang="ts">
defineProps<{
  title: string
  copy?: string
}>()
</script>

<template>
  <section class="sos-empty-state">
    <span v-if="$slots.icon" class="sos-empty-state__icon" aria-hidden="true">
      <slot name="icon" />
    </span>
    <h3 class="sos-empty-state__title">{{ title }}</h3>
    <p v-if="copy || $slots.default" class="sos-empty-state__copy">
      <slot>{{ copy }}</slot>
    </p>
    <div v-if="$slots.actions" class="sos-empty-state__actions">
      <slot name="actions" />
    </div>
  </section>
</template>
