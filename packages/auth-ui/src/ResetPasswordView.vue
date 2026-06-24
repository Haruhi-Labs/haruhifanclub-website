<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
})

const session = useSession(props.apiBase)
const router = useRouter()
const route = useRoute()

const token = String((route.query && route.query.token) || '')
const password = ref('')
const confirm = ref('')
const loading = ref(false)
const error = ref('')
const done = ref(false)

async function onSubmit() {
  error.value = ''
  if (password.value.length < 8) {
    error.value = '密码至少 8 位'
    return
  }
  if (password.value !== confirm.value) {
    error.value = '两次输入的密码不一致'
    return
  }
  loading.value = true
  try {
    await session.resetPassword(token, password.value)
    done.value = true
    setTimeout(() => router.push(props.loginPath), 1500)
  } catch (e) {
    error.value = e?.message || '重置失败，链接可能已过期'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="hauth-root hauth-page">
    <div class="hauth-card">
      <h2 class="hauth-title">重置密码</h2>
      <p class="hauth-sub">设置一个新密码，旧的登录会话将全部失效。</p>

      <div v-if="!token" class="hauth-msg hauth-msg--err">链接无效：缺少令牌。请重新发起“忘记密码”。</div>
      <div v-else-if="done" class="hauth-msg hauth-msg--ok">密码已重置，正在跳转到登录…</div>

      <form v-if="token && !done" @submit.prevent="onSubmit">
        <div v-if="error" class="hauth-msg hauth-msg--err">{{ error }}</div>
        <div class="hauth-field">
          <label class="hauth-label">新密码（至少 8 位）</label>
          <input class="hauth-input" type="password" v-model="password" autocomplete="new-password" required />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">确认新密码</label>
          <input class="hauth-input" type="password" v-model="confirm" autocomplete="new-password" required />
        </div>
        <button class="hauth-btn" :disabled="loading">{{ loading ? '提交中…' : '重置密码' }}</button>
      </form>
    </div>
  </div>
</template>
