<script setup lang="ts">
import { computed, useId } from 'vue'

const props = withDefaults(
  defineProps<{
    label?: string
    help?: string
    error?: string
    required?: boolean
    forId?: string
  }>(),
  {
    label: undefined,
    help: undefined,
    error: undefined,
    required: false,
    forId: undefined,
  }
)

// 每个实例唯一 id，避免多个未传 forId 的字段共享同一 help id
const autoId = useId()
const describedBy = computed(() =>
  props.help || props.error ? `${props.forId || autoId}-help` : undefined
)
</script>

<template>
  <div class="sos-field" :class="{ 'sos-field--error': Boolean(props.error) }">
    <label v-if="props.label" class="sos-field__label" :for="props.forId">
      {{ props.label }}
      <span v-if="props.required" class="sos-field__required" aria-hidden="true">*</span>
    </label>
    <slot
      :id="props.forId"
      :aria-describedby="describedBy"
      :aria-invalid="Boolean(props.error) || undefined"
    />
    <p
      v-if="props.error || props.help"
      :id="describedBy"
      class="sos-field__help"
      :role="props.error ? 'alert' : undefined"
    >
      {{ props.error || props.help }}
    </p>
  </div>
</template>
