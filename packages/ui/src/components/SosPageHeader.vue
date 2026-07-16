<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    as?: string
    titleAs?: string
    eyebrow?: string
    title?: string
    copy?: string
    meta?: string
  }>(),
  {
    as: 'header',
    titleAs: 'h1',
    eyebrow: undefined,
    title: undefined,
    copy: undefined,
    meta: undefined,
  }
)
</script>

<template>
  <component :is="props.as" class="sos-page-header">
    <div class="sos-page-header__content">
      <p v-if="props.eyebrow || $slots.eyebrow" class="sos-eyebrow sos-page-header__eyebrow">
        <slot name="eyebrow">{{ props.eyebrow }}</slot>
      </p>
      <component
        :is="props.titleAs"
        v-if="props.title || $slots.title"
        class="sos-page-header__title"
      >
        <slot name="title">{{ props.title }}</slot>
      </component>
      <p v-if="props.copy || $slots.default" class="sos-page-header__copy">
        <slot>{{ props.copy }}</slot>
      </p>
      <p v-if="props.meta || $slots.meta" class="sos-page-header__meta">
        <slot name="meta">{{ props.meta }}</slot>
      </p>
    </div>
    <div v-if="$slots.actions" class="sos-page-header__actions">
      <slot name="actions" />
    </div>
  </component>
</template>
