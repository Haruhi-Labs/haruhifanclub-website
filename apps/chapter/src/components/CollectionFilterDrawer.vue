<script setup>
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps({
  open: { type: Boolean, default: false },
  title: { type: String, default: '搜索与筛选' },
  panelId: { type: String, default: 'collection-filter-drawer' },
})

const emit = defineEmits(['update:open'])
const panel = ref(null)
let previousFocus = null
let previousBodyOverflow = ''
let locked = false

function close() {
  emit('update:open', false)
}

function focusableElements() {
  if (!panel.value) return []
  return Array.from(
    panel.value.querySelectorAll(
      'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])'
    )
  ).filter((element) => !element.hasAttribute('hidden'))
}

function onKeydown(event) {
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }
  if (event.key !== 'Tab' || !panel.value) return
  const focusable = focusableElements()
  if (!focusable.length) {
    event.preventDefault()
    panel.value.focus()
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

function unlockPage() {
  document.removeEventListener('keydown', onKeydown)
  if (!locked) return
  document.body.style.overflow = previousBodyOverflow
  locked = false
}

watch(
  () => props.open,
  async (open) => {
    if (typeof document === 'undefined') return
    if (open) {
      previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
      previousBodyOverflow = document.body.style.overflow
      document.body.style.overflow = 'hidden'
      locked = true
      document.addEventListener('keydown', onKeydown)
      await nextTick()
      const initial = panel.value?.querySelector(
        '.collection-filter-drawer__body [autofocus],.collection-filter-drawer__body input:not([disabled]),.collection-filter-drawer__body select:not([disabled]),.collection-filter-drawer__body textarea:not([disabled]),.collection-filter-drawer__body button:not([disabled])'
      )
      ;(initial || panel.value)?.focus()
    } else {
      unlockPage()
      const target = previousFocus
      previousFocus = null
      await nextTick()
      target?.focus()
    }
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') unlockPage()
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="collection-filter-drawer__overlay sos-scope" @click.self="close">
      <aside
        :id="panelId"
        ref="panel"
        class="collection-filter-drawer"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="`${panelId}-title`"
        tabindex="-1"
      >
        <header class="collection-filter-drawer__header">
          <div>
            <p class="sos-eyebrow">TOOLS</p>
            <h2 :id="`${panelId}-title`">{{ title }}</h2>
          </div>
          <button
            type="button"
            class="sos-button sos-button--ghost sos-icon-button sos-button--sm"
            aria-label="关闭搜索与筛选"
            @click="close"
          >
            ×
          </button>
        </header>
        <div class="collection-filter-drawer__body">
          <slot />
        </div>
      </aside>
    </div>
  </Teleport>
</template>
