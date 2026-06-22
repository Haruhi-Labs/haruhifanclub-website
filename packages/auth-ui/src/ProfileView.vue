<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  settingsPath: { type: String, default: '/account/settings' },
})

const session = useSession(props.apiBase)
const router = useRouter()

const form = reactive({ nickname: '', avatar: '', bio: '' })
const saving = ref(false)
const error = ref('')
const okMsg = ref('')
const resendMsg = ref('')

const user = computed(() => session.state.user)
const initial = computed(() => (form.nickname || user.value?.nickname || 'U').slice(0, 1).toUpperCase())

function load() {
  const u = session.state.user
  if (u) {
    form.nickname = u.nickname || ''
    form.avatar = u.avatar || ''
    form.bio = u.bio || ''
  }
}

onMounted(async () => {
  if (!session.state.ready) await session.refresh()
  if (!session.state.user) {
    router.push(props.loginPath + '?redirect=' + encodeURIComponent('/account'))
    return
  }
  load()
})

async function save() {
  error.value = ''
  okMsg.value = ''
  if (!form.nickname.trim()) {
    error.value = '昵称不能为空'
    return
  }
  saving.value = true
  try {
    await session.updateProfile({
      nickname: form.nickname.trim(),
      avatar: form.avatar.trim(),
      bio: form.bio.trim(),
    })
    okMsg.value = '资料已保存'
  } catch (e) {
    error.value = e?.message || '保存失败'
  } finally {
    saving.value = false
  }
}

async function resend() {
  resendMsg.value = ''
  try {
    await session.resendVerification()
    resendMsg.value = '验证邮件已重新发送，请查收。'
  } catch (e) {
    resendMsg.value = e?.message || '发送失败'
  }
}
</script>

<template>
  <div class="hauth-root hauth-page">
    <div class="hauth-card hauth-card--wide" v-if="user">
      <h2 class="hauth-title">个人资料</h2>
      <p class="hauth-sub">这些信息会作为你发布内容的署名展示。</p>

      <div class="hauth-row" style="margin-bottom:20px">
        <div style="display:flex;align-items:center;gap:14px">
          <img v-if="form.avatar" :src="form.avatar" class="hauth-avatar" style="width:56px;height:56px" alt="" />
          <div v-else class="hauth-avatar" style="width:56px;height:56px;font-size:1.3rem">{{ initial }}</div>
          <div>
            <div style="font-weight:600">{{ user.email || user.username }}</div>
            <span v-if="user.emailVerified" class="hauth-badge hauth-badge--ok">✓ 邮箱已验证</span>
            <span v-else class="hauth-badge hauth-badge--warn">邮箱未验证</span>
          </div>
        </div>
        <router-link class="hauth-link" :to="settingsPath">账号设置 →</router-link>
      </div>

      <div v-if="!user.emailVerified" class="hauth-msg hauth-msg--err" style="display:flex;justify-content:space-between;align-items:center;gap:10px">
        <span>邮箱未验证，无法发布内容。</span>
        <button class="hauth-btn hauth-btn--sm hauth-btn--ghost" @click="resend">重发验证邮件</button>
      </div>
      <div v-if="resendMsg" class="hauth-msg hauth-msg--ok">{{ resendMsg }}</div>

      <div v-if="error" class="hauth-msg hauth-msg--err">{{ error }}</div>
      <div v-if="okMsg" class="hauth-msg hauth-msg--ok">{{ okMsg }}</div>

      <form @submit.prevent="save">
        <div class="hauth-field">
          <label class="hauth-label">昵称</label>
          <input class="hauth-input" v-model="form.nickname" maxlength="32" required />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">头像 URL（可选）</label>
          <input class="hauth-input" v-model="form.avatar" placeholder="https://…" />
        </div>
        <div class="hauth-field">
          <label class="hauth-label">个人简介（可选，最多 280 字）</label>
          <textarea class="hauth-textarea" v-model="form.bio" maxlength="280"></textarea>
        </div>
        <button class="hauth-btn" :disabled="saving">{{ saving ? '保存中…' : '保存资料' }}</button>
      </form>
    </div>
  </div>
</template>
