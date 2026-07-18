<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'
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
const modalRef = ref<HTMLElement | null>(null)
const titleId = `sos-modal-title-${useId()}`
let previousFocus: HTMLElement | null = null

function close() {
  emit('update:open', false)
  emit('close')
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }
  if (event.key !== 'Tab' || !modalRef.value) return
  const focusable = Array.from(
    modalRef.value.querySelectorAll<HTMLElement>(
      'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])'
    )
  ).filter((element) => !element.hasAttribute('hidden'))
  if (!focusable.length) {
    event.preventDefault()
    modalRef.value.focus()
    return
  }
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

// 本实例当前是否持有滚动锁，确保解锁次数与加锁次数对齐
let locked = false

watch(
  () => props.open,
  (open) => {
    if (typeof document === 'undefined') return
    if (open) {
      previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
      document.addEventListener('keydown', onKeydown)
      if (!locked) {
        lockScroll()
        locked = true
      }
      nextTick(() => {
        const initial = modalRef.value?.querySelector<HTMLElement>(
          '[autofocus],input:not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled]),a[href]'
        )
        ;(initial || modalRef.value)?.focus()
      })
    } else {
      document.removeEventListener('keydown', onKeydown)
      if (locked) {
        unlockScroll()
        locked = false
      }
      const target = previousFocus
      previousFocus = null
      nextTick(() => target?.focus())
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
      <div
        ref="modalRef"
        class="sos-modal"
        :class="{ 'sos-modal--wide': wide }"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="title ? titleId : undefined"
        tabindex="-1"
      >
        <header v-if="title || $slots.header" class="sos-modal__header">
          <slot name="header">
            <h2 :id="titleId" class="sos-modal__title">{{ title }}</h2>
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
