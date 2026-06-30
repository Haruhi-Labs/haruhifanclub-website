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

// 抽屉内/页头内点击导航后自动收起：覆盖导航链接（<a>）与品牌区（logo，可能是 <a> 或 <button>）。
// 不依赖 vue-router（SosAppbar 也用于无路由的设计系统展示页），统一在 bar 层用事件委托处理。
// 汉堡按钮本身不在品牌区、也非 <a>，不会被误关；其展开/收起由自身 toggle 负责。
function onBarClick(e: MouseEvent) {
  const t = e.target as HTMLElement | null
  if (!t) return
  if (t.closest('a') || t.closest('.sos-appbar__brand')) close()
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
    <!-- 事件委托：点击品牌区或任意导航链接后自动收起抽屉，避免 open/锁滚带到新页面 -->
    <div class="sos-appbar__inner" @click="onBarClick">
      <div class="sos-appbar__brand">
        <slot name="brand" :close="close" />
      </div>
      <!-- 桌面内联、移动端整体变为右侧抽屉 -->
      <div class="sos-appbar__cluster">
        <nav class="sos-appbar__nav">
          <slot :close="close" />
        </nav>
        <div class="sos-appbar__actions">
          <slot name="actions" :close="close" />
        </div>
      </div>
      <!-- 移动端专属：汉堡左侧、抽屉外的快捷位（如头像直达个人中心）。桌面端 CSS 隐藏。 -->
      <div class="sos-appbar__mobile-lead">
        <slot name="mobile-lead" :close="close" />
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
