<script setup>
import { ref, reactive, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { SosField, SosButton, SosNotice, SosTabs } from '@haruhi/ui'
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

const subtitle = computed(() =>
  tab.value === 'login'
    ? '登录你的账号'
    : tab.value === 'register'
      ? '创建一个新账号'
      : '找回你的密码'
)
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
    <div class="hauth-auth">
      <!-- 品牌面板 -->
      <aside class="hauth-aside">
        <div>
          <span class="hauth-aside__mark" aria-hidden="true">SOS</span>
          <p class="hauth-aside__name">{{ title }}</p>
          <p class="hauth-aside__tag">一个账号，畅通全团——团报、商城、美术部、书库与考场。</p>
        </div>
        <ul class="hauth-aside__list">
          <li>
            <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M3.5 8.5l3 3 6-6.5"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            同人投稿与作品展示
          </li>
          <li>
            <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M3.5 8.5l3 3 6-6.5"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            周边商城与活动报名
          </li>
          <li>
            <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M3.5 8.5l3 3 6-6.5"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            团员积分与一处管理
          </li>
        </ul>
      </aside>

      <!-- 表单面板 -->
      <div class="hauth-form">
        <div class="hauth-form__head">
          <h1 class="hauth-form__title">
            {{ tab === 'login' ? '欢迎回来' : tab === 'register' ? '加入应援团' : '找回密码' }}
          </h1>
          <p class="hauth-form__sub">{{ subtitle }}</p>
        </div>

        <div v-if="tab !== 'forgot'">
          <SosTabs :model-value="tab" :items="tabItems" @update:model-value="switchTab" />
        </div>

        <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
        <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

        <!-- 登录 -->
        <form v-if="tab === 'login'" class="sos-stack" @submit.prevent="onLogin">
          <SosField label="邮箱或用户名">
            <div class="sos-input-affix">
              <span class="sos-input-affix__icon hauth-affix-icon" aria-hidden="true">
                <svg viewBox="0 0 20 20" fill="none">
                  <circle cx="10" cy="6.5" r="3.2" stroke="currentColor" stroke-width="1.5" />
                  <path
                    d="M4 16c0-2.8 2.7-4.5 6-4.5s6 1.7 6 4.5"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                  />
                </svg>
              </span>
              <input v-model="form.account" autocomplete="username" required />
            </div>
          </SosField>
          <SosField label="密码">
            <div class="sos-input-affix">
              <span class="sos-input-affix__icon hauth-affix-icon" aria-hidden="true">
                <svg viewBox="0 0 20 20" fill="none">
                  <rect
                    x="4.5"
                    y="9"
                    width="11"
                    height="7"
                    rx="1.6"
                    stroke="currentColor"
                    stroke-width="1.5"
                  />
                  <path d="M7 9V6.8a3 3 0 016 0V9" stroke="currentColor" stroke-width="1.5" />
                </svg>
              </span>
              <input
                v-model="form.password"
                :type="showPw ? 'text' : 'password'"
                autocomplete="current-password"
                required
              />
              <button
                type="button"
                class="hauth-eye"
                :aria-label="showPw ? '隐藏密码' : '显示密码'"
                @click="showPw = !showPw"
              >
                <svg v-if="!showPw" viewBox="0 0 20 20" fill="none">
                  <path
                    d="M2 10s3-5 8-5 8 5 8 5-3 5-8 5-8-5-8-5z"
                    stroke="currentColor"
                    stroke-width="1.5"
                  />
                  <circle cx="10" cy="10" r="2.2" stroke="currentColor" stroke-width="1.5" />
                </svg>
                <svg v-else viewBox="0 0 20 20" fill="none">
                  <path
                    d="M3 3l14 14M8 8.2A2.2 2.2 0 0010 12c.6 0 1.1-.2 1.5-.6M5 6.4C3.2 7.7 2 10 2 10s3 5 8 5c1.2 0 2.3-.3 3.3-.7M9 5.1c.3 0 .7-.1 1-.1 5 0 8 5 8 5s-.6 1-1.7 2.1"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                  />
                </svg>
              </button>
            </div>
          </SosField>
          <SosButton type="submit" class="sos-button--block" :loading="loading">登录</SosButton>
          <div class="hauth-form__foot">
            <span class="sos-copy sos-copy--small">还没有账号？</span>
            <SosButton variant="link" type="button" @click="switchTab('forgot')">
              忘记密码？
            </SosButton>
          </div>
        </form>

        <!-- 注册 -->
        <form v-else-if="tab === 'register'" class="sos-stack" @submit.prevent="onRegister">
          <SosField label="邮箱">
            <div class="sos-input-affix">
              <span class="sos-input-affix__icon hauth-affix-icon" aria-hidden="true">
                <svg viewBox="0 0 20 20" fill="none">
                  <rect
                    x="3"
                    y="5"
                    width="14"
                    height="10"
                    rx="1.6"
                    stroke="currentColor"
                    stroke-width="1.5"
                  />
                  <path d="M4 6l6 4.5L16 6" stroke="currentColor" stroke-width="1.5" />
                </svg>
              </span>
              <input v-model="form.email" type="email" autocomplete="email" required />
            </div>
          </SosField>
          <SosField label="昵称" help="留空则用邮箱前缀">
            <input
              v-model="form.nickname"
              class="sos-input"
              maxlength="32"
              placeholder="你希望别人怎么称呼你"
            />
          </SosField>
          <SosField label="密码" help="至少 8 位">
            <div class="sos-input-affix">
              <span class="sos-input-affix__icon hauth-affix-icon" aria-hidden="true">
                <svg viewBox="0 0 20 20" fill="none">
                  <rect
                    x="4.5"
                    y="9"
                    width="11"
                    height="7"
                    rx="1.6"
                    stroke="currentColor"
                    stroke-width="1.5"
                  />
                  <path d="M7 9V6.8a3 3 0 016 0V9" stroke="currentColor" stroke-width="1.5" />
                </svg>
              </span>
              <input
                v-model="form.password"
                :type="showPw ? 'text' : 'password'"
                autocomplete="new-password"
                required
              />
              <button
                type="button"
                class="hauth-eye"
                :aria-label="showPw ? '隐藏密码' : '显示密码'"
                @click="showPw = !showPw"
              >
                <svg v-if="!showPw" viewBox="0 0 20 20" fill="none">
                  <path
                    d="M2 10s3-5 8-5 8 5 8 5-3 5-8 5-8-5-8-5z"
                    stroke="currentColor"
                    stroke-width="1.5"
                  />
                  <circle cx="10" cy="10" r="2.2" stroke="currentColor" stroke-width="1.5" />
                </svg>
                <svg v-else viewBox="0 0 20 20" fill="none">
                  <path
                    d="M3 3l14 14M8 8.2A2.2 2.2 0 0010 12M5 6.4C3.2 7.7 2 10 2 10s3 5 8 5c1.2 0 2.3-.3 3.3-.7"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                  />
                </svg>
              </button>
            </div>
          </SosField>
          <SosField label="确认密码" :error="pwMismatch ? '两次输入的密码不一致' : undefined">
            <input
              v-model="form.confirm"
              :type="showPw ? 'text' : 'password'"
              class="sos-input"
              autocomplete="new-password"
              required
            />
          </SosField>
          <SosButton type="submit" class="sos-button--block" :loading="loading">
            注册并登录
          </SosButton>
        </form>

        <!-- 找回密码 -->
        <form v-else class="sos-stack" @submit.prevent="onForgot">
          <SosField label="注册邮箱" help="我们会向该邮箱发送重置链接">
            <input
              v-model="form.email"
              class="sos-input"
              type="email"
              autocomplete="email"
              required
            />
          </SosField>
          <SosButton type="submit" class="sos-button--block" :loading="loading">
            发送重置链接
          </SosButton>
          <div style="text-align: center">
            <SosButton variant="link" type="button" @click="switchTab('login')">返回登录</SosButton>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
