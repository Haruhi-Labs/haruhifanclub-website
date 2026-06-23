<script setup lang="ts">
interface SosSelectOption {
  label: string
  value: string | number
  disabled?: boolean
}

withDefaults(
  defineProps<{
    modelValue?: string | number
    options?: SosSelectOption[]
    disabled?: boolean
    invalid?: boolean
    id?: string
  }>(),
  {
    modelValue: '',
    options: () => [],
    disabled: false,
    invalid: false,
    id: undefined,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

function onChange(event: Event) {
  emit('update:modelValue', (event.target as HTMLSelectElement).value)
}
</script>

<template>
  <select
    :id="id"
    class="sos-select"
    :value="modelValue"
    :disabled="disabled"
    :aria-invalid="invalid || undefined"
    @change="onChange"
  >
    <slot>
      <option
        v-for="option in options"
        :key="option.value"
        :value="option.value"
        :disabled="option.disabled"
      >
        {{ option.label }}
      </option>
    </slot>
  </select>
</template>
