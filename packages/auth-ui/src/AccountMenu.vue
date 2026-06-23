<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
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
const initial = computed(() =>
  (user.value?.nickname || user.value?.email || 'U').slice(0, 1).toUpperCase()
)
const accountLabel = computed(() => user.value?.nickname || user.value?.email || '')

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
  <div ref="rootEl" class="hauth-root hauth-menu">
    <!-- 未登录 -->
    <button v-if="!user" class="hauth-account-action" @click="goLogin">登录 / 注册</button>

    <!-- 已登录 -->
    <template v-else>
      <button
        class="hauth-trigger"
        aria-haspopup="true"
        :aria-expanded="open"
        @click="open = !open"
      >
        <img v-if="user.avatar" :src="user.avatar" class="hauth-avatar" alt="" />
        <span v-else class="hauth-avatar">{{ initial }}</span>
        <span v-if="accountLabel" class="hauth-trigger-name">{{ accountLabel }}</span>
      </button>
      <div v-if="open" class="hauth-dropdown">
        <div class="hauth-dropdown-head">
          <div class="hauth-dropdown-name">{{ user.nickname || '未命名' }}</div>
          <div class="hauth-dropdown-mail">{{ user.email || user.username }}</div>
          <span v-if="!user.emailVerified" class="hauth-badge hauth-badge--warn"> 邮箱未验证 </span>
        </div>
        <router-link class="hauth-item" :to="profilePath" @click="open = false">
          个人资料
        </router-link>
        <router-link class="hauth-item" :to="settingsPath" @click="open = false">
          账号设置
        </router-link>
        <button class="hauth-item hauth-item--danger" @click="logout">退出登录</button>
      </div>
    </template>
  </div>
</template>
