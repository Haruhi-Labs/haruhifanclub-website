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
const displayName = computed(() => {
  const currentUser = user.value
  return currentUser?.nickname || currentUser?.displayName || currentUser?.username || currentUser?.email || '未命名'
})
const secondaryLabel = computed(() => {
  const currentUser = user.value
  if (!currentUser) return ''
  if (currentUser.email) return currentUser.email
  return currentUser.isSuperAdmin ? '超级管理员' : '画廊成员'
})
const roleLabel = computed(() => {
  const currentUser = user.value
  if (!currentUser) return ''
  if (currentUser.isSuperAdmin) return '超管'
  return currentUser.emailVerified === false ? '待验证' : '成员'
})
const initial = computed(() => displayName.value.slice(0, 1).toUpperCase())

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
  <div class="hauth-root hauth-menu" ref="rootEl">
    <!-- 未登录 -->
    <button v-if="!user" class="hauth-btn hauth-btn--sm" @click="goLogin">登录 / 注册</button>

    <!-- 已登录 -->
    <template v-else>
      <button class="hauth-trigger" @click="open = !open" aria-haspopup="true" :aria-expanded="open">
        <span class="hauth-avatar-wrap">
          <img v-if="user.avatar" :src="user.avatar" class="hauth-avatar" alt="" />
          <span v-else class="hauth-avatar">{{ initial }}</span>
          <span class="hauth-presence" aria-hidden="true"></span>
        </span>
        <span class="hauth-trigger-main">
          <span class="hauth-trigger-name">{{ displayName }}</span>
          <span class="hauth-trigger-meta">{{ roleLabel }}</span>
        </span>
        <span class="hauth-trigger-caret" aria-hidden="true"></span>
      </button>
      <div v-if="open" class="hauth-dropdown">
        <div class="hauth-dropdown-head">
          <div class="hauth-dropdown-profile">
            <span class="hauth-avatar-wrap">
              <img v-if="user.avatar" :src="user.avatar" class="hauth-avatar" alt="" />
              <span v-else class="hauth-avatar">{{ initial }}</span>
            </span>
            <div class="hauth-dropdown-copy">
              <div class="hauth-dropdown-name">{{ displayName }}</div>
              <div class="hauth-dropdown-mail">{{ secondaryLabel }}</div>
            </div>
          </div>
          <span v-if="user.isSuperAdmin" class="hauth-badge hauth-badge--ok">超级管理员</span>
          <span v-if="user.emailVerified === false" class="hauth-badge hauth-badge--warn" style="margin-top:6px">邮箱未验证</span>
        </div>
        <router-link class="hauth-item" :to="profilePath" @click="open = false">个人资料</router-link>
        <router-link class="hauth-item" :to="settingsPath" @click="open = false">账号设置</router-link>
        <button class="hauth-item hauth-item--danger" @click="logout">退出登录</button>
      </div>
    </template>
  </div>
</template>
