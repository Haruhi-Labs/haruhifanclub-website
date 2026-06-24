<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string
    placeholder?: string
    size?: 'md' | 'lg'
    square?: boolean
    submit?: boolean
    disabled?: boolean
    ariaLabel?: string
  }>(),
  {
    modelValue: '',
    placeholder: '搜索…',
    size: 'md',
    square: false,
    submit: false,
    disabled: false,
    ariaLabel: '搜索',
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  search: [value: string]
  clear: []
}>()

const classes = computed(() => [
  'sos-search',
  props.size === 'lg' ? 'sos-search--lg' : '',
  props.square ? 'sos-search--square' : '',
])

const onInput = (e: Event) => emit('update:modelValue', (e.target as HTMLInputElement).value)
const onEnter = () => emit('search', props.modelValue)
const onClear = () => {
  emit('update:modelValue', '')
  emit('clear')
}
</script>

<template>
  <div :class="classes">
    <span class="sos-search__icon" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="7" />
        <path d="M21 21l-4.3-4.3" />
      </svg>
    </span>
    <input
      class="sos-search__input"
      type="search"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :aria-label="ariaLabel"
      @input="onInput"
      @keydown.enter="onEnter"
    />
    <button
      v-if="modelValue"
      type="button"
      class="sos-search__clear"
      aria-label="清除搜索"
      @click="onClear"
    >
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
        <path d="M18 6L6 18M6 6l12 12" />
      </svg>
    </button>
    <button
      v-if="submit"
      type="button"
      class="sos-search__submit"
      :aria-label="ariaLabel"
      :disabled="disabled"
      @click="onEnter"
    >
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="7" />
        <path d="M21 21l-4.3-4.3" />
      </svg>
    </button>
  </div>
</template>
