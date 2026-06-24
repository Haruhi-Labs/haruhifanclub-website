<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    type?: string
    placeholder?: string
    invalid?: boolean
    disabled?: boolean
    id?: string
  }>(),
  {
    modelValue: '',
    type: 'text',
    placeholder: undefined,
    invalid: false,
    disabled: false,
    id: undefined,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

function onInput(event: Event) {
  const el = event.target as HTMLInputElement
  // number 类型按数值发射（空值除外），保持与 modelValue 的类型契约一致
  emit(
    'update:modelValue',
    props.type === 'number' && el.value !== '' ? el.valueAsNumber : el.value
  )
}
</script>

<template>
  <input
    :id="id"
    class="sos-input"
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-invalid="invalid || undefined"
    @input="onInput"
  />
</template>
