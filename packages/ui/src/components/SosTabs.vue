<script setup lang="ts">
import { computed } from 'vue'

interface SosTabItem {
  value: string | number
  label: string
  disabled?: boolean
}

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    items?: SosTabItem[]
    variant?: 'pill' | 'underline'
  }>(),
  {
    modelValue: undefined,
    items: () => [],
    variant: 'pill',
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const listClass = computed(() => (props.variant === 'underline' ? 'sos-tablist' : 'sos-tabs'))
const tabClass = computed(() => (props.variant === 'underline' ? 'sos-tablist__tab' : 'sos-tab'))
</script>

<template>
  <div :class="listClass" role="tablist">
    <button
      v-for="item in items"
      :key="item.value"
      type="button"
      role="tab"
      :class="tabClass"
      :aria-selected="item.value === modelValue ? 'true' : 'false'"
      :disabled="item.disabled"
      @click="emit('update:modelValue', item.value)"
    >
      {{ item.label }}
    </button>
  </div>
</template>
