<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { SosCard, SosField, SosInput, SosButton, SosNotice, SosTabs, SosDivider } from '@haruhi/ui'
import { useSession } from './useSession.js'
import brandLogo from './assets/haruhi-logo-192.png'
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

// 通行密钥（Passkey）
const supportsPasskey = ref(false)
const passkeyLoading = ref(false)
let conditionalAbort = null

// 两步验证（2FA）二次校验步骤
const twoFactor = reactive({ active: false, pendingToken: '', code: '', backup: false })

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
    const r = await session.login(form.account.trim(), form.password)
    if (r && r.twoFactorRequired) {
      // 进入两步验证步骤
      twoFactor.active = true
      twoFactor.pendingToken = r.pendingToken
      twoFactor.code = ''
      twoFactor.backup = false
      return
    }
    go()
  } catch (e) {
    error.value = e?.status === 401 ? '邮箱/用户名或密码错误' : e?.message || '登录失败'
  } finally {
    loading.value = false
  }
}

async function on2fa() {
  error.value = ''
  loading.value = true
  try {
    await session.login2fa(twoFactor.pendingToken, twoFactor.code.trim(), twoFactor.backup)
    go()
  } catch (e) {
    error.value = e?.message || '验证失败，请重试'
  } finally {
    loading.value = false
  }
}

function cancel2fa() {
  twoFactor.active = false
  twoFactor.pendingToken = ''
  twoFactor.code = ''
  error.value = ''
}

// 手动点击「通行密钥登录」
async function onPasskey() {
  error.value = ''
  passkeyLoading.value = true
  try {
    await session.loginPasskey()
    go()
  } catch (e) {
    // 用户取消系统弹窗不算错误
    if (e?.name !== 'NotAllowedError' && e?.name !== 'AbortError') {
      error.value = e?.message || '通行密钥登录失败'
    }
  } finally {
    passkeyLoading.value = false
  }
}

// 条件式自动填充：页面加载即在后台等待用户从输入框自动填充里选择通行密钥
onMounted(async () => {
  await session.ensureReady({ force: true })
  if (session.state.user) {
    go()
    return
  }

  supportsPasskey.value = session.isPasskeySupported()
  if (!supportsPasskey.value) return
  if (!(await session.isConditionalUiAvailable())) return
  try {
    conditionalAbort = new AbortController()
    await session.loginPasskey({ conditional: true, signal: conditionalAbort.signal })
    go()
  } catch {
    /* 条件式自动填充被取消/未选择，静默忽略 */
  }
})

onBeforeUnmount(() => conditionalAbort?.abort())

async function onRegister() {
  error.value = ''
  // 昵称地位等同用户名：必填、唯一
  if (!form.nickname.trim()) {
    error.value = '请填写昵称'
    return
  }
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
      nickname: form.nickname.trim(),
    })
    okMsg.value = '注册成功，已自动登录。'
    setTimeout(go, 1200)
  } catch (e) {
    // 409 既可能是邮箱已注册、也可能是昵称已占用，直接透传后端文案
    error.value = e?.message || '注册失败'
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
        <img class="hauth-brand__mark" :src="brandLogo" alt="" />
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

      <div
        v-if="tab !== 'forgot' && !twoFactor.active"
        style="display: flex; justify-content: center"
      >
        <SosTabs :model-value="tab" :items="tabItems" @update:model-value="switchTab" />
      </div>

      <SosNotice v-if="error" tone="danger" style="margin-top: var(--sos-space-4)">
        {{ error }}
      </SosNotice>
      <SosNotice v-if="okMsg" tone="success" style="margin-top: var(--sos-space-4)">
        {{ okMsg }}
      </SosNotice>

      <!-- 账号 / 密码 / 找回（两步验证激活时整体隐藏） -->
      <template v-if="!twoFactor.active">
        <!-- 登录 -->
        <form
          v-if="tab === 'login'"
          class="sos-stack"
          style="margin-top: var(--sos-space-5)"
          @submit.prevent="onLogin"
        >
          <SosField label="邮箱或用户名">
            <SosInput v-model="form.account" autocomplete="username webauthn" required />
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

          <template v-if="supportsPasskey">
            <SosDivider />
            <SosButton
              variant="secondary"
              type="button"
              class="sos-button--block"
              :loading="passkeyLoading"
              @click="onPasskey"
            >
              <span aria-hidden="true" style="margin-right: 0.4em">🔑</span>使用通行密钥登录
            </SosButton>
          </template>
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
          <SosField label="昵称" required help="别人看到的名字，需唯一、不可与他人重名">
            <SosInput v-model="form.nickname" maxlength="32" required placeholder="你希望别人怎么称呼你" />
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
      </template>

      <!-- 两步验证：密码通过后输入动态码 / 备用恢复码 -->
      <form
        v-else
        class="sos-stack"
        style="margin-top: var(--sos-space-5)"
        @submit.prevent="on2fa"
      >
        <p class="sos-copy sos-copy--small">
          {{ twoFactor.backup ? '输入一个未使用的备用恢复码' : '输入验证器 App 上的 6 位动态码' }}
        </p>
        <SosField :label="twoFactor.backup ? '备用恢复码' : '动态验证码'">
          <SosInput
            v-model="twoFactor.code"
            :inputmode="twoFactor.backup ? 'text' : 'numeric'"
            autocomplete="one-time-code"
            required
          />
        </SosField>
        <SosButton type="submit" class="sos-button--block" :loading="loading">验证并登录</SosButton>
        <div style="text-align: center">
          <SosButton
            variant="link"
            type="button"
            @click="twoFactor.backup = !twoFactor.backup"
          >
            {{ twoFactor.backup ? '改用动态验证码' : '改用备用恢复码' }}
          </SosButton>
          <SosButton variant="link" type="button" @click="cancel2fa">返回登录</SosButton>
        </div>
      </form>
    </SosCard>
  </div>
</template>
