<script setup lang="ts">
import { computed } from 'vue'

interface ProductProgress {
  value: number
  max: number
  label?: string
  valueLabel?: string
}

const props = withDefaults(
  defineProps<{
    title: string
    desc?: string
    image?: string
    imageAlt?: string
    price: number | string
    originalPrice?: number | string
    badge?: string
    badgeTone?: 'accent' | 'signal' | 'danger'
    soldOut?: boolean
    soldOutLabel?: string
    state?: string
    progress?: ProductProgress
    interactive?: boolean
  }>(),
  {
    desc: undefined,
    image: undefined,
    imageAlt: '',
    originalPrice: undefined,
    badge: undefined,
    badgeTone: 'accent',
    soldOut: false,
    soldOutLabel: '售罄',
    state: undefined,
    progress: undefined,
    interactive: true,
  }
)

const emit = defineEmits<{
  click: []
}>()

const fmt = (value: number | string | undefined) => {
  if (value === undefined) return ''
  return typeof value === 'number' ? `¥${value}` : value
}
const nowPrice = computed(() => fmt(props.price))
const wasPrice = computed(() => fmt(props.originalPrice))
const pct = computed(() =>
  props.progress ? Math.min(100, Math.round((props.progress.value / props.progress.max) * 100)) : 0
)
</script>

<template>
  <article
    class="sos-card sos-product-card"
    :class="{ 'sos-card--interactive': interactive }"
    @click="interactive && emit('click')"
  >
    <div class="sos-product-card__media">
      <span v-if="badge" class="sos-ribbon" :class="`sos-ribbon--${badgeTone}`">{{ badge }}</span>
      <img v-if="image" :src="image" :alt="imageAlt || title" />
      <div v-if="soldOut" class="sos-product-card__veil">{{ soldOutLabel }}</div>
      <div v-if="$slots.actions" class="sos-product-card__actions">
        <slot name="actions" />
      </div>
    </div>
    <div class="sos-card__body">
      <h3 class="sos-product-card__title">{{ title }}</h3>
      <p v-if="desc" class="sos-product-card__desc">{{ desc }}</p>
      <div v-if="progress" class="sos-progress">
        <div class="sos-progress__meta">
          <span>{{ progress.label || '预售进度' }}</span>
          <span>{{ progress.valueLabel || `${progress.value}/${progress.max} · ${pct}%` }}</span>
        </div>
        <div class="sos-progress__track">
          <span class="sos-progress__fill" :style="{ width: `${pct}%` }" />
        </div>
      </div>
      <div class="sos-product-card__price-row">
        <span class="sos-price">
          <span class="sos-price__now">{{ nowPrice }}</span>
          <span v-if="originalPrice" class="sos-price__original">{{ wasPrice }}</span>
        </span>
        <span v-if="state" class="sos-product-card__state">{{ state }}</span>
      </div>
    </div>
  </article>
</template>
