<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'

withDefaults(
  defineProps<{
    as?: string
    /** 汉堡按钮的无障碍标签 */
    menuLabel?: string
  }>(),
  { menuLabel: '菜单' },
)

// 移动端右侧抽屉开合（桌面端汉堡隐藏，open 恒为 false）
const open = ref(false)
function toggle() {
  open.value = !open.value
}
function close() {
  open.value = false
}

// 抽屉内点击导航链接后自动收起（导航即关）
function onClusterClick(e: MouseEvent) {
  const t = e.target as HTMLElement | null
  if (t && t.closest('a')) close()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

// 打开时锁定页面滚动 + 监听 Esc
watch(open, (v) => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = v ? 'hidden' : ''
  if (v) document.addEventListener('keydown', onKeydown)
  else document.removeEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = ''
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <component :is="as || 'header'" class="sos-appbar" :class="{ 'is-open': open }">
    <div class="sos-appbar__inner">
      <div class="sos-appbar__brand">
        <slot name="brand" />
      </div>
      <!-- 桌面内联、移动端整体变为右侧抽屉 -->
      <div class="sos-appbar__cluster" @click="onClusterClick">
        <nav class="sos-appbar__nav">
          <slot />
        </nav>
        <div class="sos-appbar__actions">
          <slot name="actions" />
        </div>
      </div>
      <button
        type="button"
        class="sos-appbar__burger"
        :aria-expanded="open"
        :aria-label="menuLabel"
        @click="toggle"
      >
        <span class="sos-appbar__burger-bars" />
      </button>
    </div>
    <div class="sos-appbar__scrim" @click="close" />
  </component>
</template>
