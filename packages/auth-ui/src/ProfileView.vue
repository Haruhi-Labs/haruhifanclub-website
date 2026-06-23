<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  SosCard,
  SosField,
  SosInput,
  SosTextarea,
  SosButton,
  SosNotice,
  SosBadge,
  SosAvatar,
  SosEyebrow,
  SosTitle,
} from '@haruhi/ui'
import { useSession } from './useSession.js'
import './auth.css'

const props = defineProps({
  apiBase: { type: String, default: '/api' },
  loginPath: { type: String, default: '/login' },
  settingsPath: { type: String, default: '/account/settings' },
  site: { type: String, default: undefined },
})

const session = useSession(props.apiBase)
const router = useRouter()

const form = reactive({ nickname: '', avatar: '', bio: '' })
const saving = ref(false)
const error = ref('')
const okMsg = ref('')
const resendMsg = ref('')

const user = computed(() => session.state.user)

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
  <div class="hauth-root sos-scope" :data-sos-site="site">
    <div v-if="user" class="hauth-account">
      <header class="sos-stack sos-stack--tight">
        <SosEyebrow>账号</SosEyebrow>
        <SosTitle as="h1" size="xl">个人资料</SosTitle>
        <p class="sos-copy">这些信息会作为你发布内容的署名展示。</p>
      </header>

      <!-- 身份卡 -->
      <SosCard as="section">
        <div class="sos-cluster">
          <div class="hauth-identity">
            <SosAvatar :src="form.avatar || undefined" :name="form.nickname || 'U'" size="lg" />
            <div class="hauth-identity__main">
              <p class="hauth-identity__name">{{ form.nickname || '未命名' }}</p>
              <p class="hauth-identity__mail">{{ user.email || user.username }}</p>
              <div>
                <SosBadge v-if="user.emailVerified" variant="success">✓ 邮箱已验证</SosBadge>
                <SosBadge v-else variant="danger">邮箱未验证</SosBadge>
              </div>
            </div>
          </div>
          <SosButton variant="secondary" size="sm" as="a" :href="settingsPath">
            账号设置 →
          </SosButton>
        </div>

        <SosNotice
          v-if="!user.emailVerified"
          tone="warning"
          title="邮箱未验证"
          style="margin-top: var(--sos-space-5)"
        >
          验证邮箱后才能发布内容。
          <template #action>
            <SosButton variant="secondary" size="sm" @click="resend">重发验证邮件</SosButton>
          </template>
        </SosNotice>
        <SosNotice v-if="resendMsg" tone="success" style="margin-top: var(--sos-space-3)">
          {{ resendMsg }}
        </SosNotice>
      </SosCard>

      <!-- 编辑表单 -->
      <SosCard as="section">
        <SosTitle as="h2" style="font-size: var(--sos-text-lg)">编辑资料</SosTitle>
        <SosNotice v-if="error" tone="danger" style="margin-top: var(--sos-space-4)">
          {{ error }}
        </SosNotice>
        <SosNotice v-if="okMsg" tone="success" style="margin-top: var(--sos-space-4)">
          {{ okMsg }}
        </SosNotice>
        <form class="sos-stack" style="margin-top: var(--sos-space-5)" @submit.prevent="save">
          <SosField label="昵称" required>
            <SosInput v-model="form.nickname" maxlength="32" required />
          </SosField>
          <SosField label="头像 URL" help="可选，留空则显示昵称首字">
            <SosInput v-model="form.avatar" placeholder="https://…" />
          </SosField>
          <SosField label="个人简介" help="可选，最多 280 字">
            <SosTextarea
              v-model="form.bio"
              :rows="4"
              maxlength="280"
              placeholder="介绍一下你自己…"
            />
          </SosField>
          <div>
            <SosButton type="submit" :loading="saving">保存资料</SosButton>
          </div>
        </form>
      </SosCard>
    </div>
  </div>
</template>
