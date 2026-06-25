<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { SosAvatar } from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  profilePath: { type: String, default: '/account' },
  // settingsPath / home 保留以兼容各 app 既有传参（个人中心内已自带设置与退出，不再用下拉）
  settingsPath: { type: String, default: '/account/settings' },
  home: { type: String, default: '/' },
})

const session = useSession(props.apiBase)
const router = useRouter()

const user = computed(() => session.state.user)
const accountLabel = computed(
  () => user.value?.nickname || user.value?.email || user.value?.username || '',
)

onMounted(() => {
  if (!session.state.ready) session.refresh()
})

function goLogin() {
  router.push(props.loginPath)
}
// 个人中心已是完整工作台，头像直接进概览，不再展开下拉
function goAccount() {
  router.push(props.profilePath)
}
</script>

<template>
  <div class="hauth-menu">
    <!-- 未登录 -->
    <button
      v-if="!user"
      type="button"
      class="sos-button sos-button--secondary sos-button--sm"
      @click="goLogin"
    >
      登录 / 注册
    </button>

    <!-- 已登录：点头像资料卡直接进个人中心 -->
    <button
      v-else
      type="button"
      class="hauth-trigger"
      title="进入个人中心"
      @click="goAccount"
    >
      <SosAvatar :src="user.avatar || undefined" :name="accountLabel || 'U'" size="sm" />
      <span v-if="accountLabel" class="hauth-trigger__name">{{ accountLabel }}</span>
    </button>
  </div>
</template>
