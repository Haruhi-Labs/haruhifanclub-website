<script setup>
// 移动端头像快捷入口：点头像直达个人中心（无二次选择）。
// 仅渲染头像本身，显隐与定位由放置它的容器负责（如 SosAppbar 的 mobile-lead 槽位仅在窄屏显示）。
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { SosAvatar } from '@haruhi/ui'
import { useSession } from './useSession.js'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  profilePath: { type: String, default: '/account' },
})

const session = useSession(props.apiBase)
const router = useRouter()

const user = computed(() => session.state.user)
const accountLabel = computed(
  () => user.value?.nickname || user.value?.email || user.value?.username || ''
)

onMounted(() => {
  session.ensureReady()
})

function goAccount() {
  router.push(props.profilePath)
}
</script>

<template>
  <!-- 未登录不显示（登录入口仍在抽屉内的 AccountMenu） -->
  <button
    v-if="user"
    type="button"
    class="hauth-avatar-link"
    title="进入个人中心"
    aria-label="进入个人中心"
    @click="goAccount"
  >
    <SosAvatar :src="user.avatar || undefined" :name="accountLabel || 'U'" size="sm" />
  </button>
</template>

<style scoped>
.hauth-avatar-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  background: none;
  cursor: pointer;
  line-height: 0;
  border-radius: 999px;
}
.hauth-avatar-link:focus-visible {
  outline: none;
  box-shadow: var(--sos-ring);
}
</style>
