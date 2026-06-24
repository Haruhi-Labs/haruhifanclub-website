<script setup lang="ts">
interface SosSelectOption {
  label: string
  value: string | number
  disabled?: boolean
}

const props = withDefaults(
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
  'update:modelValue': [value: string | number]
}>()

function onChange(event: Event) {
  const raw = (event.target as HTMLSelectElement).value
  // 还原选项原始类型，避免 number 选项被强制转成 string
  const matched = props.options.find((opt) => String(opt.value) === raw)
  emit('update:modelValue', matched ? matched.value : raw)
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
