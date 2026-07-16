<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'

interface SosTabItem {
  value: string | number
  label: string
  disabled?: boolean
  controls?: string
}

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    items?: SosTabItem[]
    variant?: 'pill' | 'underline'
    label?: string
  }>(),
  {
    modelValue: undefined,
    items: () => [],
    variant: 'pill',
    label: '分段导航',
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const listClass = computed(() => (props.variant === 'underline' ? 'sos-tablist' : 'sos-tabs'))
const tabClass = computed(() => (props.variant === 'underline' ? 'sos-tablist__tab' : 'sos-tab'))
const tabButtons = ref<HTMLButtonElement[]>([])

function setTabButton(element: unknown, index: number) {
  if (element instanceof HTMLButtonElement) tabButtons.value[index] = element
}

function selectTab(index: number) {
  const item = props.items[index]
  if (!item || item.disabled) return
  emit('update:modelValue', item.value)
  nextTick(() => tabButtons.value[index]?.focus())
}

function moveTab(event: KeyboardEvent, index: number) {
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return
  event.preventDefault()
  const available = props.items
    .map((item, itemIndex) => (!item.disabled ? itemIndex : -1))
    .filter((itemIndex) => itemIndex >= 0)
  if (!available.length) return
  const current = Math.max(0, available.indexOf(index))
  const target =
    event.key === 'Home'
      ? available[0]
      : event.key === 'End'
        ? available[available.length - 1]
        : available[
            (current + (event.key === 'ArrowRight' ? 1 : -1) + available.length) % available.length
          ]
  selectTab(target)
}
</script>

<template>
  <div :class="listClass" role="tablist" :aria-label="label">
    <button
      v-for="item in items"
      :key="item.value"
      :ref="(element) => setTabButton(element, items.indexOf(item))"
      type="button"
      role="tab"
      :class="tabClass"
      :aria-selected="item.value === modelValue ? 'true' : 'false'"
      :aria-controls="item.controls"
      :tabindex="item.value === modelValue ? 0 : -1"
      :disabled="item.disabled"
      @click="selectTab(items.indexOf(item))"
      @keydown="moveTab($event, items.indexOf(item))"
    >
      {{ item.label }}
    </button>
  </div>
</template>
