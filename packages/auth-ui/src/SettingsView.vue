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
import QRCode from 'qrcode'
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

// 两步验证（2FA / TOTP）
const tfaEnabled = computed(() => !!session.state.user?.twoFactorEnabled)
const tfa = reactive({
  step: 'idle', // idle | setup
  uri: '',
  secret: '',
  qr: '',
  backupCodes: [],
  code: '',
  busy: false,
  error: '',
  ok: '',
})

async function start2fa() {
  tfa.error = ''
  tfa.ok = ''
  tfa.busy = true
  try {
    const r = await session.setup2fa()
    tfa.uri = r.otpauthUri
    tfa.secret = r.secret
    tfa.backupCodes = r.backupCodes || []
    tfa.qr = await QRCode.toDataURL(r.otpauthUri, { margin: 1, width: 200 })
    tfa.code = ''
    tfa.step = 'setup'
  } catch (e) {
    tfa.error = e?.message || '无法开始设置'
  } finally {
    tfa.busy = false
  }
}

async function confirm2fa() {
  tfa.error = ''
  tfa.busy = true
  try {
    await session.enable2fa(tfa.code.trim())
    await session.refresh()
    tfa.step = 'idle'
    tfa.ok = '两步验证已启用。'
    tfa.uri = tfa.secret = tfa.qr = ''
    tfa.backupCodes = []
  } catch (e) {
    tfa.error = e?.message || '验证码不正确'
  } finally {
    tfa.busy = false
  }
}

function cancel2faSetup() {
  tfa.step = 'idle'
  tfa.error = ''
  tfa.uri = tfa.secret = tfa.qr = ''
  tfa.backupCodes = []
}

async function disable2fa() {
  tfa.error = ''
  tfa.ok = ''
  const password = window.prompt('停用两步验证需验证当前密码：')
  if (password == null || !password) return
  try {
    await session.disable2fa(password)
    await session.refresh()
    tfa.ok = '两步验证已停用。'
  } catch (e) {
    tfa.error = e?.status === 401 ? '密码不正确' : e?.message || '停用失败'
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

      <!-- 两步验证 -->
      <SosCard as="section">
        <div class="sos-cluster">
          <div class="sos-stack sos-stack--tight">
            <SosTitle as="h2" style="font-size: var(--sos-text-lg)">两步验证</SosTitle>
            <p class="sos-copy sos-copy--small">
              登录时除密码外再输入验证器动态码，显著提升账号安全。
            </p>
          </div>
          <SosBadge v-if="tfaEnabled" variant="success">已启用</SosBadge>
        </div>

        <SosNotice v-if="tfa.error" tone="danger" style="margin-top: var(--sos-space-4)">
          {{ tfa.error }}
        </SosNotice>
        <SosNotice v-if="tfa.ok" tone="success" style="margin-top: var(--sos-space-4)">
          {{ tfa.ok }}
        </SosNotice>

        <!-- 已启用：停用 -->
        <div v-if="tfaEnabled" style="margin-top: var(--sos-space-4)">
          <SosButton variant="danger" @click="disable2fa">停用两步验证</SosButton>
        </div>

        <!-- 未启用 + 未进入向导：启用入口 -->
        <div v-else-if="tfa.step === 'idle'" style="margin-top: var(--sos-space-4)">
          <SosButton :loading="tfa.busy" @click="start2fa">启用两步验证</SosButton>
        </div>

        <!-- 设置向导 -->
        <div v-else class="sos-stack" style="margin-top: var(--sos-space-5)">
          <div>
            <p class="sos-copy sos-copy--small" style="margin-bottom: var(--sos-space-2)">
              1. 用验证器 App（如 1Password / Google Authenticator）扫描二维码：
            </p>
            <img
              v-if="tfa.qr"
              :src="tfa.qr"
              alt="两步验证二维码"
              width="180"
              height="180"
              style="border: 1px solid var(--sos-border-subtle); border-radius: var(--sos-radius-md)"
            />
            <p class="sos-copy sos-copy--small" style="margin-top: var(--sos-space-2)">
              无法扫描？手动输入密钥：<code>{{ tfa.secret }}</code>
            </p>
          </div>

          <div>
            <p class="sos-copy sos-copy--small" style="margin-bottom: var(--sos-space-2)">
              2. 妥善保存备用恢复码（验证器丢失时可用，每个仅一次）：
            </p>
            <div class="hauth-backup-codes">
              <code v-for="c in tfa.backupCodes" :key="c">{{ c }}</code>
            </div>
          </div>

          <form class="sos-stack sos-stack--tight" @submit.prevent="confirm2fa">
            <p class="sos-copy sos-copy--small">3. 输入验证器当前的 6 位动态码以完成启用：</p>
            <SosField label="动态验证码">
              <SosInput v-model="tfa.code" inputmode="numeric" autocomplete="one-time-code" required />
            </SosField>
            <div class="sos-inline">
              <SosButton type="submit" :loading="tfa.busy">确认启用</SosButton>
              <SosButton variant="ghost" type="button" @click="cancel2faSetup">取消</SosButton>
            </div>
          </form>
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
