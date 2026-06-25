<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  home: { type: String, default: '/' },
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
  <div class="hauth-root hauth-page">
    <div class="hauth-card hauth-card--wide" v-if="user">
      <h2 class="hauth-title">账号设置</h2>
      <p class="hauth-sub">{{ user.email || user.username }}</p>

      <!-- 修改密码 -->
      <section class="hauth-section">
        <h3>修改密码</h3>
        <p class="hauth-sub">修改后，除当前设备外的所有登录都会被退出。</p>
        <div v-if="pwError" class="hauth-msg hauth-msg--err">{{ pwError }}</div>
        <div v-if="pwOk" class="hauth-msg hauth-msg--ok">{{ pwOk }}</div>
        <form @submit.prevent="changePassword">
          <div class="hauth-field">
            <label class="hauth-label">当前密码</label>
            <input class="hauth-input" type="password" v-model="pw.old" autocomplete="current-password" required />
          </div>
          <div class="hauth-field">
            <label class="hauth-label">新密码（至少 8 位）</label>
            <input class="hauth-input" type="password" v-model="pw.neu" autocomplete="new-password" required />
          </div>
          <div class="hauth-field">
            <label class="hauth-label">确认新密码</label>
            <input class="hauth-input" type="password" v-model="pw.confirm" autocomplete="new-password" required />
          </div>
          <button class="hauth-btn" :disabled="pwSaving">{{ pwSaving ? '提交中…' : '修改密码' }}</button>
        </form>
      </section>

      <!-- 登录设备 -->
      <section class="hauth-section">
        <h3>登录设备</h3>
        <p class="hauth-sub">这是你当前所有有效的登录会话，可远程下线可疑设备。</p>
        <div v-if="sessLoading" class="hauth-spin">加载中…</div>
        <ul v-else class="hauth-sessions">
          <li v-for="s in sessions" :key="s.id" class="hauth-sess">
            <div>
              <div>
                {{ shortUa(s.userAgent) }}
                <span v-if="s.current" class="hauth-badge hauth-badge--ok" style="margin-left:6px">当前设备</span>
              </div>
              <div class="hauth-sess-meta">IP {{ s.ip || '—' }} · 最近活跃 {{ fmt(s.lastSeenAt) }}</div>
            </div>
            <button v-if="!s.current" class="hauth-btn hauth-btn--sm hauth-btn--ghost" @click="revoke(s.id)">下线</button>
          </li>
        </ul>
      </section>

      <!-- 退出 -->
      <section class="hauth-section">
        <h3>退出登录</h3>
        <p class="hauth-sub">退出当前设备的登录。</p>
        <button class="hauth-btn hauth-btn--danger" @click="doLogout">退出登录</button>
      </section>
    </div>
  </div>
</template>
