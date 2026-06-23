<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue?: boolean
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
  'update:modelValue': [value: boolean]
}>()

function onChange(event: Event) {
  emit('update:modelValue', (event.target as HTMLInputElement).checked)
}
</script>

<template>
  <label class="sos-check">
    <input
      :type="type"
      :name="name"
      :value="value"
      :checked="modelValue"
      :disabled="disabled"
      @change="onChange"
    />
    <span><slot /></span>
  </label>
</template>
