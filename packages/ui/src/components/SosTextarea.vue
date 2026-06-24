<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue?: string
    placeholder?: string
    rows?: number
    invalid?: boolean
    disabled?: boolean
    id?: string
  }>(),
  {
    modelValue: '',
    placeholder: undefined,
    rows: 4,
    invalid: false,
    disabled: false,
    id: undefined,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

function onInput(event: Event) {
  emit('update:modelValue', (event.target as HTMLTextAreaElement).value)
}
</script>

<template>
  <textarea
    :id="id"
    class="sos-textarea"
    :rows="rows"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-invalid="invalid || undefined"
    @input="onInput"
  />
</template>
