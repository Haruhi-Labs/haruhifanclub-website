<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  SosCard,
  SosField,
  SosInput,
  SosButton,
  SosNotice,
  SosBadge,
  SosSpinner,
  SosEyebrow,
  SosTitle,
} from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  home: { type: String, default: '/' },
  site: { type: String, default: undefined },
})

const session = useSession(props.apiBase)
const router = useRouter()

const user = computed(() => session.state.user)

// 改密
const pw = reactive({ old: '', neu: '', confirm: '' })
const pwSaving = ref(false)
const pwError = ref('')
const pwOk = ref('')

// 会话
const sessions = ref([])
const sessLoading = ref(false)

onMounted(async () => {
  if (!session.state.ready) await session.refresh()
  if (!session.state.user) {
    router.push(props.loginPath + '?redirect=' + encodeURIComponent('/account/settings'))
    return
  }
  await loadSessions()
})

async function loadSessions() {
  sessLoading.value = true
  try {
    const r = await session.listSessions()
    sessions.value = r?.sessions || []
  } catch {
    sessions.value = []
  } finally {
    sessLoading.value = false
  }
}

async function changePassword() {
  pwError.value = ''
  pwOk.value = ''
  if (pw.neu.length < 8) {
    pwError.value = '新密码至少 8 位'
    return
  }
  if (pw.neu !== pw.confirm) {
    pwError.value = '两次输入的新密码不一致'
    return
  }
  pwSaving.value = true
  try {
    await session.changePassword(pw.old, pw.neu)
    pwOk.value = '密码已修改，其它设备的登录已退出。'
    pw.old = pw.neu = pw.confirm = ''
    await loadSessions()
  } catch (e) {
    pwError.value = e?.message || '修改失败'
  } finally {
    pwSaving.value = false
  }
}

async function revoke(id) {
  try {
    await session.revokeSession(id)
    await loadSessions()
  } catch {
    /* ignore */
  }
}

async function doLogout() {
  await session.logout()
  router.push(props.home)
}

function fmt(s) {
  if (!s) return '—'
  return String(s).replace('T', ' ').slice(0, 16)
}
function shortUa(ua) {
  if (!ua) return '未知设备'
  return ua.slice(0, 48)
}
</script>

<template>
  <div class="hauth-root sos-scope" :data-sos-site="site">
    <div v-if="user" class="hauth-account">
      <header class="sos-stack sos-stack--tight">
        <SosEyebrow>账号</SosEyebrow>
        <SosTitle as="h1" size="xl">账号设置</SosTitle>
        <p class="sos-copy">{{ user.email || user.username }}</p>
      </header>

      <!-- 修改密码 -->
      <SosCard as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">修改密码</SosTitle>
        <p class="sos-copy sos-copy--small" style="margin-top: var(--sos-space-1)">
          修改后，除当前设备外的所有登录都会被退出。
        </p>
        <SosNotice v-if="pwError" tone="danger" style="margin-top: var(--sos-space-4)">
          {{ pwError }}
        </SosNotice>
        <SosNotice v-if="pwOk" tone="success" style="margin-top: var(--sos-space-4)">
          {{ pwOk }}
        </SosNotice>
        <form
          class="sos-stack"
          style="margin-top: var(--sos-space-5)"
          @submit.prevent="changePassword"
        >
          <SosField label="当前密码">
            <SosInput v-model="pw.old" type="password" autocomplete="current-password" required />
          </SosField>
          <div class="sos-form-row">
            <SosField label="新密码" help="至少 8 位">
              <SosInput v-model="pw.neu" type="password" autocomplete="new-password" required />
            </SosField>
            <SosField label="确认新密码">
              <SosInput v-model="pw.confirm" type="password" autocomplete="new-password" required />
            </SosField>
          </div>
          <div>
            <SosButton type="submit" :loading="pwSaving">修改密码</SosButton>
          </div>
        </form>
      </SosCard>

      <!-- 登录设备 -->
      <SosCard as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">登录设备</SosTitle>
        <p class="sos-copy sos-copy--small" style="margin-top: var(--sos-space-1)">
          你当前所有有效的登录会话，可远程下线可疑设备。
        </p>
        <div
          v-if="sessLoading"
          class="sos-inline"
          style="justify-content: center; margin-top: var(--sos-space-5)"
        >
          <SosSpinner />
          <span class="sos-copy sos-copy--small">加载中…</span>
        </div>
        <div v-else style="margin-top: var(--sos-space-4)">
          <div v-for="s in sessions" :key="s.id" class="hauth-session">
            <div class="hauth-session__main">
              <div class="hauth-session__ua">
                {{ shortUa(s.userAgent) }}
                <SosBadge v-if="s.current" variant="success">当前设备</SosBadge>
              </div>
              <div class="hauth-session__meta">
                IP {{ s.ip || '—' }} · 最近活跃 {{ fmt(s.lastSeenAt) }}
              </div>
            </div>
            <SosButton v-if="!s.current" variant="ghost" size="sm" @click="revoke(s.id)">
              下线
            </SosButton>
          </div>
        </div>
      </SosCard>

      <!-- 退出 -->
      <SosCard as="section">
        <div class="sos-cluster">
          <div class="sos-stack sos-stack--tight">
            <SosTitle as="h2" style="font-size: var(--sos-text-lg)">退出登录</SosTitle>
            <p class="sos-copy sos-copy--small">退出当前设备的登录。</p>
          </div>
          <SosButton variant="danger" @click="doLogout">退出登录</SosButton>
        </div>
      </SosCard>
    </div>
  </div>
</template>
