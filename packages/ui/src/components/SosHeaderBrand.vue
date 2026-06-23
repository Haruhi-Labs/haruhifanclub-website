<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    href?: string
    logoSrc?: string
    logoAlt?: string
    title: string
    subtitle?: string
    compact?: boolean
  }>(),
  {
    as: 'div',
    href: undefined,
    logoSrc: undefined,
    logoAlt: '',
    subtitle: undefined,
    compact: false,
  }
)

const tag = computed(() => (props.href ? 'a' : props.as))
const classes = computed(() => ({
  'sos-brand-lockup': true,
  'sos-brand-lockup--compact': props.compact,
}))
</script>

<template>
  <component :is="tag" :class="classes" :href="props.href">
    <span v-if="props.logoSrc" class="sos-brand-lockup__mark">
      <img :src="props.logoSrc" :alt="props.logoAlt" />
    </span>
    <span class="sos-brand-lockup__text">
      <strong>{{ props.title }}</strong>
      <small v-if="props.subtitle">{{ props.subtitle }}</small>
    </span>
  </component>
</template>
