<script setup lang="ts">
interface SosCrumb {
  label: string
  href?: string
}

withDefaults(
  defineProps<{
    items?: SosCrumb[]
    separator?: string
  }>(),
  {
    items: () => [],
    separator: '/',
  }
)
</script>

<template>
  <nav class="sos-breadcrumb" aria-label="面包屑">
    <template v-for="(item, index) in items" :key="index">
      <a
        v-if="item.href && index < items.length - 1"
        class="sos-breadcrumb__item"
        :href="item.href"
      >
        {{ item.label }}
      </a>
      <span
        v-else
        class="sos-breadcrumb__item"
        :aria-current="index === items.length - 1 ? 'page' : undefined"
      >
        {{ item.label }}
      </span>
      <span v-if="index < items.length - 1" class="sos-breadcrumb__sep" aria-hidden="true">
        {{ separator }}
      </span>
    </template>
  </nav>
</template>
