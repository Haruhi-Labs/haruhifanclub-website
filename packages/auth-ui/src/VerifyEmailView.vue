<script setup>
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  home: { type: String, default: '/' },
})

const session = useSession(props.apiBase)
const route = useRoute()

const state = ref('loading') // loading | ok | error
const error = ref('')

onMounted(async () => {
  const token = String((route.query && route.query.token) || '')
  if (!token) {
    state.value = 'error'
    error.value = '链接无效：缺少令牌。'
    return
  }
  try {
    await session.verifyEmail(token)
    await session.refresh()
    state.value = 'ok'
  } catch (e) {
    state.value = 'error'
    error.value = e?.message || '验证失败，链接可能已过期。'
  }
})
</script>

<template>
  <div class="hauth-root hauth-page">
    <div class="hauth-card">
      <h2 class="hauth-title">邮箱验证</h2>
      <div v-if="state === 'loading'" class="hauth-spin">正在验证…</div>
      <template v-else-if="state === 'ok'">
        <div class="hauth-msg hauth-msg--ok">邮箱验证成功！现在你可以发布内容了。</div>
        <a class="hauth-btn" :href="home" style="display:block;text-align:center;text-decoration:none">返回首页</a>
      </template>
      <template v-else>
        <div class="hauth-msg hauth-msg--err">{{ error }}</div>
        <a class="hauth-btn hauth-btn--ghost" :href="home" style="display:block;text-align:center;text-decoration:none">返回首页</a>
      </template>
    </div>
  </div>
</template>
