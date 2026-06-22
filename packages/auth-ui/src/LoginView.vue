<script setup>
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  // 登录/注册成功后跳转目标（未给则用 ?redirect 查询，再退回 '/'）
  home: { type: String, default: '/' },
  title: { type: String, default: '春日应援团' },
})

const session = useSession(props.apiBase)
const router = useRouter()
const route = useRoute()

const tab = ref('login') // login | register | forgot
const loading = ref(false)
const error = ref('')
const okMsg = ref('')

const form = reactive({ account: '', email: '', nickname: '', password: '' })

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
  <div class="hauth-root hauth-page">
    <div class="hauth-card">
      <h2 class="hauth-title">{{ title }}</h2>
      <p class="hauth-sub">
        {{ tab === 'login' ? '登录你的账号' : tab === 'register' ? '创建新账号' : '找回密码' }}
      </p>

      <div class="hauth-tabs" v-if="tab !== 'forgot'">
        <button class="hauth-tab" :class="{ 'is-active': tab === 'login' }" @click="switchTab('login')">登录</button>
        <button class="hauth-tab" :class="{ 'is-active': tab === 'register' }" @click="switchTab('register')">注册</button>
      </div>

      <div v-if="error" class="hauth-msg hauth-msg--err">{{ error }}</div>
      <div v-if="okMsg" class="hauth-msg hauth-msg--ok">{{ okMsg }}</div>

      <!-- 登录 -->
      <form v-if="tab === 'login'" @submit.prevent="onLogin">
        <div class="hauth-field">
          <label class="hauth-label">邮箱或用户名</label>
          <input class="hauth-input" v-model="form.account" autocomplete="username" required />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">密码</label>
          <input class="hauth-input" type="password" v-model="form.password" autocomplete="current-password" required />
        </div>
        <button class="hauth-btn" :disabled="loading">{{ loading ? '登录中…' : '登录' }}</button>
        <div class="hauth-foot">
          <button type="button" class="hauth-link" @click="switchTab('forgot')">忘记密码？</button>
        </div>
      </form>

      <!-- 注册 -->
      <form v-else-if="tab === 'register'" @submit.prevent="onRegister">
        <div class="hauth-field">
          <label class="hauth-label">邮箱</label>
          <input class="hauth-input" type="email" v-model="form.email" autocomplete="email" required />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">昵称（可选）</label>
          <input class="hauth-input" v-model="form.nickname" maxlength="32" placeholder="留空则用邮箱前缀" />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">密码（至少 8 位）</label>
          <input class="hauth-input" type="password" v-model="form.password" autocomplete="new-password" required />
        </div>
        <button class="hauth-btn" :disabled="loading">{{ loading ? '注册中…' : '注册并登录' }}</button>
      </form>

      <!-- 找回密码 -->
      <form v-else @submit.prevent="onForgot">
        <div class="hauth-field">
          <label class="hauth-label">注册邮箱</label>
          <input class="hauth-input" type="email" v-model="form.email" autocomplete="email" required />
        </div>
        <button class="hauth-btn" :disabled="loading">{{ loading ? '发送中…' : '发送重置链接' }}</button>
        <div class="hauth-foot">
          <button type="button" class="hauth-link" @click="switchTab('login')">返回登录</button>
        </div>
      </form>
    </div>
  </div>
</template>
