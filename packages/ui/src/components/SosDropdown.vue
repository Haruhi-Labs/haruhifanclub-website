<script setup lang="ts">
import { ref, onBeforeUnmount } from 'vue'

withDefaults(
  defineProps<{
    align?: 'start' | 'end'
  }>(),
  {
    align: 'start',
  }
)

const open = ref(false)
const root = ref<HTMLElement | null>(null)

function onDocClick(event: MouseEvent) {
  if (root.value && !root.value.contains(event.target as Node)) {
    open.value = false
  }
}

function toggle() {
  open.value = !open.value
  if (typeof document === 'undefined') return
  if (open.value) {
    document.addEventListener('click', onDocClick)
  } else {
    document.removeEventListener('click', onDocClick)
  }
}

function closeMenu() {
  open.value = false
  if (typeof document !== 'undefined') document.removeEventListener('click', onDocClick)
}

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') document.removeEventListener('click', onDocClick)
})
</script>

<template>
  <div ref="root" style="position: relative; display: inline-flex">
    <span @click="toggle">
      <slot name="trigger" :open="open" />
    </span>
    <div
      v-if="open"
      class="sos-menu"
      role="menu"
      :style="{
        position: 'absolute',
        top: 'calc(100% + 0.4rem)',
        [align === 'end' ? 'right' : 'left']: '0',
      }"
      @click="closeMenu"
    >
      <slot />
    </div>
  </div>
</template>
