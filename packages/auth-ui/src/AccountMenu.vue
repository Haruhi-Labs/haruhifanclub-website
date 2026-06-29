<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { SosAvatar } from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  profilePath: { type: String, default: '/account' },
  settingsPath: { type: String, default: '/account/settings' },
  home: { type: String, default: '/' },
})

const session = useSession(props.apiBase)
const router = useRouter()

const open = ref(false)
const user = computed(() => session.state.user)
const accountLabel = computed(
  () => user.value?.nickname || user.value?.email || user.value?.username || '',
)
// 资料编辑子页（控制台 /account/profile）
const profileEditPath = computed(() => `${props.profilePath.replace(/\/$/, '')}/profile`)

onMounted(() => {
  session.ensureReady()
})

// 悬浮展开 / 移出延迟收起（延迟避开 trigger→panel 的间隙抖动）
let closeTimer = null
function openMenu() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
  open.value = true
}
function scheduleClose() {
  closeTimer = setTimeout(() => {
    open.value = false
  }, 160)
}
onBeforeUnmount(() => closeTimer && clearTimeout(closeTimer))

// 键盘可达性收口：焦点移出整个菜单（Tab 到组件外）时收起，避免面板残留（Esc 见模板）
const rootEl = ref(null)
function onFocusOut(e) {
  if (!rootEl.value?.contains(e.relatedTarget)) open.value = false
}

async function goLogin() {
  await session.ensureReady()
  if (session.state.user) {
    router.push(props.profilePath)
    return
  }
  router.push(props.loginPath)
}
// 点击头像直接进个人中心（无需二次操作）
function goAccount() {
  open.value = false
  router.push(props.profilePath)
}
function go(path) {
  open.value = false
  router.push(path)
}
async function logout() {
  open.value = false
  await session.logout()
  router.push(props.home)
}
</script>

<template>
  <div
    ref="rootEl"
    class="hauth-menu"
    @mouseenter="openMenu"
    @mouseleave="scheduleClose"
    @focusout="onFocusOut"
    @keydown.esc="open = false"
  >
    <!-- 未登录 -->
    <button
      v-if="!user"
      type="button"
      class="sos-button sos-button--secondary sos-button--sm"
      @click="goLogin"
    >
      登录 / 注册
    </button>

    <!-- 已登录：点头像直达个人中心；悬浮展开快捷菜单 -->
    <template v-else>
      <button
        type="button"
        class="hauth-trigger"
        aria-haspopup="menu"
        :aria-expanded="open"
        title="进入个人中心"
        @click="goAccount"
        @focus="openMenu"
      >
        <SosAvatar :src="user.avatar || undefined" :name="accountLabel || 'U'" size="sm" />
        <span v-if="accountLabel" class="hauth-trigger__name">{{ accountLabel }}</span>
      </button>

      <div v-if="open" class="hauth-menu__panel sos-menu sos-scope" role="menu">
        <div class="hauth-menu-head">
          <span class="hauth-menu-head__name">{{ user.nickname || '未命名' }}</span>
          <span class="hauth-menu-head__mail">{{ user.email || user.username }}</span>
        </div>
        <button type="button" class="sos-menu__item" role="menuitem" @click="goAccount">
          个人中心
        </button>
        <button type="button" class="sos-menu__item" role="menuitem" @click="go(profileEditPath)">
          资料编辑
        </button>
        <button type="button" class="sos-menu__item" role="menuitem" @click="go(settingsPath)">
          设置
        </button>
        <div class="sos-menu__sep"></div>
        <button
          type="button"
          class="sos-menu__item sos-menu__item--danger"
          role="menuitem"
          @click="logout"
        >
          退出登录
        </button>
      </div>
    </template>
  </div>
</template>
