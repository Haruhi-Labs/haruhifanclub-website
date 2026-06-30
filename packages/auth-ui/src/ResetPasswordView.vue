<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { SosCard, SosField, SosInput, SosButton, SosNotice } from '@haruhi/ui'
import { useSession } from './useSession.js'
import brandLogo from './assets/haruhi-logo-192.png'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  title: { type: String, default: '凉宫春日应援团' },
  site: { type: String, default: undefined },
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
const showPw = ref(false)

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
  <div class="hauth-root sos-scope hauth-shell" :data-sos-site="site">
    <SosCard class="hauth-card" as="section">
      <header class="hauth-brand">
        <img class="hauth-brand__mark" :src="brandLogo" alt="" />
        <h1 class="sos-title" style="font-size: var(--sos-text-2xl)">重置密码</h1>
        <p class="sos-copy sos-copy--small">设置一个新密码，旧的登录会话将全部失效。</p>
      </header>

      <SosNotice
        v-if="!token"
        tone="danger"
        title="链接无效"
        style="margin-top: var(--sos-space-4)"
      >
        缺少令牌，请重新发起“忘记密码”。
      </SosNotice>
      <SosNotice
        v-else-if="done"
        tone="success"
        title="密码已重置"
        style="margin-top: var(--sos-space-4)"
      >
        正在跳转到登录…
      </SosNotice>

      <form
        v-if="token && !done"
        class="sos-stack"
        style="margin-top: var(--sos-space-5)"
        @submit.prevent="onSubmit"
      >
        <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
        <SosField label="新密码" help="至少 8 位">
          <div class="hauth-pw">
            <SosInput
              v-model="password"
              :type="showPw ? 'text' : 'password'"
              autocomplete="new-password"
              required
            />
            <button type="button" class="hauth-pw__toggle" @click="showPw = !showPw">
              {{ showPw ? '隐藏' : '显示' }}
            </button>
          </div>
        </SosField>
        <SosField label="确认新密码">
          <SosInput v-model="confirm" type="password" autocomplete="new-password" required />
        </SosField>
        <SosButton type="submit" class="sos-button--block" :loading="loading">重置密码</SosButton>
      </form>
    </SosCard>
  </div>
</template>
