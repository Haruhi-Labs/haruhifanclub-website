<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    surface?: boolean
  }>(),
  {
    as: 'div',
    surface: false,
  }
)

const classes = computed(() => ['sos-toolbar', props.surface ? 'sos-toolbar--surface' : undefined])
</script>

<template>
  <component :is="props.as" :class="classes">
    <div class="sos-toolbar__group">
      <slot />
    </div>
    <div v-if="$slots.actions" class="sos-toolbar__group">
      <slot name="actions" />
    </div>
  </component>
</template>
