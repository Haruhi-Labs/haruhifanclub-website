<script setup lang="ts">
withDefaults(
  defineProps<{
    as?: string
    href?: string
    active?: boolean
  }>(),
  {
    as: 'a',
    href: undefined,
    active: false,
  }
)
</script>

<template>
  <component :is="as" class="sos-navlink" :href="href" :aria-current="active ? 'page' : undefined">
    <slot />
  </component>
</template>
