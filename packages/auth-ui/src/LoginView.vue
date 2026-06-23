<script setup>
import { ref, reactive, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { SosCard, SosField, SosInput, SosButton, SosNotice, SosTabs } from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  // 登录/注册成功后跳转目标（未给则用 ?redirect 查询，再退回 '/'）
  home: { type: String, default: '/' },
  title: { type: String, default: '凉宫春日应援团' },
  // 可选：站点表达模式（news/shop/art/library/exam），令账号页融入所在站气质
  site: { type: String, default: undefined },
})

const session = useSession(props.apiBase)
const router = useRouter()
const route = useRoute()

const tab = ref('login') // login | register | forgot
const loading = ref(false)
const error = ref('')
const okMsg = ref('')
const showPw = ref(false)

const form = reactive({ account: '', email: '', nickname: '', password: '', confirm: '' })

const tabItems = [
  { value: 'login', label: '登录' },
  { value: 'register', label: '注册' },
]

const pwMismatch = computed(
  () => tab.value === 'register' && form.confirm.length > 0 && form.password !== form.confirm
)

function go() {
  const target = (route.query && route.query.redirect) || props.home || '/'
  router.push(String(target))
}

function switchTab(t) {
  tab.value = t
  error.value = ''
  okMsg.value = ''
}

async function onLogin() {
  error.value = ''
  loading.value = true
  try {
    await session.login(form.account.trim(), form.password)
    go()
  } catch (e) {
    error.value = e?.status === 401 ? '邮箱/用户名或密码错误' : e?.message || '登录失败'
  } finally {
    loading.value = false
  }
}

async function onRegister() {
  error.value = ''
  if (form.password.length < 8) {
    error.value = '密码至少 8 位'
    return
  }
  if (form.password !== form.confirm) {
    error.value = '两次输入的密码不一致'
    return
  }
  loading.value = true
  try {
    await session.register({
      email: form.email.trim(),
      password: form.password,
      nickname: form.nickname.trim() || undefined,
    })
    okMsg.value = '注册成功，已自动登录。验证邮件已发送，请尽快验证邮箱后再发布内容。'
    setTimeout(go, 1200)
  } catch (e) {
    error.value = e?.status === 409 ? '该邮箱已注册' : e?.message || '注册失败'
  } finally {
    loading.value = false
  }
}

async function onForgot() {
  error.value = ''
  loading.value = true
  try {
    await session.forgotPassword(form.email.trim())
    okMsg.value = '若该邮箱已注册，重置链接已发送，请查收邮件（1 小时内有效）。'
  } catch (e) {
    error.value = e?.message || '发送失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="hauth-root sos-scope hauth-shell" :data-sos-site="site">
    <SosCard class="hauth-card" as="section">
      <header class="hauth-brand">
        <span class="hauth-brand__mark" aria-hidden="true">{{ title.slice(0, 1) }}</span>
        <h1 class="sos-title" style="font-size: var(--sos-text-2xl)">{{ title }}</h1>
        <p class="sos-copy sos-copy--small">
          {{
            tab === 'login'
              ? '登录你的账号'
              : tab === 'register'
                ? '创建一个新账号'
                : '找回你的密码'
          }}
        </p>
      </header>

      <div v-if="tab !== 'forgot'" style="display: flex; justify-content: center">
        <SosTabs :model-value="tab" :items="tabItems" @update:model-value="switchTab" />
      </div>

      <SosNotice v-if="error" tone="danger" style="margin-top: var(--sos-space-4)">
        {{ error }}
      </SosNotice>
      <SosNotice v-if="okMsg" tone="success" style="margin-top: var(--sos-space-4)">
        {{ okMsg }}
      </SosNotice>

      <!-- 登录 -->
      <form
        v-if="tab === 'login'"
        class="sos-stack"
        style="margin-top: var(--sos-space-5)"
        @submit.prevent="onLogin"
      >
        <SosField label="邮箱或用户名">
          <SosInput v-model="form.account" autocomplete="username" required />
        </SosField>
        <SosField label="密码">
          <div class="hauth-pw">
            <SosInput
              v-model="form.password"
              :type="showPw ? 'text' : 'password'"
              autocomplete="current-password"
              required
            />
            <button type="button" class="hauth-pw__toggle" @click="showPw = !showPw">
              {{ showPw ? '隐藏' : '显示' }}
            </button>
          </div>
        </SosField>
        <SosButton type="submit" class="sos-button--block" :loading="loading">登录</SosButton>
        <div style="text-align: center">
          <SosButton variant="link" type="button" @click="switchTab('forgot')">
            忘记密码？
          </SosButton>
        </div>
      </form>

      <!-- 注册 -->
      <form
        v-else-if="tab === 'register'"
        class="sos-stack"
        style="margin-top: var(--sos-space-5)"
        @submit.prevent="onRegister"
      >
        <SosField label="邮箱">
          <SosInput v-model="form.email" type="email" autocomplete="email" required />
        </SosField>
        <SosField label="昵称" help="留空则用邮箱前缀">
          <SosInput v-model="form.nickname" maxlength="32" placeholder="你希望别人怎么称呼你" />
        </SosField>
        <SosField label="密码" help="至少 8 位">
          <div class="hauth-pw">
            <SosInput
              v-model="form.password"
              :type="showPw ? 'text' : 'password'"
              autocomplete="new-password"
              required
            />
            <button type="button" class="hauth-pw__toggle" @click="showPw = !showPw">
              {{ showPw ? '隐藏' : '显示' }}
            </button>
          </div>
        </SosField>
        <SosField label="确认密码" :error="pwMismatch ? '两次输入的密码不一致' : undefined">
          <SosInput
            v-model="form.confirm"
            :type="showPw ? 'text' : 'password'"
            autocomplete="new-password"
            required
          />
        </SosField>
        <SosButton type="submit" class="sos-button--block" :loading="loading">
          注册并登录
        </SosButton>
      </form>

      <!-- 找回密码 -->
      <form
        v-else
        class="sos-stack"
        style="margin-top: var(--sos-space-5)"
        @submit.prevent="onForgot"
      >
        <SosField label="注册邮箱" help="我们会向该邮箱发送重置链接">
          <SosInput v-model="form.email" type="email" autocomplete="email" required />
        </SosField>
        <SosButton type="submit" class="sos-button--block" :loading="loading">
          发送重置链接
        </SosButton>
        <div style="text-align: center">
          <SosButton variant="link" type="button" @click="switchTab('login')">返回登录</SosButton>
        </div>
      </form>
    </SosCard>
  </div>
</template>
