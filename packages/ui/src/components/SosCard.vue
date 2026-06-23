<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    interactive?: boolean
  }>(),
  {
    as: 'article',
    interactive: false,
  }
)

const classes = computed(() => ({
  'sos-card': true,
  'sos-card--interactive': props.interactive,
}))
</script>

<template>
  <component :is="props.as" :class="classes">
    <slot name="media" />
    <div class="sos-card__body">
      <slot />
      <footer v-if="$slots.footer" class="sos-card__footer">
        <slot name="footer" />
      </footer>
    </div>
  </component>
</template>
