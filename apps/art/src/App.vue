<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import TopBar from './components/TopBar.vue'
import SiteFooter from './components/SiteFooter.vue'
import ArtSpatialField from './components/ArtSpatialField.vue'

const LIGHTS_OUT_KEY = 'haruhi-art-lights-out'
const lightsOut = ref(false)
const route = useRoute()
const isHomeRoute = computed(() => route.path === '/')
const atmosphereVariant = computed(() => {
  if (['gallery', 'gallery-search', 'artwork-detail', 'announcements', 'adventurer-profile'].includes(route.name)) {
    return 'gallery'
  }
  if (['upload', 'exchange', 'terminal'].includes(route.name)) return 'upload'
  return ''
})
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
  <div class="bg-layer gallery-bg" :class="{ 'is-spatial': atmosphereVariant }"></div>
  <div class="bg-layer gallery-mask" :class="{ 'is-spatial': atmosphereVariant }"></div>
  <ArtSpatialField v-if="atmosphereVariant" :variant="atmosphereVariant" />

  <div
    class="app-shell sos-scope"
    :class="{ 'is-home-route': isHomeRoute, 'has-art-space': atmosphereVariant }"
    data-sos-site="art"
  >
    <TopBar />

    <main class="main" :class="{ 'is-home-route': isHomeRoute }">
      <!-- key 用 route.path（不含 query）：画廊首页与搜索页都靠 query 驱动弹窗或结果分页，
           query 变化不能重挂整个视图。注释须放在 KeepAlive 外，否则 KeepAlive 会被判定有多个子节点。 -->
      <router-view v-slot="{ Component, route: viewRoute }">
        <KeepAlive include="HomeView">
          <component :is="Component" :key="viewRoute.name === 'home' ? 'home' : viewRoute.path" />
        </KeepAlive>
      </router-view>
    </main>

    <SiteFooter />
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
.app-shell {
  display: flex;
  flex-direction: column;
  min-height: 100dvh;
}

.app-shell.has-art-space {
  position: relative;
  z-index: 1;
  background: transparent;
}

.gallery-bg.is-spatial {
  top: -22vh;
  bottom: -22vh;
  background-size: cover;
  background-position: center;
  will-change: transform;
  animation: art-spatial-backdrop-drift 34s cubic-bezier(0.42, 0, 0.3, 1) infinite alternate;
}

.gallery-mask.is-spatial {
  top: -20vh;
  bottom: -20vh;
  background-image:
    radial-gradient(circle at 18% 22%, rgba(224, 255, 248, 0.14), transparent 36%),
    radial-gradient(circle at 82% 74%, rgba(255, 226, 211, 0.1), transparent 40%);
  background-size: 140% 135%, 132% 142%;
  will-change: transform;
  animation: art-spatial-light-drift 29s ease-in-out infinite alternate;
}

@keyframes art-spatial-backdrop-drift {
  0% {
    transform: translate3d(calc(-1.2% + var(--art-backdrop-x, 0px)), -0.7%, 0) rotate(var(--art-backdrop-tilt, 0deg)) scale(1.08);
  }
  45% {
    transform: translate3d(calc(0.8% + var(--art-backdrop-x, 0px)), 0.9%, 0) rotate(var(--art-backdrop-tilt, 0deg)) scale(1.12);
  }
  100% {
    transform: translate3d(calc(1.4% + var(--art-backdrop-x, 0px)), -0.4%, 0) rotate(var(--art-backdrop-tilt, 0deg)) scale(1.09);
  }
}

@keyframes art-spatial-light-drift {
  0% { transform: translate3d(calc(-1.4% + var(--art-mask-x, 0px)), -0.8%, 0) scale(1.03); }
  50% { transform: translate3d(calc(1% + var(--art-mask-x, 0px)), 1.2%, 0) scale(1.07); }
  100% { transform: translate3d(calc(1.8% + var(--art-mask-x, 0px)), -0.4%, 0) scale(1.04); }
}

@media (prefers-reduced-motion: reduce) {
  .gallery-bg.is-spatial,
  .gallery-mask.is-spatial {
    animation: none;
  }
}

.app-shell > .main {
  flex: 1;
  /* SosAppbar 为流内 sticky、自占布局高度，无需为 fixed 头预留，仅留呼吸间距 */
  padding-top: var(--sos-space-6);
}

html.art-home-route .app-shell > .main {
  padding-top: 0;
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
  .app-shell > .main {
    padding-top: var(--sos-space-4);
  }

  html.art-home-route .app-shell > .main {
    padding-top: 0;
  }

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

.page-enter-active,
.page-leave-active {
  transition:
    opacity 0.18s cubic-bezier(0.2, 0.8, 0.2, 1),
    transform 0.18s cubic-bezier(0.2, 0.8, 0.2, 1);
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px) scale(0.995);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.995);
}
</style>
