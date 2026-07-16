<script setup>
import { computed, nextTick, ref, useId, watch } from 'vue'
import { resolveUploadUrl } from '@haruhi/api-client'

const props = defineProps({
  modelValue: { type: [String, Number], default: '' },
  options: { type: Array, default: () => [] },
  label: { type: String, default: '选择成员' },
  placeholder: { type: String, default: '输入昵称或用户名搜索' },
  emptyText: { type: String, default: '没有匹配的已加入成员' },
  disabled: { type: Boolean, default: false },
})

const emit = defineEmits(['update:modelValue'])
const root = ref(null)
const input = ref(null)
const query = ref('')
const open = ref(false)
const activeIndex = ref(0)
const inputId = `member-search-${useId()}`
const listId = `${inputId}-listbox`

const selectedOption = computed(() =>
  props.options.find((option) => String(option.value) === String(props.modelValue))
)
const filteredOptions = computed(() => {
  const needle = query.value.trim().toLocaleLowerCase('zh-CN')
  if (!needle) return props.options
  return props.options.filter((option) =>
    [option.label, option.displayName, option.username, option.subtitle]
      .filter(Boolean)
      .some((value) => String(value).toLocaleLowerCase('zh-CN').includes(needle))
  )
})

watch(
  () => props.modelValue,
  () => {
    query.value = selectedOption.value?.label || ''
  },
  { immediate: true }
)

watch(filteredOptions, () => {
  activeIndex.value = 0
})

function showOptions(event) {
  if (props.disabled) return
  open.value = true
  activeIndex.value = Math.max(
    0,
    filteredOptions.value.findIndex(
      (option) => String(option.value) === String(props.modelValue)
    )
  )
  event?.currentTarget?.select?.()
}

function updateQuery(event) {
  query.value = event.target.value
  emit('update:modelValue', '')
  open.value = true
}

function choose(option) {
  emit('update:modelValue', option.value)
  query.value = option.label
  open.value = false
  nextTick(() => input.value?.focus())
}

function clear() {
  emit('update:modelValue', '')
  query.value = ''
  open.value = true
  nextTick(() => input.value?.focus())
}

function move(direction) {
  if (!open.value) open.value = true
  if (!filteredOptions.value.length) return
  activeIndex.value =
    (activeIndex.value + direction + filteredOptions.value.length) % filteredOptions.value.length
}

function selectActive() {
  const option = filteredOptions.value[activeIndex.value]
  if (open.value && option) choose(option)
}

function closeOnFocusOut(event) {
  if (!root.value?.contains(event.relatedTarget)) open.value = false
}
</script>

<template>
  <div ref="root" class="member-search-select" @focusout="closeOnFocusOut">
    <label :for="inputId" class="member-search-select__label">{{ label }}</label>
    <div class="member-search-select__control">
      <input
        :id="inputId"
        ref="input"
        class="sos-input"
        type="search"
        role="combobox"
        autocomplete="off"
        :value="query"
        :placeholder="placeholder"
        :disabled="disabled"
        :aria-expanded="open"
        :aria-controls="listId"
        :aria-activedescendant="
          open && filteredOptions[activeIndex]
            ? `${listId}-option-${filteredOptions[activeIndex].value}`
            : undefined
        "
        aria-autocomplete="list"
        @focus="showOptions"
        @click="showOptions"
        @input="updateQuery"
        @keydown.down.prevent="move(1)"
        @keydown.up.prevent="move(-1)"
        @keydown.enter.prevent="selectActive"
        @keydown.esc.prevent="open = false"
      />
      <button
        v-if="modelValue"
        type="button"
        class="member-search-select__clear"
        aria-label="清除已选择成员"
        @mousedown.prevent
        @click="clear"
      >
        清除
      </button>
    </div>
    <div v-if="open" :id="listId" class="member-search-select__menu" role="listbox">
      <button
        v-for="(option, index) in filteredOptions"
        :id="`${listId}-option-${option.value}`"
        :key="option.value"
        type="button"
        role="option"
        class="member-search-select__option"
        :class="{ 'is-active': index === activeIndex }"
        :aria-selected="String(option.value) === String(modelValue)"
        @mouseenter="activeIndex = index"
        @mousedown.prevent
        @click="choose(option)"
      >
        <img
          v-if="option.avatar"
          :src="resolveUploadUrl(option.avatar)"
          :alt="`${option.displayName || option.label}的头像`"
        />
        <span v-else class="member-search-select__avatar" aria-hidden="true">
          {{ (option.displayName || option.label).slice(0, 1) }}
        </span>
        <span>
          <strong>{{ option.displayName || option.label }}</strong>
          <small v-if="option.username">@{{ option.username }}</small>
          <small v-else-if="option.subtitle">{{ option.subtitle }}</small>
        </span>
      </button>
      <p v-if="!filteredOptions.length" class="member-search-select__empty" role="status">
        {{ emptyText }}
      </p>
    </div>
  </div>
</template>
