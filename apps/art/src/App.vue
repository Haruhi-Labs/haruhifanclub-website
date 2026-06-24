<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import TopBar from './components/TopBar.vue'

const LIGHTS_OUT_KEY = 'haruhi-art-lights-out'
const lightsOut = ref(false)
const route = useRoute()
const isHomeRoute = computed(() => route.path === '/')
let lightsApplyFrame = 0
let homeRouteApplyFrame = 0
let cancelLightsPersist = null
let appliedGlobalLightsOut = false
let appliedHomeLightsOut = false

function emitHomeLightsSwitch(phase, value) {
  if (typeof window === 'undefined' || !isHomeRoute.value) return
  window.dispatchEvent(new CustomEvent('art-home-lights-switch', {
    detail: { phase, value },
  }))
}

function setLightsOutClass(value) {
  if (typeof document === 'undefined') return
  const useHomeTheme = isHomeRoute.value
  const nextGlobalLightsOut = value && !useHomeTheme
  const nextHomeLightsOut = value && useHomeTheme

  if (appliedGlobalLightsOut !== nextGlobalLightsOut) {
    document.documentElement.classList.toggle('art-lights-out', nextGlobalLightsOut)
    appliedGlobalLightsOut = nextGlobalLightsOut
  }

  if (appliedHomeLightsOut !== nextHomeLightsOut) {
    document.documentElement.classList.toggle('art-home-lights-out', nextHomeLightsOut)
    document.body?.classList.toggle('art-home-lights-out', nextHomeLightsOut)
    appliedHomeLightsOut = nextHomeLightsOut
  }
}

function applyLightsOut(value, { deferHome = false } = {}) {
  if (typeof window === 'undefined' || !deferHome || !isHomeRoute.value) {
    setLightsOutClass(value)
    return
  }

  emitHomeLightsSwitch('start', value)
  if (lightsApplyFrame) window.cancelAnimationFrame(lightsApplyFrame)
  lightsApplyFrame = window.requestAnimationFrame(() => {
    lightsApplyFrame = 0
    setLightsOutClass(value)
    window.requestAnimationFrame(() => {
      emitHomeLightsSwitch('end', value)
    })
  })
}

function setHomeRouteClass(value) {
  if (typeof document === 'undefined') return
  document.documentElement.classList.toggle('art-home-route', value)
  document.body?.classList.toggle('art-home-route', value)
}

function applyHomeRoute(value) {
  if (typeof document === 'undefined') return

  if (homeRouteApplyFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(homeRouteApplyFrame)
    homeRouteApplyFrame = 0
  }

  if (!value && typeof window !== 'undefined') {
    homeRouteApplyFrame = window.requestAnimationFrame(() => {
      homeRouteApplyFrame = 0
      setHomeRouteClass(false)
      applyLightsOut(lightsOut.value)
    })
    return
  }

  setHomeRouteClass(true)
  applyLightsOut(lightsOut.value)
}

function persistLightsOut(value) {
  if (typeof window === 'undefined') return

  cancelLightsPersist?.()
  const write = () => {
    cancelLightsPersist = null
    window.localStorage.setItem(LIGHTS_OUT_KEY, value ? '1' : '0')
  }

  if ('requestIdleCallback' in window) {
    const id = window.requestIdleCallback(write, { timeout: 800 })
    cancelLightsPersist = () => window.cancelIdleCallback?.(id)
  } else {
    const id = window.setTimeout(write, 0)
    cancelLightsPersist = () => window.clearTimeout(id)
  }
}

function toggleLightsOut() {
  lightsOut.value = !lightsOut.value
}

onMounted(() => {
  lightsOut.value = window.localStorage.getItem(LIGHTS_OUT_KEY) === '1'
  applyLightsOut(lightsOut.value)
})

watch(lightsOut, (value) => {
  applyLightsOut(value, { deferHome: true })
  persistLightsOut(value)
})

watch(isHomeRoute, (value) => {
  applyHomeRoute(value)
}, { immediate: true, flush: 'post' })

onBeforeUnmount(() => {
  if (lightsApplyFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(lightsApplyFrame)
  }
  if (homeRouteApplyFrame && typeof window !== 'undefined') {
    window.cancelAnimationFrame(homeRouteApplyFrame)
  }
  lightsApplyFrame = 0
  homeRouteApplyFrame = 0
  cancelLightsPersist?.()
  cancelLightsPersist = null
  applyLightsOut(false)
  applyHomeRoute(false)
  document.documentElement.classList.remove('art-home-lights-out')
  document.body?.classList.remove('art-home-lights-out')
  appliedGlobalLightsOut = false
  appliedHomeLightsOut = false
})
</script>

<template>
  <div class="bg-layer gallery-bg"></div>
  <div class="bg-layer gallery-mask"></div>
  
  <div class="app-shell" :class="{ 'is-home-route': isHomeRoute }">
    <header class="topbar" :class="{ 'is-home-route': isHomeRoute }">
      <TopBar />
    </header>

    <main class="main" :class="{ 'is-home-route': isHomeRoute }">
      <router-view v-slot="{ Component, route }">
        <KeepAlive include="HomeView">
          <component :is="Component" :key="route.name === 'home' ? 'home' : route.fullPath" />
        </KeepAlive>
      </router-view>
    </main>
  </div>

  <aside class="secret-toolbar" :class="{ 'is-active': lightsOut }" aria-label="特殊功能工具栏">
    <button
      class="lights-toggle"
      type="button"
      :aria-pressed="lightsOut ? 'true' : 'false'"
      @click="toggleLightsOut"
    >
      <span class="lights-toggle__icon" aria-hidden="true"></span>
      <span class="lights-toggle__text">{{ lightsOut ? '开灯' : '关灯' }}</span>
    </button>
  </aside>
</template>

<style>
/* =========================================
   全局页面布局约束
   ========================================= */
.app-shell {
  /* ⚠️ 重要：因为导航栏是 fixed 定位，必须给内容区一个顶部内边距。
     数值 = 导航栏高度 (约72px) + 间距 (24px) = 96px */
  padding-top: 96px;
  /* Ensure the shell takes full height so short pages don't abruptly end */
  min-height: 100dvh;
}

.secret-toolbar {
  position: fixed;
  right: 18px;
  bottom: 20px;
  z-index: 850;
  display: flex;
  align-items: center;
  opacity: 0.38;
  transform: translateX(34px);
  transition:
    opacity 0.22s ease,
    transform 0.22s ease,
    filter 0.22s ease;
  filter: saturate(0.78);
}

.secret-toolbar:hover,
.secret-toolbar:focus-within,
.secret-toolbar.is-active {
  opacity: 1;
  transform: translateX(0);
  filter: saturate(1);
}

.lights-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 38px;
  padding: 8px 12px 8px 10px;
  border: 1px solid rgba(255, 255, 255, 0.42);
  border-radius: 999px 0 0 999px;
  background:
    linear-gradient(135deg, rgba(18, 25, 45, 0.82), rgba(53, 37, 75, 0.72)),
    rgba(10, 14, 28, 0.78);
  color: rgba(255, 255, 255, 0.9);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.22), inset 0 1px 0 rgba(255, 255, 255, 0.18);
  cursor: pointer;
  user-select: none;
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  transition:
    background 0.22s ease,
    border-color 0.22s ease,
    color 0.22s ease,
    box-shadow 0.22s ease;
}

.lights-toggle:hover {
  border-color: rgba(255, 225, 150, 0.68);
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.28), 0 0 18px rgba(255, 209, 102, 0.18);
}

.lights-toggle__icon {
  position: relative;
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: #ffd166;
  box-shadow: 0 0 12px rgba(255, 209, 102, 0.66);
  transition:
    background 0.22s ease,
    box-shadow 0.22s ease,
    transform 0.22s ease;
}

.lights-toggle__icon::after {
  content: "";
  position: absolute;
  top: -2px;
  right: -2px;
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: rgba(18, 25, 45, 0);
  transition: background 0.22s ease, transform 0.22s ease;
}

.secret-toolbar.is-active .lights-toggle {
  border-color: rgba(126, 200, 255, 0.45);
  background:
    linear-gradient(135deg, rgba(5, 12, 28, 0.9), rgba(28, 31, 62, 0.82)),
    rgba(4, 8, 18, 0.84);
}

.secret-toolbar.is-active .lights-toggle__icon {
  background: #b9d8ff;
  box-shadow: 0 0 16px rgba(126, 200, 255, 0.5);
  transform: rotate(-14deg);
}

.secret-toolbar.is-active .lights-toggle__icon::after {
  background: rgba(5, 12, 28, 0.94);
  transform: translateX(1px);
}

.lights-toggle__text {
  font-size: 13px;
  font-weight: 900;
  letter-spacing: 0;
  white-space: nowrap;
}

@media (max-width: 768px) {
  .secret-toolbar {
    right: 10px;
    bottom: 12px;
    transform: translateX(42px);
  }

  .lights-toggle {
    min-height: 36px;
    padding: 8px 10px;
  }
}

/* =========================================
   全局页面切换动画：磨砂浮动 + 缩放效果
   ========================================= */

/* 1. 进场和离场的激活状态 */
.page-enter-active,
.page-leave-active {
  /* 使用贝塞尔曲线模拟物理惯性，比 linear 更自然 */
  transition: 
    opacity 0.18s cubic-bezier(0.2, 0.8, 0.2, 1),
    transform 0.18s cubic-bezier(0.2, 0.8, 0.2, 1);
}

/* 2. 进场开始状态 (页面刚要出来时) */
.page-enter-from {
  opacity: 0;
  /* 稍微向下偏移 15px，有一种浮上来的感觉 */
  transform: translateY(8px) scale(0.995);
}

/* 3. 离场结束状态 (旧页面离开后) */
.page-leave-to {
  opacity: 0;
  /* 稍微向上偏移 -15px，有一种飘走的感觉 */
  transform: translateY(-8px) scale(0.995);
}
</style>
