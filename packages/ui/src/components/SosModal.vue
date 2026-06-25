<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue'
import { lockScroll, unlockScroll } from '../internal/scroll-lock'

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

// 本实例当前是否持有滚动锁，确保解锁次数与加锁次数对齐
let locked = false

watch(
  () => props.open,
  (open) => {
    if (typeof document === 'undefined') return
    if (open) {
      document.addEventListener('keydown', onKeydown)
      if (!locked) {
        lockScroll()
        locked = true
      }
    } else {
      document.removeEventListener('keydown', onKeydown)
      if (locked) {
        unlockScroll()
        locked = false
      }
    }
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  if (typeof document === 'undefined') return
  document.removeEventListener('keydown', onKeydown)
  if (locked) {
    unlockScroll()
    locked = false
  }
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
