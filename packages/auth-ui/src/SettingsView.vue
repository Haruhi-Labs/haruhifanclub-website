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

// 通行密钥
const passkeys = ref([])
const pkSupported = ref(false)
const pkLoading = ref(false)
const pkBusy = ref(false)
const pkError = ref('')
const pkOk = ref('')

onMounted(async () => {
  if (!session.state.ready) await session.refresh()
  if (!session.state.user) {
    router.push(props.loginPath + '?redirect=' + encodeURIComponent('/account/settings'))
    return
  }
  pkSupported.value = session.isPasskeySupported()
  await Promise.all([loadSessions(), pkSupported.value ? loadPasskeys() : Promise.resolve()])
})

async function loadPasskeys() {
  pkLoading.value = true
  try {
    const r = await session.listPasskeys()
    passkeys.value = r?.passkeys || []
  } catch {
    passkeys.value = []
  } finally {
    pkLoading.value = false
  }
}

// 添加：按钮点击即触发系统验证器（保持用户手势），成功后可再重命名
async function addPasskey() {
  pkError.value = ''
  pkOk.value = ''
  pkBusy.value = true
  try {
    await session.addPasskey()
    pkOk.value = '通行密钥已添加，可点「重命名」给它起个名字。'
    await loadPasskeys()
  } catch (e) {
    if (e?.name !== 'NotAllowedError' && e?.name !== 'AbortError') {
      pkError.value = e?.message || '添加失败，请重试'
    }
  } finally {
    pkBusy.value = false
  }
}

async function renamePasskey(p) {
  const name = window.prompt('重命名通行密钥', p.name || '')
  if (name == null || !name.trim()) return
  try {
    await session.renamePasskey(p.id, name.trim())
    await loadPasskeys()
  } catch (e) {
    pkError.value = e?.message || '重命名失败'
  }
}

async function removePasskey(id) {
  if (!window.confirm('确定删除这把通行密钥？删除后将无法用它登录。')) return
  try {
    await session.deletePasskey(id)
    await loadPasskeys()
  } catch (e) {
    pkError.value = e?.message || '删除失败'
  }
}

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

      <!-- 通行密钥 -->
      <SosCard v-if="pkSupported" as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">通行密钥</SosTitle>
        <p class="sos-copy sos-copy--small" style="margin-top: var(--sos-space-1)">
          用面容 / 指纹 / 设备 PIN 免密码登录，更安全也更省事。
        </p>
        <SosNotice v-if="pkError" tone="danger" style="margin-top: var(--sos-space-4)">
          {{ pkError }}
        </SosNotice>
        <SosNotice v-if="pkOk" tone="success" style="margin-top: var(--sos-space-4)">
          {{ pkOk }}
        </SosNotice>
        <div
          v-if="pkLoading"
          class="sos-inline"
          style="justify-content: center; margin-top: var(--sos-space-5)"
        >
          <SosSpinner />
          <span class="sos-copy sos-copy--small">加载中…</span>
        </div>
        <div v-else style="margin-top: var(--sos-space-4)">
          <p
            v-if="!passkeys.length"
            class="sos-copy sos-copy--small"
            style="color: var(--sos-text-tertiary)"
          >
            还没有通行密钥。
          </p>
          <div v-for="p in passkeys" :key="p.id" class="hauth-session">
            <div class="hauth-session__main">
              <div class="hauth-session__ua">{{ p.name || '未命名通行密钥' }}</div>
              <div class="hauth-session__meta">
                添加于 {{ fmt(p.createdAt) }}
                <template v-if="p.lastUsedAt"> · 最近使用 {{ fmt(p.lastUsedAt) }}</template>
              </div>
            </div>
            <div class="sos-inline sos-inline--tight">
              <SosButton variant="ghost" size="sm" @click="renamePasskey(p)">重命名</SosButton>
              <SosButton variant="ghost" size="sm" @click="removePasskey(p.id)">删除</SosButton>
            </div>
          </div>
        </div>
        <div style="margin-top: var(--sos-space-5)">
          <SosButton :loading="pkBusy" @click="addPasskey">＋ 添加通行密钥</SosButton>
        </div>
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
