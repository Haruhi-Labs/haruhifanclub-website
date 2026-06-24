<script setup lang="ts">
import { computed } from 'vue'
import { safeUrl } from '../internal/safe-url'

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

// 协议白名单过滤后再决定是否渲染为链接，拦截 javascript: 等危险值
const safeHref = computed(() => safeUrl(props.href))
const tag = computed(() => (safeHref.value ? 'a' : props.as))
const classes = computed(() => ({
  'sos-brand-lockup': true,
  'sos-brand-lockup--compact': props.compact,
}))
</script>

<template>
  <component :is="tag" :class="classes" :href="safeHref">
    <span v-if="props.logoSrc" class="sos-brand-lockup__mark">
      <img :src="props.logoSrc" :alt="props.logoAlt" />
    </span>
    <span class="sos-brand-lockup__text">
      <strong>{{ props.title }}</strong>
      <small v-if="props.subtitle">{{ props.subtitle }}</small>
    </span>
  </component>
</template>
