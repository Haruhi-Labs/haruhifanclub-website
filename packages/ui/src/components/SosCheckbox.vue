<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: boolean | string | number
    type?: 'checkbox' | 'radio'
    value?: string | number
    name?: string
    disabled?: boolean
  }>(),
  {
    modelValue: false,
    type: 'checkbox',
    value: undefined,
    name: undefined,
    disabled: false,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: boolean | string | number]
}>()

// radio 用「当前选中值」表达选择，checkbox 用布尔
const isChecked = computed(() =>
  props.type === 'radio' ? props.modelValue === props.value : Boolean(props.modelValue)
)

function onChange(event: Event) {
  const el = event.target as HTMLInputElement
  if (props.type === 'radio') {
    if (el.checked && props.value !== undefined) emit('update:modelValue', props.value)
  } else {
    emit('update:modelValue', el.checked)
  }
}
</script>

<template>
  <label class="sos-check">
    <input
      :type="type"
      :name="name"
      :value="value"
      :checked="isChecked"
      :disabled="disabled"
      @change="onChange"
    />
    <span><slot /></span>
  </label>
</template>
