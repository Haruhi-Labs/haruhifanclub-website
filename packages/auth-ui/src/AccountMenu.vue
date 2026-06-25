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
  () => user.value?.nickname || user.value?.email || user.value?.username || ''
)

onMounted(() => {
  if (!session.state.ready) session.refresh()
  document.addEventListener('click', onDocClick)
})
onBeforeUnmount(() => document.removeEventListener('click', onDocClick))

const rootEl = ref(null)
function onDocClick(e) {
  if (rootEl.value && !rootEl.value.contains(e.target)) open.value = false
}

function goLogin() {
  router.push(props.loginPath)
}
async function logout() {
  open.value = false
  await session.logout()
  router.push(props.home)
}
</script>

<template>
  <div ref="rootEl" class="hauth-menu">
    <!-- 未登录 -->
    <button
      v-if="!user"
      type="button"
      class="sos-button sos-button--secondary sos-button--sm"
      @click="goLogin"
    >
      登录 / 注册
    </button>

    <!-- 已登录 -->
    <template v-else>
      <button
        type="button"
        class="hauth-trigger"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="open = !open"
      >
        <SosAvatar :src="user.avatar || undefined" :name="accountLabel || 'U'" size="sm" />
        <span v-if="accountLabel" class="hauth-trigger__name">{{ accountLabel }}</span>
      </button>

      <div v-if="open" class="hauth-menu__panel sos-menu sos-scope" role="menu">
        <div class="hauth-menu-head">
          <span class="hauth-menu-head__name">{{ user.nickname || '未命名' }}</span>
          <span class="hauth-menu-head__mail">{{ user.email || user.username }}</span>
        </div>
        <router-link class="sos-menu__item" :to="profilePath" @click="open = false">
          个人资料
        </router-link>
        <router-link class="sos-menu__item" :to="settingsPath" @click="open = false">
          账号设置
        </router-link>
        <div class="sos-menu__sep"></div>
        <button type="button" class="sos-menu__item sos-menu__item--danger" @click="logout">
          退出登录
        </button>
      </div>
    </template>
  </div>
</template>
