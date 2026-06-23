<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    interactive?: boolean
    selected?: boolean
    loading?: boolean
  }>(),
  {
    as: 'article',
    interactive: false,
    selected: false,
    loading: false,
  }
)

const classes = computed(() => ({
  'sos-card': true,
  'sos-card--interactive': props.interactive,
}))
</script>

<template>
  <component
    :is="props.as"
    :class="classes"
    :aria-selected="props.selected ? 'true' : undefined"
    :aria-busy="props.loading ? 'true' : undefined"
  >
    <slot name="media" />
    <div class="sos-card__body">
      <slot />
      <footer v-if="$slots.footer" class="sos-card__footer">
        <slot name="footer" />
      </footer>
    </div>
  </component>
</template>
