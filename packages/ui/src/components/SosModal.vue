<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue'

const props = withDefaults(
  defineProps<{
    open?: boolean
    title?: string
    wide?: boolean
    closeOnBackdrop?: boolean
  }>(),
  {
    open: false,
    title: undefined,
    wide: false,
    closeOnBackdrop: true,
  }
)

const emit = defineEmits<{
  'update:open': [value: boolean]
  close: []
}>()

function close() {
  emit('update:open', false)
  emit('close')
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close()
}

watch(
  () => props.open,
  (open) => {
    if (typeof document === 'undefined') return
    if (open) {
      document.addEventListener('keydown', onKeydown)
      document.body.style.overflow = 'hidden'
    } else {
      document.removeEventListener('keydown', onKeydown)
      document.body.style.overflow = ''
    }
  }
)

onBeforeUnmount(() => {
  if (typeof document === 'undefined') return
  document.removeEventListener('keydown', onKeydown)
  document.body.style.overflow = ''
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="sos-overlay sos-scope" @click.self="closeOnBackdrop && close()">
      <div class="sos-modal" :class="{ 'sos-modal--wide': wide }" role="dialog" aria-modal="true">
        <header v-if="title || $slots.header" class="sos-modal__header">
          <slot name="header">
            <h2 class="sos-modal__title">{{ title }}</h2>
          </slot>
          <button
            type="button"
            class="sos-button sos-button--ghost sos-icon-button sos-button--sm"
            aria-label="关闭"
            @click="close"
          >
            ×
          </button>
        </header>
        <div class="sos-modal__body">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="sos-modal__footer">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>
