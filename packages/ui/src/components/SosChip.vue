<script setup lang="ts">
withDefaults(
  defineProps<{
    pressed?: boolean
    removable?: boolean
    disabled?: boolean
  }>(),
  {
    pressed: false,
    removable: false,
    disabled: false,
  }
)

const emit = defineEmits<{
  toggle: []
  remove: []
}>()
</script>

<template>
  <button
    type="button"
    class="sos-chip"
    :aria-pressed="pressed ? 'true' : 'false'"
    :disabled="disabled"
    @click="emit('toggle')"
  >
    <slot />
    <span v-if="removable" class="sos-chip__remove" aria-hidden="true" @click.stop="emit('remove')">
      ×
    </span>
  </button>
</template>
